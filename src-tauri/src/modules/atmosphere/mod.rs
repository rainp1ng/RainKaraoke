pub mod audio_player;

pub use audio_player::AtmosphereAudioPlayer;

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// 气氛组状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphereState {
    pub playing_sounds: Vec<i64>,
    pub volume: f32,
}

/// 气氛组音效
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphereSoundData {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub volume: f32,
}

/// 气氛组管理器
pub struct AtmosphereManager {
    pub state: Arc<Mutex<AtmosphereState>>,
    audio_player: AtmosphereAudioPlayer,
    app_handle: Option<AppHandle>,
}

impl AtmosphereManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AtmosphereState {
                playing_sounds: Vec::new(),
                volume: 0.8,
            })),
            audio_player: AtmosphereAudioPlayer::new(),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.audio_player.set_app_handle(handle.clone());
        self.app_handle = Some(handle);
    }

    /// 播放音效
    pub fn play_sound(&mut self, sound: &AtmosphereSoundData) -> Result<(), String> {
        let state = self.state.lock().unwrap();
        let master_volume = state.volume;
        drop(state);

        let final_volume = master_volume * sound.volume;

        self.audio_player.play(&sound.file_path, final_volume, sound.id)?;

        self.emit_state_change();

        Ok(())
    }

    /// 停止音效
    pub fn stop_sound(&mut self, sound_id: Option<i64>) -> Result<(), String> {
        self.audio_player.stop(sound_id)
    }

    /// 设置音量
    pub fn set_volume(&mut self, volume: f32) {
        self.state.lock().unwrap().volume = volume;
    }

    /// 获取状态
    pub fn get_state(&self) -> AtmosphereState {
        self.state.lock().unwrap().clone()
    }

    /// 发送状态变化事件
    fn emit_state_change(&self) {
        if let Some(ref handle) = self.app_handle {
            let state = self.state.lock().unwrap().clone();
            let _ = handle.emit("atmosphere:state-changed", state);
        }
    }
}

impl Default for AtmosphereManager {
    fn default() -> Self {
        Self::new()
    }
}
