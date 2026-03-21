use std::sync::{Arc, Mutex, mpsc::{self, Sender}};
use tauri::{AppHandle, Emitter};

pub mod audio_player;
pub mod playback_state;

pub use audio_player::AudioPlayer;
pub use playback_state::{PlaybackState, PlaybackStatus};

/// 播放命令
#[derive(Debug)]
pub enum PlayerCommand {
    Play { song_id: i64, video_path: Option<String>, audio_path: Option<String> },
    Pause,
    Resume,
    Stop,
    Seek(f64),
    ToggleVocal(bool),
    SetPitch(i32),
    SetSpeed(f64),
    SetVolume(f64),
}

/// 媒体引擎 - 只管理播放状态，不实际播放音频
/// 实际的媒体播放由前端的 VideoPlayer 组件负责
pub struct MediaEngine {
    state: Arc<Mutex<PlaybackState>>,
    command_tx: Sender<PlayerCommand>,
    app_handle: Option<AppHandle>,
}

impl MediaEngine {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(PlaybackState::default()));
        let state_clone = state.clone();
        let (command_tx, command_rx) = mpsc::channel::<PlayerCommand>();

        // 启动状态管理线程
        std::thread::spawn(move || {
            let state = state_clone;

            while let Ok(cmd) = command_rx.recv() {
                match cmd {
                    PlayerCommand::Play { song_id, video_path, audio_path, .. } => {
                        println!("[MediaEngine] 收到播放命令, song_id={}", song_id);
                        let mut s = match state.lock() {
                            Ok(guard) => guard,
                            Err(e) => {
                                eprintln!("[MediaEngine] Mutex 被污染，恢复中...");
                                e.into_inner()
                            }
                        };

                        // 只更新状态，不播放音频（前端负责播放）
                        s.current_song_id = Some(song_id);
                        s.current_video_path = video_path.clone();
                        s.current_audio_path = audio_path.clone();
                        s.status = PlaybackStatus::Playing;
                        s.current_time = 0.0;
                        println!("[MediaEngine] 状态已更新为 Playing");
                    }
                    PlayerCommand::Pause => {
                        let mut s = state.lock().unwrap();
                        s.status = PlaybackStatus::Paused;
                    }
                    PlayerCommand::Resume => {
                        let mut s = state.lock().unwrap();
                        s.status = PlaybackStatus::Playing;
                    }
                    PlayerCommand::Stop => {
                        let mut s = state.lock().unwrap();
                        s.status = PlaybackStatus::Idle;
                        s.current_song_id = None;
                        s.current_time = 0.0;
                        s.duration = 0.0;
                    }
                    PlayerCommand::Seek(time) => {
                        let mut s = state.lock().unwrap();
                        s.current_time = time;
                    }
                    PlayerCommand::ToggleVocal(is_vocal) => {
                        let mut s = state.lock().unwrap();
                        s.is_vocal = is_vocal;
                    }
                    PlayerCommand::SetPitch(semitones) => {
                        let mut s = state.lock().unwrap();
                        s.pitch = semitones;
                    }
                    PlayerCommand::SetSpeed(speed) => {
                        let mut s = state.lock().unwrap();
                        s.speed = speed;
                    }
                    PlayerCommand::SetVolume(volume) => {
                        let mut s = state.lock().unwrap();
                        s.volume = volume.clamp(0.0, 1.0);
                    }
                }
            }
        });

        Self {
            state,
            command_tx,
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    /// 播放歌曲
    pub fn play(&mut self, song_id: i64, video_path: Option<String>, audio_path: Option<String>) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::Play { song_id, video_path, audio_path })
            .map_err(|e| format!("发送播放命令失败: {}", e))?;

        self.emit_state_change();
        Ok(())
    }

    /// 暂停播放
    pub fn pause(&mut self) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::Pause)
            .map_err(|e| format!("发送暂停命令失败: {}", e))?;

        self.emit_state_change();
        Ok(())
    }

    /// 继续播放
    pub fn resume(&mut self) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::Resume)
            .map_err(|e| format!("发送继续播放命令失败: {}", e))?;

        self.emit_state_change();
        Ok(())
    }

    /// 停止播放
    pub fn stop(&mut self) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::Stop)
            .map_err(|e| format!("发送停止命令失败: {}", e))?;

        self.emit_state_change();
        Ok(())
    }

    /// 跳转到指定时间
    pub fn seek(&mut self, time: f64) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::Seek(time))
            .map_err(|e| format!("发送跳转命令失败: {}", e))?;
        Ok(())
    }

    /// 切换原唱/伴唱
    pub fn toggle_vocal(&mut self, is_vocal: bool) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::ToggleVocal(is_vocal))
            .map_err(|e| format!("发送切换命令失败: {}", e))?;

        self.emit_state_change();
        Ok(())
    }

    /// 设置音调
    pub fn set_pitch(&mut self, semitones: i32) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::SetPitch(semitones))
            .map_err(|e| format!("发送音调命令失败: {}", e))?;
        Ok(())
    }

    /// 设置播放速度
    pub fn set_speed(&mut self, speed: f64) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::SetSpeed(speed))
            .map_err(|e| format!("发送速度命令失败: {}", e))?;
        Ok(())
    }

    /// 设置音量
    pub fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::SetVolume(volume))
            .map_err(|e| format!("发送音量命令失败: {}", e))?;
        Ok(())
    }

    /// 获取当前状态
    pub fn get_state(&self) -> PlaybackState {
        match self.state.lock() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                eprintln!("[MediaEngine] get_state: Mutex 被污染，恢复中...");
                e.into_inner().clone()
            }
        }
    }

    /// 更新播放时间（由前端定时调用）
    pub fn update_time(&mut self, time: f64) {
        let mut state = self.state.lock().unwrap();
        state.current_time = time;
    }

    /// 发送状态变化事件
    fn emit_state_change(&self) {
        if let Some(ref handle) = self.app_handle {
            let state = match self.state.lock() {
                Ok(guard) => guard.clone(),
                Err(e) => {
                    eprintln!("[MediaEngine] emit_state_change: Mutex 被污染，恢复中...");
                    e.into_inner().clone()
                }
            };
            let _ = handle.emit("playback:state-changed", state);
        }
    }
}

impl Default for MediaEngine {
    fn default() -> Self {
        Self::new()
    }
}

// 确保可以跨线程发送
unsafe impl Send for MediaEngine {}
unsafe impl Sync for MediaEngine {}
