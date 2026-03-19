use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterludeTrack {
    pub id: i64,
    pub title: Option<String>,
    pub file_path: String,
    pub duration: Option<i32>,
    pub volume: f32,
    pub is_active: bool,
    pub play_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInterludeTrack {
    pub title: Option<String>,
    pub file_path: String,
    pub volume: Option<f32>,
}
