use crate::modules::effects::{AudioProcessor, EffectType};

/// 参数均衡器（带低切/高切）
pub struct EQProcessor {
    bands: [EQBand; 4],
    low_cut: LowCutFilter,
    high_cut: HighCutFilter,
    sample_rate: f32,
    coeffs_updated: bool,  // 使用实例变量代替 static
}

/// 参数均衡器频段
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
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * self.q);

        // Peaking EQ coefficients from RBJ Audio EQ Cookbook
        // For gain = 0 dB, this should be a flat response (b0=1, b1=0, b2=0, a1=0, a2=0)
        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = 2.0 * cos_w0;
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

/// 低切滤波器（高通滤波器）
struct LowCutFilter {
    enabled: bool,
    frequency: f32,
    // 双二阶滤波器系数
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    // 状态
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl LowCutFilter {
    fn new() -> Self {
        Self {
            enabled: false,
            frequency: 80.0,
            b0: 1.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    fn update_coefficients(&mut self, sample_rate: f32) {
        if !self.enabled {
            return;
        }

        // 2阶 Butterworth 高通滤波器
        let w0 = std::f32::consts::TAU * self.frequency / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        // Q = 0.707 for Butterworth
        let alpha = sin_w0 / (2.0 * 0.707);

        let a0 = 1.0 + alpha;

        self.b0 = (1.0 + cos_w0) / 2.0 / a0;
        self.b1 = -(1.0 + cos_w0) / a0;
        self.b2 = (1.0 + cos_w0) / 2.0 / a0;
        self.a1 = 2.0 * cos_w0 / a0;
        self.a2 = -(1.0 - alpha) / a0;
    }

    fn process(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }

        let output = self.b0 * input
            + self.b1 * self.x1
            + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

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

/// 高切滤波器（低通滤波器）
struct HighCutFilter {
    enabled: bool,
    frequency: f32,
    // 双二阶滤波器系数
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    // 状态
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl HighCutFilter {
    fn new() -> Self {
        Self {
            enabled: false,
            frequency: 12000.0,
            b0: 1.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    fn update_coefficients(&mut self, sample_rate: f32) {
        if !self.enabled {
            return;
        }

        // 2阶 Butterworth 低通滤波器
        let w0 = std::f32::consts::TAU * self.frequency / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        // Q = 0.707 for Butterworth
        let alpha = sin_w0 / (2.0 * 0.707);

        let a0 = 1.0 + alpha;

        self.b0 = (1.0 - cos_w0) / 2.0 / a0;
        self.b1 = (1.0 - cos_w0) / a0;
        self.b2 = (1.0 - cos_w0) / 2.0 / a0;
        self.a1 = -2.0 * cos_w0 / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    fn process(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }

        let output = self.b0 * input
            + self.b1 * self.x1
            + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

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
        let mut processor = Self {
            bands: [
                EQBand::new(100.0, 0.0, 0.7),   // Low
                EQBand::new(500.0, 0.0, 0.7),   // Low-Mid
                EQBand::new(4000.0, 0.0, 0.7),  // High-Mid
                EQBand::new(12000.0, 0.0, 0.7), // High
            ],
            low_cut: LowCutFilter::new(),
            high_cut: HighCutFilter::new(),
            sample_rate,
            coeffs_updated: false,
        };
        // 在初始化时更新系数
        processor.update_all_coefficients();
        processor
    }

    fn update_all_coefficients(&mut self) {
        for band in &mut self.bands {
            band.update_coefficients(self.sample_rate);
        }
        self.low_cut.update_coefficients(self.sample_rate);
        self.high_cut.update_coefficients(self.sample_rate);
        self.coeffs_updated = true;
    }
}

impl AudioProcessor for EQProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        // 暂时直接复制输入到输出，用于调试
        // TODO: 修复 EQ 滤波器不稳定问题
        output.copy_from_slice(input);
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        // 处理低切/高切参数
        match name {
            "lowCut.enabled" => {
                self.low_cut.enabled = value > 0.5;
                return;
            }
            "lowCut.frequency" => {
                self.low_cut.frequency = value;
                return;
            }
            "highCut.enabled" => {
                self.high_cut.enabled = value > 0.5;
                return;
            }
            "highCut.frequency" => {
                self.high_cut.frequency = value;
                return;
            }
            _ => {}
        }

        // 处理频段参数
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
        self.low_cut.reset();
        self.high_cut.reset();
    }

    fn effect_type(&self) -> EffectType {
        EffectType::EQ
    }
}
