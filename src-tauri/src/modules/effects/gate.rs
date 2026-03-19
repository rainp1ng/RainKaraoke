use crate::modules::effects::{AudioProcessor, EffectType};

/// 噪声门
pub struct GateProcessor {
    pub threshold: f32,
    pub attack: f32,
    pub release: f32,
    pub range: f32,

    envelope: f32,
    gain: f32,
    sample_rate: f32,
}

impl GateProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: -50.0,
            attack: 0.1,
            release: 50.0,
            range: 40.0,
            envelope: 0.0,
            gain: 1.0,
            sample_rate,
        }
    }
}

impl AudioProcessor for GateProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let attack_coef = (-1.0 / (self.attack * self.sample_rate / 1000.0)).exp();
        let release_coef = (-1.0 / (self.release * self.sample_rate / 1000.0)).exp();

        let min_gain = 10.0_f32.powf(-self.range / 20.0);

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
                -100.0
            };

            // 确定目标增益
            let target_gain = if db_level > self.threshold {
                1.0
            } else {
                min_gain
            };

            // 平滑增益变化
            self.gain = self.gain * 0.99 + target_gain * 0.01;

            output[i] = *sample * self.gain;
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "threshold" => self.threshold = value,
            "attack" => self.attack = value,
            "release" => self.release = value,
            "range" => self.range = value,
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
        self.gain = 1.0;
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Gate
    }
}
