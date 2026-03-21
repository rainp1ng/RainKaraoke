use std::io::BufReader;
use std::sync::{Arc, Mutex};
use rodio::{Decoder, OutputStream, Sink, Source};

/// 过场音乐播放器
#[derive(Clone)]
pub struct InterludeAudioPlayer {
    // 使用 Arc<Mutex> 来跨线程共享
    sink: Arc<Mutex<Option<Sink>>>,
    _stream: Arc<Mutex<Option<OutputStream>>>,
}

impl InterludeAudioPlayer {
    pub fn new() -> Self {
        Self {
            sink: Arc::new(Mutex::new(None)),
            _stream: Arc::new(Mutex::new(None)),
        }
    }

    pub fn load(&mut self, path: &str) -> Result<(), String> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("无法初始化音频输出: {}", e))?;

        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("无法创建音频播放器: {}", e))?;

        let file = std::fs::File::open(path)
            .map_err(|e| format!("无法打开文件: {}", e))?;

        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("无法解码音频: {}", e))?;

        // 设置循环播放
        sink.append(source.repeat_infinite());

        *self.sink.lock().unwrap() = Some(sink);
        *self._stream.lock().unwrap() = Some(stream);
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), String> {
        if let Some(ref sink) = *self.sink.lock().unwrap() {
            sink.play();
        }
        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(ref sink) = *self.sink.lock().unwrap() {
            sink.pause();
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref sink) = *self.sink.lock().unwrap() {
            sink.play();
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.lock().unwrap().take() {
            sink.stop();
        }
        *self._stream.lock().unwrap() = None;
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Some(ref sink) = *self.sink.lock().unwrap() {
            sink.set_volume(volume);
        }
    }
}

// 手动实现 Send，因为我们使用 Arc<Mutex> 来包装不可 Send 的类型
unsafe impl Send for InterludeAudioPlayer {}

impl Default for InterludeAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
