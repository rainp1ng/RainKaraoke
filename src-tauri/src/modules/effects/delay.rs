use crate::modules::effects::{AudioProcessor, EffectType};

/// 延迟效果器
pub struct DelayProcessor {
    pub time: f32,      // ms
    pub feedback: f32,
    pub mix: f32,
    pub ping_pong: bool,

    left_buffer: Vec<f32>,
    right_buffer: Vec<f32>,
    left_pos: usize,
    right_pos: usize,
    sample_rate: f32,
}

impl DelayProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let max_samples = (sample_rate * 2.0) as usize; // 2秒最大延迟
        Self {
            time: 250.0,
            feedback: 0.3,
            mix: 0.2,
            ping_pong: false,
            left_buffer: vec![0.0; max_samples],
            right_buffer: vec![0.0; max_samples],
            left_pos: 0,
            right_pos: 0,
            sample_rate,
        }
    }
}

impl AudioProcessor for DelayProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let delay_samples = (self.time * self.sample_rate / 1000.0) as usize;

        for (i, sample) in input.iter().enumerate() {
            let read_pos = if self.left_pos > delay_samples {
                self.left_pos - delay_samples
            } else {
                self.left_buffer.len() - (delay_samples - self.left_pos)
            };

            let delayed = self.left_buffer[read_pos];

            // 写入缓冲区
            self.left_buffer[self.left_pos] = *sample + delayed * self.feedback;
            self.left_pos = (self.left_pos + 1) % self.left_buffer.len();

            // 混合输出
            output[i] = *sample * (1.0 - self.mix) + delayed * self.mix;
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "time" => self.time = value,
            "feedback" => self.feedback = value / 100.0,
            "mix" => self.mix = value / 100.0,
            "pingPong" => self.ping_pong = value > 0.5,
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.left_buffer.fill(0.0);
        self.right_buffer.fill(0.0);
        self.left_pos = 0;
        self.right_pos = 0;
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Delay
    }
}
