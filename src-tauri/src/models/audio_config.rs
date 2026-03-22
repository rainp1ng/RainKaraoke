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
    pub ducking_recovery_delay: i32, // 恢复延迟（秒，1-9）

    pub midi_device_id: Option<String>,
    pub midi_enabled: bool,

    // 气氛组停止按钮 MIDI 配置
    pub atmosphere_stop_midi_message_type: Option<String>,
    pub atmosphere_stop_midi_note: Option<i32>,
    pub atmosphere_stop_midi_channel: Option<i32>,
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
    pub ducking_recovery_delay: Option<i32>,
    pub midi_device_id: Option<String>,
    pub midi_enabled: Option<bool>,
    pub atmosphere_stop_midi_message_type: Option<String>,
    pub atmosphere_stop_midi_note: Option<i32>,
    pub atmosphere_stop_midi_channel: Option<i32>,
}
