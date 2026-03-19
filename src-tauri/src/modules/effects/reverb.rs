use crate::modules::effects::{AudioProcessor, EffectType};

/// 简化的混响处理器 (基于 Freeverb 原理)
pub struct ReverbProcessor {
    // 参数
    pub room_size: f32,
    pub damping: f32,
    pub wet_level: f32,
    pub dry_level: f32,
    pub pre_delay: f32,

    // 内部状态
    comb_filters: Vec<CombFilter>,
    allpass_filters: Vec<AllPassFilter>,
    pre_delay_buffer: Vec<f32>,
    pre_delay_pos: usize,
    sample_rate: f32,
}

#[derive(Clone)]
struct CombFilter {
    buffer: Vec<f32>,
    position: usize,
    feedback: f32,
    damp: f32,
    damp_state: f32,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            position: 0,
            feedback: 0.84,
            damp: 0.2,
            damp_state: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.position];
        self.damp_state = output * (1.0 - self.damp) + self.damp_state * self.damp;
        self.buffer[self.position] = input + self.damp_state * self.feedback;
        self.position = (self.position + 1) % self.buffer.len();
        output
    }
}

#[derive(Clone)]
struct AllPassFilter {
    buffer: Vec<f32>,
    position: usize,
    feedback: f32,
}

impl AllPassFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            position: 0,
            feedback: 0.5,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffer_out = self.buffer[self.position];
        let output = -input + buffer_out;
        self.buffer[self.position] = input + buffer_out * self.feedback;
        self.position = (self.position + 1) % self.buffer.len();
        output
    }
}

impl ReverbProcessor {
    pub fn new(sample_rate: f32) -> Self {
        // 基于 Freeverb 的延迟时间
        let comb_sizes = vec![
            (1116.0 * sample_rate / 44100.0) as usize,
            (1188.0 * sample_rate / 44100.0) as usize,
            (1277.0 * sample_rate / 44100.0) as usize,
            (1356.0 * sample_rate / 44100.0) as usize,
            (1422.0 * sample_rate / 44100.0) as usize,
            (1491.0 * sample_rate / 44100.0) as usize,
            (1557.0 * sample_rate / 44100.0) as usize,
            (1617.0 * sample_rate / 44100.0) as usize,
        ];

        let allpass_sizes = vec![
            (556.0 * sample_rate / 44100.0) as usize,
            (441.0 * sample_rate / 44100.0) as usize,
        ];

        let comb_filters: Vec<CombFilter> = comb_sizes.into_iter().map(CombFilter::new).collect();
        let allpass_filters: Vec<AllPassFilter> = allpass_sizes.into_iter().map(AllPassFilter::new).collect();

        Self {
            room_size: 0.5,
            damping: 0.5,
            wet_level: 0.3,
            dry_level: 0.7,
            pre_delay: 0.0,
            comb_filters,
            allpass_filters,
            pre_delay_buffer: vec![0.0; 100],
            pre_delay_pos: 0,
            sample_rate,
        }
    }

    fn update_params(&mut self) {
        for comb in &mut self.comb_filters {
            comb.feedback = self.room_size * 0.7 + 0.14;
            comb.damp = self.damping * 0.4;
        }
    }
}

impl AudioProcessor for ReverbProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.update_params();

        for (i, sample) in input.iter().enumerate() {
            // 预延迟
            let delayed = self.pre_delay_buffer[self.pre_delay_pos];
            self.pre_delay_buffer[self.pre_delay_pos] = *sample;
            self.pre_delay_pos = (self.pre_delay_pos + 1) % self.pre_delay_buffer.len();

            // 并行梳状滤波器
            let mut comb_out = 0.0;
            for comb in &mut self.comb_filters {
                comb_out += comb.process(delayed);
            }

            // 串行全通滤波器
            let mut allpass_out = comb_out / 8.0;
            for allpass in &mut self.allpass_filters {
                allpass_out = allpass.process(allpass_out);
            }

            // 混合输出
            output[i] = *sample * self.dry_level + allpass_out * self.wet_level;
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "roomSize" => self.room_size = value / 100.0,
            "damping" => self.damping = value / 100.0,
            "wetLevel" => self.wet_level = value / 100.0,
            "dryLevel" => self.dry_level = value / 100.0,
            "preDelay" => self.pre_delay = value,
            _ => {}
        }
    }

    fn reset(&mut self) {
        for comb in &mut self.comb_filters {
            comb.buffer.fill(0.0);
            comb.damp_state = 0.0;
            comb.position = 0;
        }
        for allpass in &mut self.allpass_filters {
            allpass.buffer.fill(0.0);
            allpass.position = 0;
        }
        self.pre_delay_buffer.fill(0.0);
        self.pre_delay_pos = 0;
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Reverb
    }
}
