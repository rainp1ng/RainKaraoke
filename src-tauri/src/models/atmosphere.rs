use serde::{Deserialize, Serialize};

/// MIDI 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum MidiMessageType {
    #[default]
    Note,
    CC,
    PC,
}

impl std::fmt::Display for MidiMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MidiMessageType::Note => write!(f, "NOTE"),
            MidiMessageType::CC => write!(f, "CC"),
            MidiMessageType::PC => write!(f, "PC"),
        }
    }
}

impl From<&str> for MidiMessageType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "CC" => MidiMessageType::CC,
            "PC" => MidiMessageType::PC,
            _ => MidiMessageType::Note,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtmosphereSound {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub duration: Option<i32>,
    pub volume: f32,
    pub midi_message_type: MidiMessageType,
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
    pub midi_message_type: Option<MidiMessageType>,
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
    pub midi_message_type: Option<MidiMessageType>,
    pub midi_note: Option<i32>,
    pub midi_channel: Option<i32>,
    pub is_one_shot: Option<bool>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}
