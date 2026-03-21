use crate::modules::effects::{AudioProcessor, EffectType};
use std::sync::atomic::{AtomicU32, Ordering};

/// 将 f32 转换为 u32 存储
fn f32_to_u32(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * (u32::MAX as f32)) as u32
}

/// 将 u32 转换为 f32
fn u32_to_f32(v: u32) -> f32 {
    v as f32 / u32::MAX as f32
}

/// 电平表处理器
/// 测量音频信号的峰值电平，用于 UI 显示
pub struct LevelMeterProcessor {
    /// 当前峰值电平 (存储为 u32)
    peak_level: AtomicU32,
    /// 峰值保持时间计数器（帧数，不是采样数）
    peak_hold_frames: u32,
    /// 峰值保持时间（帧数，约 100ms @ 30fps）
    peak_hold_time_frames: u32,
    /// 衰减系数（每帧衰减）
    /// 0.9 表示每帧衰减 10%，在 30fps 下约 0.3s 从满衰减到 0
    decay_per_frame: f32,
}

impl LevelMeterProcessor {
    pub fn new(_sample_rate: f32) -> Self {
        Self {
            peak_level: AtomicU32::new(0),
            peak_hold_frames: 0,
            peak_hold_time_frames: 3, // 约保持 3 帧 (100ms @ 30fps)
            decay_per_frame: 0.85,    // 每帧衰减 15%
        }
    }

    /// 获取当前电平 (0.0 - 1.0)
    pub fn get_level(&self) -> f32 {
        u32_to_f32(self.peak_level.load(Ordering::Relaxed))
    }

    /// 获取当前电平 (dB)
    pub fn get_level_db(&self) -> f32 {
        let level = self.get_level();
        if level > 0.0 {
            20.0 * level.log10()
        } else {
            -60.0
        }
    }

    /// 重置电平
    pub fn reset_level(&self) {
        self.peak_level.store(0, Ordering::Relaxed);
    }
}

impl AudioProcessor for LevelMeterProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        // 直接透传
        output.copy_from_slice(input);

        // 计算当前帧的峰值
        let mut max_sample = 0.0f32;
        for &sample in input.iter() {
            let abs_sample = sample.abs();
            if abs_sample > max_sample {
                max_sample = abs_sample;
            }
        }

        // 获取当前峰值
        let current_peak = u32_to_f32(self.peak_level.load(Ordering::Relaxed));

        if max_sample > current_peak {
            // 新峰值更高，直接更新
            self.peak_level.store(f32_to_u32(max_sample), Ordering::Relaxed);
            self.peak_hold_frames = 0;
        } else {
            // 峰值保持和衰减
            self.peak_hold_frames += 1;
            if self.peak_hold_frames > self.peak_hold_time_frames {
                // 衰减 - 每帧按比例衰减
                let decayed = current_peak * self.decay_per_frame;
                self.peak_level.store(f32_to_u32(decayed), Ordering::Relaxed);
            }
        }
    }

    fn set_parameter(&mut self, _name: &str, _value: f32) {
        // 无参数
    }

    fn reset(&mut self) {
        self.peak_level.store(0, Ordering::Relaxed);
        self.peak_hold_frames = 0;
    }

    fn effect_type(&self) -> EffectType {
        EffectType::LevelMeter
    }

    fn get_level(&self) -> Option<f32> {
        Some(u32_to_f32(self.peak_level.load(Ordering::Relaxed)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_meter() {
        let mut meter = LevelMeterProcessor::new(44100.0);

        let input = vec![0.5; 256];
        let mut output = vec![0.0; 256];

        meter.process(&input, &mut output);

        assert!((meter.get_level() - 0.5).abs() < 0.01);
        assert!((output[0] - 0.5).abs() < 0.01);
    }
}
