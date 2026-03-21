use std::path::PathBuf;
use std::fs::File;
use std::io::BufWriter;
use serde::{Deserialize, Serialize};

/// WAV 文件头信息
#[derive(Debug, Clone, Copy)]
struct WavHeader {
    num_channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
}

/// 音频录制器
/// 支持将音频数据录制为 WAV 文件
pub struct AudioRecorder {
    /// 录音保存路径
    path: Option<PathBuf>,
    /// WAV 文件写入器
    writer: Option<BufWriter<File>>,
    /// 是否正在录音
    is_recording: bool,
    /// WAV 头信息
    header: WavHeader,
    /// 已写入的采样数
    samples_written: u32,
    /// 数据块起始位置（用于更新文件大小）
    data_chunk_start: u64,
}

/// 录音状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingState {
    /// 是否正在录音
    pub is_recording: bool,
    /// 录音文件路径
    pub file_path: Option<String>,
    /// 已录制时长（毫秒）
    pub duration_ms: u64,
    /// 采样率
    pub sample_rate: u32,
    /// 声道数
    pub channels: u16,
}

impl AudioRecorder {
    /// 创建新的录音器
    pub fn new() -> Self {
        Self {
            path: None,
            writer: None,
            is_recording: false,
            header: WavHeader {
                num_channels: 2,
                sample_rate: 44100,
                bits_per_sample: 16,
            },
            samples_written: 0,
            data_chunk_start: 0,
        }
    }

    /// 开始录音
    /// # 参数
    /// - `path`: 录音文件保存路径
    /// - `sample_rate`: 采样率
    /// - `channels`: 声道数
    pub fn start_recording(
        &mut self,
        path: PathBuf,
        sample_rate: u32,
        channels: u16,
    ) -> Result<(), String> {
        if self.is_recording {
            return Err("Already recording".to_string());
        }

        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // 创建文件
        let file = File::create(&path)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        let mut writer = BufWriter::new(file);

        // 写入 WAV 头（稍后会更新大小信息）
        self.header = WavHeader {
            num_channels: channels,
            sample_rate,
            bits_per_sample: 16,
        };

        Self::write_wav_header(&mut writer, &self.header, 0)
            .map_err(|e| format!("Failed to write WAV header: {}", e))?;

        // 记录数据块起始位置
        self.data_chunk_start = 44; // 标准 WAV 头大小

        self.path = Some(path);
        self.writer = Some(writer);
        self.is_recording = true;
        self.samples_written = 0;

        println!("[Recorder] Started recording to {:?}", self.path);
        Ok(())
    }

    /// 停止录音
    pub fn stop_recording(&mut self) -> Result<Option<PathBuf>, String> {
        if !self.is_recording {
            return Ok(None);
        }

        let path = self.path.take();

        if let Some(mut writer) = self.writer.take() {
            // 更新 WAV 文件头中的大小信息
            self.update_wav_sizes(&mut writer)
                .map_err(|e| format!("Failed to update WAV sizes: {}", e))?;
        }

        self.is_recording = false;
        println!("[Recorder] Stopped recording, {} samples written", self.samples_written);

        Ok(path)
    }

    /// 写入音频采样数据
    /// 输入为 f32 格式的音频数据，范围 [-1.0, 1.0]
    pub fn write_samples(&mut self, samples: &[f32]) -> Result<(), String> {
        if !self.is_recording {
            return Ok(());
        }

        let writer = self.writer.as_mut().ok_or("No writer available")?;

        // 将 f32 转换为 16-bit PCM
        for sample in samples {
            let sample_clamped = sample.clamp(-1.0, 1.0);
            let sample_i16 = (sample_clamped * 32767.0) as i16;
            let bytes = sample_i16.to_le_bytes();

            use std::io::Write;
            writer.write_all(&bytes)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        self.samples_written += samples.len() as u32;
        Ok(())
    }

    /// 写入立体声采样数据（交错格式）
    pub fn write_interleaved_samples(&mut self, left: &[f32], right: &[f32]) -> Result<(), String> {
        if !self.is_recording {
            return Ok(());
        }

        if left.len() != right.len() {
            return Err("Left and right channel lengths don't match".to_string());
        }

        let writer = self.writer.as_mut().ok_or("No writer available")?;

        use std::io::Write;

        // 交错写入左右声道
        for (l, r) in left.iter().zip(right.iter()) {
            let l_clamped = l.clamp(-1.0, 1.0);
            let r_clamped = r.clamp(-1.0, 1.0);

            let l_i16 = (l_clamped * 32767.0) as i16;
            let r_i16 = (r_clamped * 32767.0) as i16;

            writer.write_all(&l_i16.to_le_bytes())
                .map_err(|e| format!("Failed to write sample: {}", e))?;
            writer.write_all(&r_i16.to_le_bytes())
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        self.samples_written += (left.len() * 2) as u32;
        Ok(())
    }

    /// 获取录音状态
    pub fn get_state(&self) -> RecordingState {
        let duration_ms = if self.header.sample_rate > 0 {
            let total_samples = self.samples_written / self.header.num_channels as u32;
            (total_samples as u64 * 1000) / self.header.sample_rate as u64
        } else {
            0
        };

        RecordingState {
            is_recording: self.is_recording,
            file_path: self.path.as_ref().map(|p| p.to_string_lossy().to_string()),
            duration_ms,
            sample_rate: self.header.sample_rate,
            channels: self.header.num_channels,
        }
    }

    /// 是否正在录音
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// 写入 WAV 文件头
    fn write_wav_header(
        writer: &mut BufWriter<File>,
        header: &WavHeader,
        data_size: u32,
    ) -> Result<(), std::io::Error> {
        use std::io::Write;

        let byte_rate = header.sample_rate * header.num_channels as u32 * header.bits_per_sample as u32 / 8;
        let block_align = header.num_channels * header.bits_per_sample / 8;

        // RIFF header
        writer.write_all(b"RIFF")?;
        writer.write_all(&(36 + data_size).to_le_bytes())?; // File size - 8
        writer.write_all(b"WAVE")?;

        // fmt chunk
        writer.write_all(b"fmt ")?;
        writer.write_all(&16u32.to_le_bytes())?; // Chunk size
        writer.write_all(&1u16.to_le_bytes())?; // Audio format (1 = PCM)
        writer.write_all(&header.num_channels.to_le_bytes())?;
        writer.write_all(&header.sample_rate.to_le_bytes())?;
        writer.write_all(&byte_rate.to_le_bytes())?;
        writer.write_all(&block_align.to_le_bytes())?;
        writer.write_all(&header.bits_per_sample.to_le_bytes())?;

        // data chunk
        writer.write_all(b"data")?;
        writer.write_all(&data_size.to_le_bytes())?;

        Ok(())
    }

    /// 更新 WAV 文件大小信息
    fn update_wav_sizes(&self, writer: &mut BufWriter<File>) -> Result<(), std::io::Error> {
        use std::io::{Seek, SeekFrom, Write};

        let data_size = self.samples_written * self.header.bits_per_sample as u32 / 8;
        let file_size = 36 + data_size;

        // 更新文件大小
        writer.seek(SeekFrom::Start(4))?;
        writer.write_all(&file_size.to_le_bytes())?;

        // 更新数据块大小
        writer.seek(SeekFrom::Start(40))?;
        writer.write_all(&data_size.to_le_bytes())?;

        writer.flush()?;
        Ok(())
    }
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// 双轨录音器
/// 支持同时录制人声和乐器两个轨道
pub struct DualTrackRecorder {
    /// 人声录音器
    vocal_recorder: AudioRecorder,
    /// 乐器录音器
    instrument_recorder: AudioRecorder,
    /// 采样率
    sample_rate: u32,
    /// 声道数
    channels: u16,
}

impl DualTrackRecorder {
    /// 创建新的双轨录音器
    pub fn new() -> Self {
        Self {
            vocal_recorder: AudioRecorder::new(),
            instrument_recorder: AudioRecorder::new(),
            sample_rate: 44100,
            channels: 2,
        }
    }

    /// 开始录音
    pub fn start_recording(
        &mut self,
        vocal_path: Option<PathBuf>,
        instrument_path: Option<PathBuf>,
        sample_rate: u32,
        channels: u16,
    ) -> Result<(), String> {
        self.sample_rate = sample_rate;
        self.channels = channels;

        if let Some(path) = vocal_path {
            self.vocal_recorder.start_recording(path, sample_rate, channels)?;
        }

        if let Some(path) = instrument_path {
            self.instrument_recorder.start_recording(path, sample_rate, channels)?;
        }

        Ok(())
    }

    /// 停止录音
    pub fn stop_recording(&mut self) -> Result<(Option<PathBuf>, Option<PathBuf>), String> {
        let vocal_path = self.vocal_recorder.stop_recording()?;
        let instrument_path = self.instrument_recorder.stop_recording()?;
        Ok((vocal_path, instrument_path))
    }

    /// 写入人声采样
    pub fn write_vocal_samples(&mut self, samples: &[f32]) -> Result<(), String> {
        self.vocal_recorder.write_samples(samples)
    }

    /// 写入乐器采样
    pub fn write_instrument_samples(&mut self, samples: &[f32]) -> Result<(), String> {
        self.instrument_recorder.write_samples(samples)
    }

    /// 写入人声立体声采样
    pub fn write_vocal_interleaved(&mut self, left: &[f32], right: &[f32]) -> Result<(), String> {
        self.vocal_recorder.write_interleaved_samples(left, right)
    }

    /// 写入乐器立体声采样
    pub fn write_instrument_interleaved(&mut self, left: &[f32], right: &[f32]) -> Result<(), String> {
        self.instrument_recorder.write_interleaved_samples(left, right)
    }

    /// 是否正在录音
    pub fn is_recording(&self) -> bool {
        self.vocal_recorder.is_recording() || self.instrument_recorder.is_recording()
    }

    /// 获取人声录音状态
    pub fn get_vocal_state(&self) -> RecordingState {
        self.vocal_recorder.get_state()
    }

    /// 获取乐器录音状态
    pub fn get_instrument_state(&self) -> RecordingState {
        self.instrument_recorder.get_state()
    }
}

impl Default for DualTrackRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_audio_recorder_start_stop() {
        let mut recorder = AudioRecorder::new();

        // 开始录音
        let result = recorder.start_recording(
            PathBuf::from("/tmp/test_recording.wav"),
            44100,
            2,
        );
        assert!(result.is_ok());
        assert!(recorder.is_recording());

        // 写入一些采样
        let samples = vec![0.5, 0.3, -0.2, -0.4];
        let result = recorder.write_samples(&samples);
        assert!(result.is_ok());

        // 停止录音
        let result = recorder.stop_recording();
        assert!(result.is_ok());
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_recording_state() {
        let mut recorder = AudioRecorder::new();

        let state = recorder.get_state();
        assert!(!state.is_recording);
        assert!(state.file_path.is_none());

        recorder.start_recording(
            PathBuf::from("/tmp/test_state.wav"),
            48000,
            1,
        ).unwrap();

        let state = recorder.get_state();
        assert!(state.is_recording);
        assert!(state.file_path.is_some());
        assert_eq!(state.sample_rate, 48000);
        assert_eq!(state.channels, 1);

        recorder.stop_recording().unwrap();
    }
}
