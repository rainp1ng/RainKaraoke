use crate::modules::effects::{AudioProcessor, EffectType};

/// 增益效果器
/// 用于调整音量大小
pub struct GainProcessor {
    /// 增益值 (dB)，范围 -24 到 +24
    pub gain_db: f32,
    /// 线性增益系数
    linear_gain: f32,
}

impl GainProcessor {
    pub fn new() -> Self {
        Self {
            gain_db: 0.0,
            linear_gain: 1.0,
        }
    }

    fn update_linear_gain(&mut self) {
        // dB 转线性: linear = 10^(dB/20)
        self.linear_gain = 10f32.powf(self.gain_db / 20.0);
    }
}

impl Default for GainProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioProcessor for GainProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (i, sample) in input.iter().enumerate() {
            output[i] = sample * self.linear_gain;
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "gainDb" | "gain" => {
                self.gain_db = value;
                self.update_linear_gain();
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        // 增益效果器无状态，无需重置
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Gain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_processor() {
        let mut processor = GainProcessor::new();

        // 测试 0dB (无变化)
        processor.set_parameter("gainDb", 0.0);
        let input = [0.5, 0.25, -0.5, -0.25];
        let mut output = [0.0; 4];
        processor.process(&input, &mut output);
        assert!((output[0] - 0.5).abs() < 0.001);

        // 测试 +6dB (约 2x 增益)
        processor.set_parameter("gainDb", 6.0);
        processor.process(&input, &mut output);
        assert!((output[0] - 1.0).abs() < 0.1);

        // 测试 -6dB (约 0.5x 增益)
        processor.set_parameter("gainDb", -6.0);
        processor.process(&input, &mut output);
        assert!((output[0] - 0.25).abs() < 0.05);
    }
}
