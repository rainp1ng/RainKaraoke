use std::io::BufReader;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

/// 过场音乐播放器
pub struct InterludeAudioPlayer {
    _stream: Option<OutputStream>,
    _stream_handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
}

impl InterludeAudioPlayer {
    pub fn new() -> Self {
        Self {
            _stream: None,
            _stream_handle: None,
            sink: None,
        }
    }

    fn init_output(&mut self) -> Result<(), String> {
        if self._stream.is_none() {
            let (stream, stream_handle) = OutputStream::try_default()
                .map_err(|e| format!("无法初始化音频输出: {}", e))?;
            self._stream = Some(stream);
            self._stream_handle = Some(stream_handle);
        }
        Ok(())
    }

    pub fn load(&mut self, path: &str) -> Result<(), String> {
        self.init_output()?;

        let sink = Sink::try_new(self._stream_handle.as_ref().unwrap())
            .map_err(|e| format!("无法创建音频播放器: {}", e))?;

        let file = std::fs::File::open(path)
            .map_err(|e| format!("无法打开文件: {}", e))?;

        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("无法解码音频: {}", e))?;

        // 设置循环播放
        sink.append(source.repeat_infinite());

        self.sink = Some(sink);
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), String> {
        if let Some(ref sink) = self.sink {
            sink.play();
        }
        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.pause();
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.play();
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Some(ref sink) = self.sink {
            sink.set_volume(volume);
        }
    }
}

impl Default for InterludeAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
