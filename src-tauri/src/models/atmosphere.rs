use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtmosphereSound {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub duration: Option<i32>,
    pub volume: f32,
    pub midi_note: Option<i32>,
    pub midi_channel: i32,
    pub is_one_shot: bool,
    pub color: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAtmosphereSound {
    pub name: String,
    pub file_path: String,
    pub volume: Option<f32>,
    pub midi_note: Option<i32>,
    pub midi_channel: Option<i32>,
    pub is_one_shot: Option<bool>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAtmosphereSound {
    pub id: i64,
    pub name: Option<String>,
    pub volume: Option<f32>,
    pub midi_note: Option<i32>,
    pub midi_channel: Option<i32>,
    pub is_one_shot: Option<bool>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}
