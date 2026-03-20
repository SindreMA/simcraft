use regex::Regex;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

pub const GEAR_SLOTS: &[&str] = &[
    "head", "neck", "shoulder", "back", "chest", "wrist",
    "hands", "waist", "legs", "feet", "finger1", "finger2",
    "trinket1", "trinket2", "main_hand", "off_hand",
];

fn paired_slot(slot: &str) -> Option<&'static str> {
    match slot {
        "finger1" => Some("finger2"),
        "finger2" => Some("finger1"),
        "trinket1" => Some("trinket2"),
        "trinket2" => Some("trinket1"),
        _ => None,
    }
}

pub fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_item_props(item_str: &str) -> Value {
    let mut item_id: u64 = 0;
    let mut ilevel: u64 = 0;
    let mut name = String::new();
    let mut bonus_ids: Vec<u64> = Vec::new();
    let mut enchant_id: u64 = 0;
    let mut gem_id: u64 = 0;

    if let Some(caps) = Regex::new(r"id=(\d+)").unwrap().captures(item_str) {
        item_id = caps[1].parse().unwrap_or(0);
    }

    if let Some(caps) = Regex::new(r"(?:ilevel|ilvl)=(\d+)")
        .unwrap()
        .captures(item_str)
    {
        ilevel = caps[1].parse().unwrap_or(0);
    }

    if let Some(caps) = Regex::new(r"bonus_id=([0-9/:]+)")
        .unwrap()
        .captures(item_str)
    {
        bonus_ids = caps[1]
            .split(&['/', ':'][..])
            .filter_map(|s| s.parse().ok())
            .collect();
    }

    if let Some(caps) = Regex::new(r"enchant_id=(\d+)")
        .unwrap()
        .captures(item_str)
    {
        enchant_id = caps[1].parse().unwrap_or(0);
    }

    if let Some(caps) = Regex::new(r"gem_id=(\d+)").unwrap().captures(item_str) {
        gem_id = caps[1].parse().unwrap_or(0);
    }

    if let Some(caps) = Regex::new(r"name=([^,]+)").unwrap().captures(item_str) {
        name = title_case(&caps[1].replace('_', " "));
    }

    if name.is_empty() {
        if let Some(caps) = Regex::new(r"^([a-z_]+),").unwrap().captures(item_str) {
            name = title_case(&caps[1].replace('_', " "));
        }
    }

    json!({
        "item_id": item_id,
        "ilevel": ilevel,
        "name": name,
        "bonus_ids": bonus_ids,
        "enchant_id": enchant_id,
        "gem_id": gem_id,
    })
}

pub fn parse_addon_string(simc_input: &str) -> Value {
    let slot_pattern = format!(r"^({})=(.*)", GEAR_SLOTS.join("|"));
    let slot_re = Regex::new(&slot_pattern).unwrap();
    let header_re = Regex::new(r"^#+\s*(.+?)\s*\((\d+)\)\s*$").unwrap();

    let mut equipped: HashMap<String, Value> = HashMap::new();
    let mut bag_items: HashMap<String, Vec<Value>> = HashMap::new();
    let mut base_profile_lines: Vec<String> = Vec::new();
    let mut pending_name = String::new();
    let mut pending_ilevel: u64 = 0;

    for raw_line in simc_input.lines() {
        let stripped = raw_line.trim();

        if stripped.starts_with('#') {
            let clean = stripped.trim_start_matches('#').trim();
            if let Some(caps) = slot_re.captures(clean) {
                let slot = caps[1].to_lowercase();
                let item_str = caps[2].to_string();
                let mut props = parse_item_props(&item_str);

                if props["name"].as_str() == Some("") && !pending_name.is_empty() {
                    props["name"] = json!(pending_name.clone());
                }
                if props["ilevel"].as_u64() == Some(0) && pending_ilevel > 0 {
                    props["ilevel"] = json!(pending_ilevel);
                }
                pending_name.clear();
                pending_ilevel = 0;

                let entry = json!({
                    "slot": slot,
                    "simc_string": item_str,
                    "is_equipped": false,
                    "item_id": props["item_id"],
                    "ilevel": props["ilevel"],
                    "name": props["name"],
                    "bonus_ids": props["bonus_ids"],
                    "enchant_id": props["enchant_id"],
                    "gem_id": props["gem_id"],
                });

                bag_items.entry(slot.clone()).or_default().push(entry.clone());

                if let Some(other) = paired_slot(&slot) {
                    let mut paired = entry.clone();
                    paired["slot"] = json!(other);
                    bag_items
                        .entry(other.to_string())
                        .or_default()
                        .push(paired);
                }
            } else if let Some(caps) = header_re.captures(stripped) {
                pending_name = caps[1].to_string();
                pending_ilevel = caps[2].parse().unwrap_or(0);
            } else {
                pending_name.clear();
                pending_ilevel = 0;
            }
        } else {
            base_profile_lines.push(stripped.to_string());
            if let Some(caps) = slot_re.captures(stripped) {
                let slot = caps[1].to_lowercase();
                let item_str = caps[2].to_string();
                let mut props = parse_item_props(&item_str);

                if props["name"].as_str() == Some("") && !pending_name.is_empty() {
                    props["name"] = json!(pending_name.clone());
                }
                if props["ilevel"].as_u64() == Some(0) && pending_ilevel > 0 {
                    props["ilevel"] = json!(pending_ilevel);
                }
                pending_name.clear();
                pending_ilevel = 0;

                equipped.insert(
                    slot.clone(),
                    json!({
                        "slot": slot,
                        "simc_string": item_str,
                        "is_equipped": true,
                        "item_id": props["item_id"],
                        "ilevel": props["ilevel"],
                        "name": props["name"],
                        "bonus_ids": props["bonus_ids"],
                        "enchant_id": props["enchant_id"],
                        "gem_id": props["gem_id"],
                    }),
                );
            }
        }
    }

    // Build items_by_slot
    let mut items_by_slot: HashMap<String, Vec<Value>> = HashMap::new();
    for slot in GEAR_SLOTS {
        let slot = slot.to_string();
        let mut items: Vec<Value> = Vec::new();
        let mut seen_ids: HashSet<u64> = HashSet::new();

        if let Some(eq) = equipped.get(&slot) {
            items.push(eq.clone());
            let iid = eq["item_id"].as_u64().unwrap_or(0);
            if iid > 0 {
                seen_ids.insert(iid);
            }
        }

        if let Some(bag) = bag_items.get(&slot) {
            for item in bag {
                let iid = item["item_id"].as_u64().unwrap_or(0);
                if iid > 0 && seen_ids.contains(&iid) {
                    continue;
                }
                if iid > 0 {
                    seen_ids.insert(iid);
                }
                items.push(item.clone());
            }
        }

        if !items.is_empty() {
            items_by_slot.insert(slot, items);
        }
    }

    json!({
        "base_profile": base_profile_lines.join("\n"),
        "items_by_slot": items_by_slot,
    })
}
