use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub status: JobStatus,
    pub sim_type: String,
    pub simc_input: String,
    pub result_json: Option<String>,
    pub combo_metadata_json: Option<String>,
    pub error_message: Option<String>,
    pub progress_pct: u8,
    pub progress_stage: Option<String>,
    pub progress_detail: Option<String>,
    pub stages_completed: Vec<String>,
    pub iterations: u32,
    pub fight_style: String,
    pub target_error: f64,
    pub created_at: String,
}

impl Job {
    pub fn new(
        simc_input: String,
        sim_type: String,
        iterations: u32,
        fight_style: String,
        target_error: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            status: JobStatus::Pending,
            sim_type,
            simc_input,
            result_json: None,
            combo_metadata_json: None,
            error_message: None,
            progress_pct: 0,
            progress_stage: None,
            progress_detail: None,
            stages_completed: Vec::new(),
            iterations,
            fight_style,
            target_error,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

pub struct JobStore {
    pub jobs: Mutex<HashMap<String, Job>>,
}

impl JobStore {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, job: Job) {
        self.jobs.lock().unwrap().insert(job.id.clone(), job);
    }

    pub fn get(&self, id: &str) -> Option<Job> {
        self.jobs.lock().unwrap().get(id).cloned()
    }

    pub fn update_status(&self, id: &str, status: JobStatus) {
        if let Some(job) = self.jobs.lock().unwrap().get_mut(id) {
            job.status = status;
        }
    }

    pub fn update_progress(&self, id: &str, pct: u8, stage: &str, detail: &str) {
        if let Some(job) = self.jobs.lock().unwrap().get_mut(id) {
            job.progress_pct = pct;
            job.progress_stage = Some(stage.to_string());
            job.progress_detail = Some(detail.to_string());
        }
    }

    pub fn complete_stage(&self, id: &str, summary: &str) {
        if let Some(job) = self.jobs.lock().unwrap().get_mut(id) {
            job.stages_completed.push(summary.to_string());
        }
    }

    pub fn set_result(&self, id: &str, result: String) {
        if let Some(job) = self.jobs.lock().unwrap().get_mut(id) {
            job.result_json = Some(result);
            job.status = JobStatus::Done;
        }
    }

    pub fn set_error(&self, id: &str, error: String) {
        if let Some(job) = self.jobs.lock().unwrap().get_mut(id) {
            job.error_message = Some(error);
            job.status = JobStatus::Failed;
        }
    }
}
