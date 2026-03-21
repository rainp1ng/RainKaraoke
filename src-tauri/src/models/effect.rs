use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectSlot {
    pub id: i64,
    pub slot_index: i32,
    pub effect_type: String,
    pub is_enabled: bool,
    pub parameters: String,  // JSON
    /// MIDI 音符编号 (0-127)，用于通过 MIDI 控制开关
    pub midi_note: Option<i32>,
    /// MIDI 通道 (0-15)
    pub midi_channel: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectChainConfig {
    pub input_device_id: Option<String>,
    pub input_volume: f32,
    pub monitor_device_id: Option<String>,
    pub stream_device_id: Option<String>,
    pub monitor_volume: f32,
    pub stream_volume: f32,
    pub bypass_all: bool,
    // 新增字段
    pub vocal_input_device: Option<String>,
    pub instrument_input_device: Option<String>,
    pub vocal_input_channel: i32,
    pub instrument_input_channel: i32,
    pub vocal_volume: f32,
    pub instrument_volume: f32,
    pub effect_input: String,
    pub recording_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetEffectSlot {
    pub slot_index: i32,
    pub effect_type: String,
    pub enabled: Option<bool>,
    pub parameters: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEffectParameters {
    pub slot_index: i32,
    pub parameters: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectPreset {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewEffectPreset {
    pub name: String,
    pub description: Option<String>,
}

// 默认效果器参数
pub fn get_default_parameters(effect_type: &str) -> String {
    match effect_type {
        "reverb" => serde_json::json!({
            "roomSize": 50,
            "damping": 30,
            "wetLevel": 30,
            "dryLevel": 70,
            "preDelay": 10
        }).to_string(),
        "chorus" => serde_json::json!({
            "rate": 1.5,
            "depth": 50,
            "mix": 30,
            "voices": 4,
            "spread": 50
        }).to_string(),
        "eq" => serde_json::json!({
            "low": {"gain": 0, "frequency": 100, "q": 0.7},
            "lowMid": {"gain": 0, "frequency": 500, "q": 0.7},
            "highMid": {"gain": 0, "frequency": 4000, "q": 0.7},
            "high": {"gain": 0, "frequency": 12000, "q": 0.7},
            "lowCut": {"enabled": false, "frequency": 80},
            "highCut": {"enabled": false, "frequency": 12000}
        }).to_string(),
        "compressor" => serde_json::json!({
            "threshold": -24,
            "ratio": 4,
            "attack": 10,
            "release": 100,
            "makeupGain": 0
        }).to_string(),
        "delay" => serde_json::json!({
            "time": 250,
            "feedback": 30,
            "mix": 20,
            "pingPong": false
        }).to_string(),
        "deesser" => serde_json::json!({
            "frequency": 6000,
            "threshold": -20,
            "range": 6
        }).to_string(),
        "exciter" => serde_json::json!({
            "frequency": 8000,
            "harmonics": 30,
            "mix": 20
        }).to_string(),
        "gate" => serde_json::json!({
            "threshold": -50,
            "attack": 1,
            "release": 50,
            "range": 40
        }).to_string(),
        "gain" => serde_json::json!({
            "gainDb": 0
        }).to_string(),
        _ => "{}".to_string(),
    }
}
