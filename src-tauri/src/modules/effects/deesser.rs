use crate::modules::effects::{AudioProcessor, EffectType};

/// 去齿音效果器
pub struct DeEsserProcessor {
    pub frequency: f32,
    pub threshold: f32,
    pub range: f32,

    envelope: f32,
    sample_rate: f32,
}

impl DeEsserProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            frequency: 6000.0,
            threshold: -20.0,
            range: 6.0,
            envelope: 0.0,
            sample_rate,
        }
    }
}

impl AudioProcessor for DeEsserProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        // 简化的去齿音实现
        // 实际实现需要带通滤波器来分离高频成分
        let high_freq_ratio = self.frequency / self.sample_rate;
        let attack_coef = (-1.0 / (0.001 * self.sample_rate)).exp();
        let release_coef = (-1.0 / (0.05 * self.sample_rate)).exp();

        for (i, sample) in input.iter().enumerate() {
            // 检测高频能量（简化）
            let high_freq = *sample; // 实际应该经过高通滤波

            let abs_sample = high_freq.abs();
            if abs_sample > self.envelope {
                self.envelope = attack_coef * self.envelope + (1.0 - attack_coef) * abs_sample;
            } else {
                self.envelope = release_coef * self.envelope + (1.0 - release_coef) * abs_sample;
            }

            // 转换为 dB
            let db_level = if self.envelope > 0.0 {
                20.0 * self.envelope.log10()
            } else {
                -60.0
            };

            // 计算衰减
            let gain_reduction = if db_level > self.threshold {
                let reduction_db = (db_level - self.threshold) * 0.5;
                10.0_f32.powf(-reduction_db.min(self.range) / 20.0)
            } else {
                1.0
            };

            output[i] = *sample * gain_reduction;
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "frequency" => self.frequency = value,
            "threshold" => self.threshold = value,
            "range" => self.range = value,
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn effect_type(&self) -> EffectType {
        EffectType::DeEsser
    }
}
