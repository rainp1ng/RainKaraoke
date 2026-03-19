use crate::modules::effects::{AudioProcessor, EffectType};

/// 动态范围压缩器
pub struct CompressorProcessor {
    pub threshold: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
    pub makeup_gain: f32,

    envelope: f32,
    sample_rate: f32,
}

impl CompressorProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: -24.0,
            ratio: 4.0,
            attack: 10.0,
            release: 100.0,
            makeup_gain: 0.0,
            envelope: 0.0,
            sample_rate,
        }
    }
}

impl AudioProcessor for CompressorProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let attack_coef = (-1.0 / (self.attack * self.sample_rate / 1000.0)).exp();
        let release_coef = (-1.0 / (self.release * self.sample_rate / 1000.0)).exp();

        for (i, sample) in input.iter().enumerate() {
            let abs_sample = sample.abs();

            // 包络跟随
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

            // 计算增益衰减
            let gain_reduction_db = if db_level > self.threshold {
                (self.threshold - db_level) * (1.0 - 1.0 / self.ratio)
            } else {
                0.0
            };

            // 应用增益
            let total_gain_db = gain_reduction_db + self.makeup_gain;
            let gain = 10.0_f32.powf(total_gain_db / 20.0);

            output[i] = *sample * gain;
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "threshold" => self.threshold = value,
            "ratio" => self.ratio = value,
            "attack" => self.attack = value,
            "release" => self.release = value,
            "makeupGain" => self.makeup_gain = value,
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Compressor
    }
}
