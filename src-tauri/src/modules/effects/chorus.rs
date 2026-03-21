use crate::modules::effects::{AudioProcessor, EffectType};

/// 合唱效果器
pub struct ChorusProcessor {
    pub rate: f32,
    pub depth: f32,
    pub mix: f32,
    pub voices: usize,
    pub spread: f32,

    delay_buffer: Vec<f32>,
    delay_pos: usize,
    phase: f32,
    sample_rate: f32,
}

impl ChorusProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            rate: 1.5,
            depth: 0.002,
            mix: 0.3,
            voices: 4,
            spread: 0.5,
            delay_buffer: vec![0.0; (sample_rate * 0.05) as usize], // 50ms max delay
            delay_pos: 0,
            phase: 0.0,
            sample_rate,
        }
    }
}

impl AudioProcessor for ChorusProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let phase_inc = std::f32::consts::TAU * self.rate / self.sample_rate;
        let buffer_len = self.delay_buffer.len();

        for (i, sample) in input.iter().enumerate() {
            // 写入延迟缓冲区
            self.delay_buffer[self.delay_pos] = *sample;
            self.delay_pos = (self.delay_pos + 1) % buffer_len;

            let mut chorus_out = 0.0;

            // 多个调制的声音
            for voice in 0..self.voices {
                let voice_phase = self.phase + voice as f32 * std::f32::consts::TAU / self.voices as f32;
                let lfo = voice_phase.sin();

                // 计算延迟时间
                let base_delay = (0.01 * self.sample_rate) as usize;
                let mod_delay = (self.depth * self.sample_rate * (1.0 + lfo)) as usize;
                let total_delay = (base_delay + mod_delay).min(buffer_len - 1); // 确保不超过缓冲区大小

                // 从延迟缓冲区读取
                let read_pos = if self.delay_pos > total_delay {
                    self.delay_pos - total_delay
                } else {
                    buffer_len - (total_delay - self.delay_pos)
                };

                // 再次验证 read_pos 在有效范围内
                if read_pos < buffer_len {
                    chorus_out += self.delay_buffer[read_pos];
                }
            }

            chorus_out /= self.voices as f32;

            // 混合输出
            output[i] = *sample * (1.0 - self.mix) + chorus_out * self.mix;

            // 更新相位
            self.phase += phase_inc;
            if self.phase >= std::f32::consts::TAU {
                self.phase -= std::f32::consts::TAU;
            }
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "rate" => self.rate = value,
            "depth" => self.depth = value / 10000.0,
            "mix" => self.mix = value / 100.0,
            "voices" => self.voices = value as usize,
            "spread" => self.spread = value / 100.0,
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.delay_buffer.fill(0.0);
        self.delay_pos = 0;
        self.phase = 0.0;
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Chorus
    }
}
