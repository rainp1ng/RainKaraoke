use std::io::BufReader;
use std::sync::Arc;
use rodio::{Decoder, OutputStream, Sink};
use tauri::{AppHandle, Emitter};

/// 气氛组音频播放器（每次播放创建独立输出流）
pub struct AtmosphereAudioPlayer {
    app_handle: Option<AppHandle>,
}

impl AtmosphereAudioPlayer {
    pub fn new() -> Self {
        Self { app_handle: None }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    /// 播放音效（在新线程中播放）
    pub fn play(&mut self, path: &str, volume: f32, sound_id: i64) -> Result<(), String> {
        let path = path.to_string();
        let app_handle = self.app_handle.clone();

        std::thread::spawn(move || {
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("无法初始化音频输出: {}", e);
                    return;
                }
            };

            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("无法创建音频播放器: {}", e);
                    return;
                }
            };

            let file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("无法打开文件 '{}': {}", path, e);
                    return;
                }
            };

            let source = match Decoder::new(BufReader::new(file)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("无法解码音频 '{}': {}", path, e);
                    return;
                }
            };

            sink.set_volume(volume);
            sink.append(source);

            // 等待播放完成
            sink.sleep_until_end();

            // 保持 _stream 存活直到播放完成
            drop(_stream);

            // 播放完成，发送事件
            if let Some(handle) = app_handle {
                let _ = handle.emit("atmosphere:sound-ended", sound_id);
            }
        });

        Ok(())
    }

    /// 停止指定音效（由于每次播放独立，无法停止单个）
    pub fn stop(&mut self, _sound_id: i64) -> Result<(), String> {
        // 由于每次播放独立，暂不支持停止单个
        Ok(())
    }

    /// 停止所有音效
    pub fn stop_all(&mut self) {
        // 由于每次播放独立，暂不支持停止所有
    }
}

impl Default for AtmosphereAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
