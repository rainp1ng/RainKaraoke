use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioConfig {
    pub default_output_device: Option<String>,
    pub interlude_output_device: Option<String>,
    pub atmosphere_output_device: Option<String>,

    pub master_volume: f32,
    pub interlude_volume: f32,
    pub atmosphere_volume: f32,

    pub ducking_enabled: bool,
    pub ducking_threshold: f32,
    pub ducking_ratio: f32,
    pub ducking_attack_ms: i32,
    pub ducking_release_ms: i32,

    pub midi_device_id: Option<String>,
    pub midi_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAudioConfig {
    pub default_output_device: Option<String>,
    pub interlude_output_device: Option<String>,
    pub atmosphere_output_device: Option<String>,
    pub master_volume: Option<f32>,
    pub interlude_volume: Option<f32>,
    pub atmosphere_volume: Option<f32>,
    pub ducking_enabled: Option<bool>,
    pub ducking_threshold: Option<f32>,
    pub ducking_ratio: Option<f32>,
    pub ducking_attack_ms: Option<i32>,
    pub ducking_release_ms: Option<i32>,
    pub midi_device_id: Option<String>,
    pub midi_enabled: Option<bool>,
}
