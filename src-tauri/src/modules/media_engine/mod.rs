use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

pub mod audio_player;
pub mod playback_state;

pub use audio_player::AudioPlayer;
pub use playback_state::{PlaybackState, PlaybackStatus};

/// 初始化媒体引擎
pub fn init(_app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// 全局播放引擎实例
pub struct MediaEngine {
    pub state: Arc<Mutex<PlaybackState>>,
    pub audio_player: AudioPlayer,
    app_handle: Option<AppHandle>,
}

impl MediaEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(PlaybackState::default())),
            audio_player: AudioPlayer::new(),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    /// 播放歌曲
    pub fn play(&mut self, song_id: i64, video_path: Option<String>, audio_path: Option<String>) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();

        // 停止当前播放
        self.audio_player.stop();

        // 设置新歌曲
        state.current_song_id = Some(song_id);
        state.current_video_path = video_path.clone();
        state.current_audio_path = audio_path.clone();
        state.status = PlaybackStatus::Playing;
        state.current_time = 0.0;

        // 如果有音频文件，播放音频
        if let Some(path) = &audio_path {
            self.audio_player.load(path)?;
            self.audio_player.play()?;
        }

        // 发送事件
        self.emit_state_change();

        Ok(())
    }

    /// 暂停播放
    pub fn pause(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.status = PlaybackStatus::Paused;

        self.audio_player.pause();

        self.emit_state_change();

        Ok(())
    }

    /// 继续播放
    pub fn resume(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.status = PlaybackStatus::Playing;

        self.audio_player.resume();

        self.emit_state_change();

        Ok(())
    }

    /// 停止播放
    pub fn stop(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.status = PlaybackStatus::Idle;
        state.current_song_id = None;
        state.current_time = 0.0;
        state.duration = 0.0;

        self.audio_player.stop();

        self.emit_state_change();

        Ok(())
    }

    /// 跳转到指定时间
    pub fn seek(&mut self, time: f64) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.current_time = time;

        self.audio_player.seek(time);

        Ok(())
    }

    /// 切换原唱/伴唱
    pub fn toggle_vocal(&mut self, is_vocal: bool) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.is_vocal = is_vocal;

        // TODO: 切换音频文件

        self.emit_state_change();

        Ok(())
    }

    /// 设置音调
    pub fn set_pitch(&mut self, semitones: i32) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.pitch = semitones;

        self.audio_player.set_pitch(semitones);

        Ok(())
    }

    /// 设置播放速度
    pub fn set_speed(&mut self, speed: f64) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.speed = speed;

        self.audio_player.set_speed(speed);

        Ok(())
    }

    /// 获取当前状态
    pub fn get_state(&self) -> PlaybackState {
        self.state.lock().unwrap().clone()
    }

    /// 更新播放时间（由前端定时调用）
    pub fn update_time(&mut self, time: f64) {
        let mut state = self.state.lock().unwrap();
        state.current_time = time;
    }

    /// 发送状态变化事件
    fn emit_state_change(&self) {
        if let Some(ref handle) = self.app_handle {
            let state = self.state.lock().unwrap().clone();
            let _ = handle.emit("playback:state-changed", state);
        }
    }
}

impl Default for MediaEngine {
    fn default() -> Self {
        Self::new()
    }
}
