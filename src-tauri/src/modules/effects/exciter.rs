use crate::modules::effects::{AudioProcessor, EffectType};

/// 激励器
pub struct ExciterProcessor {
    pub frequency: f32,
    pub harmonics: f32,
    pub mix: f32,

    sample_rate: f32,
}

impl ExciterProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            frequency: 8000.0,
            harmonics: 0.3,
            mix: 0.2,
            sample_rate,
        }
    }
}

impl AudioProcessor for ExciterProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        // 简化的激励器实现
        // 实际应该使用谐波生成器

        for (i, sample) in input.iter().enumerate() {
            // 生成谐波（简化：使用软削波）
            let harmonics = if *sample > 0.0 {
                (*sample * 2.0).min(1.0)
            } else {
                (*sample * 2.0).max(-1.0)
            };

            // 添加到原始信号
            let excited = *sample + harmonics * self.harmonics * 0.1;

            output[i] = *sample * (1.0 - self.mix) + excited * self.mix;
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "frequency" => self.frequency = value,
            "harmonics" => self.harmonics = value / 100.0,
            "mix" => self.mix = value / 100.0,
            _ => {}
        }
    }

    fn reset(&mut self) {}

    fn effect_type(&self) -> EffectType {
        EffectType::Exciter
    }
}
