use std::io::BufReader;
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use rodio::{Decoder, Sink, Source, OutputStream};

/// 全局停止信号列表
static STOP_SIGNALS: Mutex<Vec<(i64, Sender<()>)>> = Mutex::new(Vec::new());

/// 气氛组音频播放器
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

    /// 播放音效
    pub fn play(&mut self, path: &str, volume: f32, sound_id: i64) -> Result<(), String> {
        let app_handle = self.app_handle.clone();
        let path = path.to_string(); // 拷贝字符串

        // 创建停止通道
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        // 添加到全局列表
        {
            let mut signals = STOP_SIGNALS.lock().unwrap();
            signals.push((sound_id, stop_tx));
        }

        std::thread::spawn(move || {
            // 在线程内创建 OutputStream
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[气氛组] 无法初始化音频输出: {}", e);
                    // 移除信号
                    let mut signals = STOP_SIGNALS.lock().unwrap();
                    signals.retain(|(sid, _)| *sid != sound_id);
                    return;
                }
            };

            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[气氛组] 无法创建音频播放器: {}", e);
                    let mut signals = STOP_SIGNALS.lock().unwrap();
                    signals.retain(|(sid, _)| *sid != sound_id);
                    return;
                }
            };

            let file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("[气氛组] 无法打开文件 '{}': {}", path, e);
                    let mut signals = STOP_SIGNALS.lock().unwrap();
                    signals.retain(|(sid, _)| *sid != sound_id);
                    return;
                }
            };

            let source = match Decoder::new(BufReader::new(file)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[气氛组] 无法解码音频 '{}': {}", path, e);
                    let mut signals = STOP_SIGNALS.lock().unwrap();
                    signals.retain(|(sid, _)| *sid != sound_id);
                    return;
                }
            };

            sink.set_volume(volume);
            sink.append(source);

            // 轮询检查播放状态和停止信号
            loop {
                if sink.empty() {
                    break;
                }
                // 非阻塞检查停止信号
                match stop_rx.try_recv() {
                    Ok(()) => {
                        sink.stop();
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {
                        sink.stop();
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            // 保持 stream 存活直到播放结束
            drop(_stream);

            // 移除信号
            {
                let mut signals = STOP_SIGNALS.lock().unwrap();
                signals.retain(|(sid, _)| *sid != sound_id);
            }

            // 播放完成，发送事件
            if let Some(handle) = app_handle {
                let _ = handle.emit("atmosphere:sound-ended", sound_id);
            }
        });

        Ok(())
    }

    /// 停止指定音效
    pub fn stop(&mut self, sound_id: Option<i64>) -> Result<(), String> {
        let mut signals = STOP_SIGNALS.lock().unwrap();

        match sound_id {
            Some(id) => {
                // 停止指定音效
                if let Some(pos) = signals.iter().position(|(sid, _)| *sid == id) {
                    let (_, tx) = signals.remove(pos);
                    let _ = tx.send(());
                }
            }
            None => {
                // 停止所有
                for (_, tx) in signals.drain(..) {
                    let _ = tx.send(());
                }
            }
        }
        Ok(())
    }

    /// 停止所有音效
    pub fn stop_all(&mut self) {
        let mut signals = STOP_SIGNALS.lock().unwrap();
        for (_, tx) in signals.drain(..) {
            let _ = tx.send(());
        }
    }
}

impl Default for AtmosphereAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
