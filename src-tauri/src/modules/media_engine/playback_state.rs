use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Idle,
    Playing,
    Paused,
}

impl PlaybackStatus {
    pub fn to_string(&self) -> String {
        match self {
            PlaybackStatus::Idle => "idle".to_string(),
            PlaybackStatus::Playing => "playing".to_string(),
            PlaybackStatus::Paused => "paused".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub current_song_id: Option<i64>,
    pub current_video_path: Option<String>,
    pub current_audio_path: Option<String>,
    pub current_time: f64,
    pub duration: f64,
    pub is_vocal: bool,
    pub pitch: i32,
    pub speed: f64,
    pub volume: f64,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            status: PlaybackStatus::Idle,
            current_song_id: None,
            current_video_path: None,
            current_audio_path: None,
            current_time: 0.0,
            duration: 0.0,
            is_vocal: true,
            pitch: 0,
            speed: 1.0,
            volume: 0.8,
        }
    }
}
