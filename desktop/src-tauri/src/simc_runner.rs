use regex::Regex;
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;

const SIMC_TIMEOUT_SECS: u64 = 600;

fn simc_threads() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4)
}

const OVERRIDES: &[&str] = &[
    "override.bloodlust=1",
    "override.arcane_intellect=1",
    "override.power_word_fortitude=1",
    "override.battle_shout=1",
    "override.mystic_touch=1",
    "override.chaos_brand=1",
    "override.skyfury=1",
    "override.mark_of_the_wild=1",
    "override.hunters_mark=1",
    "override.bleeding=1",
];

const SIM_OPTIONS: &[&str] = &[
    "report_details=1",
    "single_actor_batch=1",
    "optimize_expressions=1",
    "desired_targets=1",
    "max_time=300",
    "temporary_enchant=",
    "scale_only=strength,intellect,agility,crit,mastery,vers,haste,weapon_dps,weapon_offhand_dps",
];

const EXPANSION_OPTIONS: &[&str] = &[
    "midnight.crucible_of_erratic_energies_violence=1",
    "midnight.crucible_of_erratic_energies_sustenance=1",
    "midnight.crucible_of_erratic_energies_predation=1",
];

struct Stage {
    name: &'static str,
    target_error: f64,
    keep_top: f64,
    min_keep: usize,
}

const STAGES: &[Stage] = &[
    Stage { name: "Low",    target_error: 1.0,  keep_top: 0.5, min_keep: 10 },
    Stage { name: "Medium", target_error: 0.2,  keep_top: 0.3, min_keep: 5  },
    Stage { name: "High",   target_error: 0.05, keep_top: 1.0, min_keep: 1  },
];

const STAGED_THRESHOLD: usize = 10;

async fn run_simc_subprocess(
    simc_path: &Path,
    job_id: &str,
    simc_input: &str,
    fight_style: &str,
    target_error: f64,
    iterations: u32,
    calculate_scale_factors: bool,
    stage_name: &str,
) -> Result<Value, String> {
    let suffix = if stage_name.is_empty() {
        String::new()
    } else {
        format!("_{}", stage_name)
    };

    let tmp_dir = TempDir::with_prefix(&format!("simc_{}{}_", job_id, suffix))
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let input_file = tmp_dir.path().join("input.simc");
    let output_file = tmp_dir.path().join("output.json");

    std::fs::write(&input_file, simc_input)
        .map_err(|e| format!("Failed to write input file: {}", e))?;

    let mut cmd = Command::new(simc_path);
    cmd.arg(input_file.to_str().unwrap_or(""))
        .arg(format!("json2={}", output_file.display()))
        .arg(format!("iterations={}", iterations))
        .arg(format!("fight_style={}", fight_style))
        .arg(format!("target_error={}", target_error))
        .arg(format!("threads={}", simc_threads()))
        .arg(format!(
            "calculate_scale_factors={}",
            if calculate_scale_factors { "1" } else { "0" }
        ));

    for opt in OVERRIDES {
        cmd.arg(*opt);
    }
    for opt in SIM_OPTIONS {
        cmd.arg(*opt);
    }
    for opt in EXPANSION_OPTIONS {
        cmd.arg(*opt);
    }

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(SIMC_TIMEOUT_SECS),
        cmd.output(),
    )
    .await
    .map_err(|_| format!("simc timed out after {}s", SIMC_TIMEOUT_SECS))?
    .map_err(|e| format!("Failed to run simc: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_msg = if !stderr.is_empty() {
            stderr.to_string()
        } else if !stdout.is_empty() {
            stdout.to_string()
        } else {
            "simc exited with non-zero code".to_string()
        };
        return Err(format!(
            "simc failed (exit {:?}): {}",
            output.status.code(),
            error_msg
        ));
    }

    if !output_file.exists() {
        return Err("simc did not produce output JSON".to_string());
    }

    let json_text = std::fs::read_to_string(&output_file)
        .map_err(|e| format!("Failed to read output JSON: {}", e))?;

    serde_json::from_str(&json_text)
        .map_err(|e| format!("Failed to parse output JSON: {}", e))
}

fn get_profileset_results(raw: &Value) -> Vec<Value> {
    raw.get("sim")
        .and_then(|s| s.get("profilesets"))
        .and_then(|p| p.get("results"))
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default()
}

pub fn filter_simc_input(simc_input: &str, keep_combos: &std::collections::HashSet<String>) -> String {
    let header_re = Regex::new(r"^###\s+(Combo \d+)").unwrap();
    let lines: Vec<&str> = simc_input.split('\n').collect();
    let mut output: Vec<&str> = Vec::new();
    let mut current_combo: Option<String> = None;
    let mut in_kept_combo = true;

    for line in &lines {
        if let Some(caps) = header_re.captures(line) {
            let combo_name = caps[1].to_string();
            in_kept_combo = keep_combos.contains(&combo_name);
            current_combo = Some(combo_name);
            if in_kept_combo {
                output.push(line);
            }
            continue;
        }

        if line.trim().starts_with("profileset.") {
            if in_kept_combo {
                output.push(line);
            }
            continue;
        }

        if current_combo.is_some() && line.trim().starts_with('#') {
            if in_kept_combo {
                output.push(line);
            }
            continue;
        }

        output.push(line);
        current_combo = None;
        in_kept_combo = true;
    }

    output.join("\n")
}

/// Run simc and return parsed JSON output.
pub async fn run_simc(
    simc_path: &Path,
    job_id: &str,
    simc_input: &str,
    options: &Value,
) -> Result<Value, String> {
    let fight_style = options
        .get("fight_style")
        .and_then(|v| v.as_str())
        .unwrap_or("Patchwerk");
    let target_error = options
        .get("target_error")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.2);
    let iterations = options
        .get("iterations")
        .and_then(|v| v.as_u64())
        .unwrap_or(1000) as u32;
    let calculate_scale_factors = options
        .get("sim_type")
        .and_then(|v| v.as_str())
        == Some("stat_weights");

    run_simc_subprocess(
        simc_path,
        job_id,
        simc_input,
        fight_style,
        target_error,
        iterations,
        calculate_scale_factors,
        "",
    )
    .await
}

/// Run a multi-stage simulation for Top Gear.
pub async fn run_simc_staged(
    simc_path: &Path,
    job_id: &str,
    simc_input: &str,
    options: &Value,
    combo_count: usize,
    on_progress: impl Fn(u8, &str, &str),
    on_stage_complete: impl Fn(&str),
) -> Result<Value, String> {
    let fight_style = options
        .get("fight_style")
        .and_then(|v| v.as_str())
        .unwrap_or("Patchwerk");
    let user_iterations = options
        .get("iterations")
        .and_then(|v| v.as_u64())
        .unwrap_or(1000) as u32;

    if combo_count < STAGED_THRESHOLD {
        on_progress(10, "Simulating", &format!("{} combos", combo_count));
        let target_error = options
            .get("target_error")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.2);
        return run_simc_subprocess(
            simc_path,
            job_id,
            simc_input,
            fight_style,
            target_error,
            user_iterations,
            false,
            "direct",
        )
        .await;
    }

    let mut current_input = simc_input.to_string();
    let mut remaining = combo_count;
    let mut result: Option<Value> = None;

    let stage_iterations = [
        std::cmp::max(100, user_iterations / 10),
        std::cmp::max(500, user_iterations / 2),
        user_iterations,
    ];

    for (stage_idx, stage) in STAGES.iter().enumerate() {
        let is_final = stage_idx == STAGES.len() - 1;

        let stage_pct = match stage_idx {
            0 => 10,
            1 => 40,
            2 => 70,
            _ => 80,
        };
        on_progress(
            stage_pct,
            &format!("Stage {} of {}", stage_idx + 1, STAGES.len()),
            &format!("{} combos · {} precision", remaining, stage.name),
        );

        println!(
            "Job {}: Stage {} — {} combos, target_error={}, iterations={}",
            job_id, stage.name, remaining, stage.target_error, stage_iterations[stage_idx]
        );

        let stage_result = run_simc_subprocess(
            simc_path,
            job_id,
            &current_input,
            fight_style,
            stage.target_error,
            stage_iterations[stage_idx],
            false,
            &stage.name.to_lowercase(),
        )
        .await?;

        result = Some(stage_result);

        if is_final {
            on_stage_complete(&format!("{} · {} combos · done", stage.name, remaining));
            break;
        }

        let profilesets = get_profileset_results(result.as_ref().unwrap());
        if profilesets.is_empty() {
            on_stage_complete(&format!("{} · no results", stage.name));
            break;
        }

        let keep_count = std::cmp::max(
            stage.min_keep,
            (profilesets.len() as f64 * stage.keep_top) as usize,
        );

        if keep_count >= profilesets.len() {
            on_stage_complete(&format!("{} · kept all {} combos", stage.name, profilesets.len()));
            continue;
        }

        let mut sorted_ps = profilesets.clone();
        sorted_ps.sort_by(|a, b| {
            let a_mean = a.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b_mean = b.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0);
            b_mean.partial_cmp(&a_mean).unwrap_or(std::cmp::Ordering::Equal)
        });

        let keep_combos: std::collections::HashSet<String> = sorted_ps
            .iter()
            .take(keep_count)
            .filter_map(|ps| ps.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            .collect();

        on_stage_complete(&format!("{} · {} → {} combos", stage.name, profilesets.len(), keep_combos.len()));

        println!(
            "Job {}: Stage {} complete — keeping {}/{} combos",
            job_id,
            stage.name,
            keep_combos.len(),
            profilesets.len()
        );

        current_input = filter_simc_input(&current_input, &keep_combos);
        remaining = keep_combos.len();
    }

    result.ok_or_else(|| "No simulation result produced".to_string())
}
