use crate::modules::effects::{AudioProcessor, EffectType};

/// 参数均衡器
pub struct EQProcessor {
    bands: [EQBand; 4],
    sample_rate: f32,
}

struct EQBand {
    frequency: f32,
    gain: f32,
    q: f32,
    // 双二阶滤波器系数
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    // 状态
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl EQBand {
    fn new(frequency: f32, gain: f32, q: f32) -> Self {
        Self {
            frequency,
            gain,
            q,
            b0: 1.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    fn update_coefficients(&mut self, sample_rate: f32) {
        let a = 10.0_f32.powf(self.gain / 40.0);
        let w0 = std::f32::consts::TAU * self.frequency / sample_rate;
        let alpha = w0.sin() / (2.0 * self.q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * w0.cos();
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = 2.0 * w0.cos();
        let a2 = -(1.0 - alpha / a);

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = -a1 / a0;
        self.a2 = -a2 / a0;
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input
            + self.b1 * self.x1
            + self.b2 * self.x2
            + self.a1 * self.y1
            + self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

impl EQProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            bands: [
                EQBand::new(100.0, 0.0, 0.7),   // Low
                EQBand::new(500.0, 0.0, 0.7),   // Low-Mid
                EQBand::new(4000.0, 0.0, 0.7),  // High-Mid
                EQBand::new(12000.0, 0.0, 0.7), // High
            ],
            sample_rate,
        }
    }
}

impl AudioProcessor for EQProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for band in &mut self.bands {
            band.update_coefficients(self.sample_rate);
        }

        for (i, sample) in input.iter().enumerate() {
            let mut processed = *sample;
            for band in &mut self.bands {
                processed = band.process(processed);
            }
            output[i] = processed;
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() != 2 {
            return;
        }

        let band_idx = match parts[0] {
            "low" => Some(0),
            "lowMid" => Some(1),
            "highMid" => Some(2),
            "high" => Some(3),
            _ => None,
        };

        if let Some(idx) = band_idx {
            match parts[1] {
                "gain" => self.bands[idx].gain = value,
                "frequency" => self.bands[idx].frequency = value,
                "q" => self.bands[idx].q = value,
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        for band in &mut self.bands {
            band.reset();
        }
    }

    fn effect_type(&self) -> EffectType {
        EffectType::EQ
    }
}
