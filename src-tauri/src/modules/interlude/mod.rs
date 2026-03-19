use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

pub mod audio_player;

pub use audio_player::InterludeAudioPlayer;

/// 过场音乐状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterludeState {
    pub is_playing: bool,
    pub current_track_id: Option<i64>,
    pub current_track_title: Option<String>,
    pub volume: f32,
    pub ducking_active: bool,
}

/// 过场音乐管理器
pub struct InterludeManager {
    pub state: Arc<Mutex<InterludeState>>,
    pub audio_player: InterludeAudioPlayer,
    tracks: Arc<Mutex<Vec<InterludeTrack>>>,
    app_handle: Option<AppHandle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterludeTrack {
    pub id: i64,
    pub title: Option<String>,
    pub file_path: String,
    pub volume: f32,
}

impl InterludeManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(InterludeState {
                is_playing: false,
                current_track_id: None,
                current_track_title: None,
                volume: 0.3,
                ducking_active: false,
            })),
            audio_player: InterludeAudioPlayer::new(),
            tracks: Arc::new(Mutex::new(Vec::new())),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    /// 设置过场音乐列表
    pub fn set_tracks(&mut self, tracks: Vec<InterludeTrack>) {
        *self.tracks.lock().unwrap() = tracks;
    }

    /// 开始播放（随机选择）
    pub fn start_random(&mut self) -> Result<(), String> {
        let tracks = self.tracks.lock().unwrap();
        if tracks.is_empty() {
            return Err("没有可用的过场音乐".to_string());
        }

        let track = tracks
            .choose(&mut rand::thread_rng())
            .ok_or("无法选择过场音乐")?
            .clone();

        drop(tracks);

        self.play_track(&track)
    }

    /// 播放指定曲目
    pub fn play_track(&mut self, track: &InterludeTrack) -> Result<(), String> {
        // 更新状态
        let mut state = self.state.lock().unwrap();
        state.is_playing = true;
        state.current_track_id = Some(track.id);
        state.current_track_title = track.title.clone();

        let volume = state.volume;
        drop(state);

        // 播放音频
        self.audio_player.load(&track.file_path)?;
        self.audio_player.set_volume(volume);
        self.audio_player.play()?;

        self.emit_state_change();

        Ok(())
    }

    /// 暂停播放
    pub fn pause(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.is_playing = false;

        self.audio_player.pause();

        self.emit_state_change();

        Ok(())
    }

    /// 继续播放
    pub fn resume(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.is_playing = true;

        self.audio_player.resume();

        self.emit_state_change();

        Ok(())
    }

    /// 停止播放
    pub fn stop(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.is_playing = false;
        state.current_track_id = None;
        state.current_track_title = None;

        self.audio_player.stop();

        self.emit_state_change();

        Ok(())
    }

    /// 设置音量
    pub fn set_volume(&mut self, volume: f32) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.volume = volume;

        self.audio_player.set_volume(volume);

        Ok(())
    }

    /// 应用 Ducking（降低音量）
    pub fn apply_ducking(&mut self, ducking_ratio: f32) {
        let state = self.state.lock().unwrap();
        let original_volume = state.volume;
        drop(state);

        let ducked_volume = original_volume * ducking_ratio;
        self.audio_player.set_volume(ducked_volume);

        let mut state = self.state.lock().unwrap();
        state.ducking_active = true;
        drop(state);

        self.emit_state_change();
    }

    /// 恢复音量（取消 Ducking）
    pub fn release_ducking(&mut self) {
        let state = self.state.lock().unwrap();
        let original_volume = state.volume;
        drop(state);

        self.audio_player.set_volume(original_volume);

        let mut state = self.state.lock().unwrap();
        state.ducking_active = false;
        drop(state);

        self.emit_state_change();
    }

    /// 获取当前状态
    pub fn get_state(&self) -> InterludeState {
        self.state.lock().unwrap().clone()
    }

    /// 发送状态变化事件
    fn emit_state_change(&self) {
        if let Some(ref handle) = self.app_handle {
            let state = self.state.lock().unwrap().clone();
            let _ = handle.emit("interlude:state-changed", state);
        }
    }
}

impl Default for InterludeManager {
    fn default() -> Self {
        Self::new()
    }
}
