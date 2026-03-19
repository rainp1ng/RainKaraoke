use std::sync::{Arc, Mutex};
use webrtc_vad::Vad;

/// VAD 检测器配置
pub struct VadConfig {
    /// 激活阈值 (0.0 - 1.0)
    pub threshold: f32,
    /// 持续激活帧数阈值
    pub activation_frames: u32,
    /// 持续静音帧数阈值
    pub silence_frames: u32,
    /// 采样率
    pub sample_rate: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            activation_frames: 3,
            silence_frames: 10,
            sample_rate: 16000,
        }
    }
}

/// VAD 状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VadState {
    Silence,
    Voice,
}

/// VAD 检测器
pub struct VoiceActivityDetector {
    vad: Vad,
    config: VadConfig,
    state: VadState,
    consecutive_voice_frames: u32,
    consecutive_silence_frames: u32,
    is_voice_active: bool,
}

impl VoiceActivityDetector {
    pub fn new(config: VadConfig) -> Self {
        let mut vad = Vad::new();
        let _ = vad.set_mode(webrtc_vad::VadMode::Aggressive);

        Self {
            vad,
            config,
            state: VadState::Silence,
            consecutive_voice_frames: 0,
            consecutive_silence_frames: 0,
            is_voice_active: false,
        }
    }

    /// 处理音频帧
    pub fn process(&mut self, frame: &[i16]) -> VadState {
        // 使用 WebRTC VAD 检测
        let is_voice = self.vad
            .is_voice_segment(frame)
            .unwrap_or(false);

        if is_voice {
            self.consecutive_voice_frames += 1;
            self.consecutive_silence_frames = 0;

            // 连续检测到语音帧数超过阈值，激活语音状态
            if self.consecutive_voice_frames >= self.config.activation_frames {
                self.state = VadState::Voice;
                self.is_voice_active = true;
            }
        } else {
            self.consecutive_silence_frames += 1;
            self.consecutive_voice_frames = 0;

            // 连续静音帧数超过阈值，切换到静音状态
            if self.consecutive_silence_frames >= self.config.silence_frames {
                self.state = VadState::Silence;
                self.is_voice_active = false;
            }
        }

        self.state
    }

    /// 获取当前状态
    pub fn get_state(&self) -> VadState {
        self.state
    }

    /// 是否处于语音激活状态
    pub fn is_voice_active(&self) -> bool {
        self.is_voice_active
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.state = VadState::Silence;
        self.consecutive_voice_frames = 0;
        self.consecutive_silence_frames = 0;
        self.is_voice_active = false;
    }
}

/// Ducking 控制器
pub struct DuckingController {
    /// Ducking 比例 (0.0 - 1.0)
    pub duck_ratio: f32,
    /// 当前音量
    pub current_volume: f32,
    /// 目标音量
    pub target_volume: f32,
    /// 淡入/淡出速度
    pub fade_speed: f32,
    /// 是否处于 Ducking 状态
    pub is_ducking: bool,
}

impl DuckingController {
    pub fn new(duck_ratio: f32, fade_speed: f32) -> Self {
        Self {
            duck_ratio,
            current_volume: 1.0,
            target_volume: 1.0,
            fade_speed,
            is_ducking: false,
        }
    }

    /// 开始 Ducking
    pub fn start_ducking(&mut self, original_volume: f32) {
        self.target_volume = original_volume * self.duck_ratio;
        self.is_ducking = true;
    }

    /// 停止 Ducking
    pub fn stop_ducking(&mut self, original_volume: f32) {
        self.target_volume = original_volume;
        self.is_ducking = false;
    }

    /// 更新音量（平滑过渡）
    pub fn update(&mut self, delta_time: f32) -> f32 {
        if (self.current_volume - self.target_volume).abs() < 0.01 {
            self.current_volume = self.target_volume;
        } else if self.current_volume < self.target_volume {
            self.current_volume = (self.current_volume + self.fade_speed * delta_time)
                .min(self.target_volume);
        } else {
            self.current_volume = (self.current_volume - self.fade_speed * delta_time)
                .max(self.target_volume);
        }

        self.current_volume
    }
}

impl Default for DuckingController {
    fn default() -> Self {
        Self::new(0.2, 2.0)
    }
}
