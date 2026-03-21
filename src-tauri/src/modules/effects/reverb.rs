use crate::modules::effects::{AudioProcessor, EffectType};

/// 专业混响处理器 (基于 Freeverb 算法)
///
/// Freeverb 使用 8 个并行梳状滤波器和 4 个串行全通滤波器
/// 来模拟自然混响的特性
pub struct ReverbProcessor {
    // 参数
    pub room_size: f32,      // 房间大小 (0-1)
    pub damping: f32,        // 阻尼 (高频衰减)
    pub wet_level: f32,      // 湿声比例
    pub dry_level: f32,      // 干声比例
    pub width: f32,          // 立体声宽度
    pub pre_delay_ms: f32,   // 预延迟 (毫秒)

    // 左声道滤波器
    comb_l: Vec<CombFilter>,
    allpass_l: Vec<AllPassFilter>,

    // 右声道滤波器 (用于立体声)
    comb_r: Vec<CombFilter>,
    allpass_r: Vec<AllPassFilter>,

    // 预延迟缓冲区
    pre_delay_buffer: Vec<f32>,
    pre_delay_pos: usize,
    pre_delay_length: usize,

    sample_rate: f32,

    // 房间类型
    room_type: RoomType,
}

#[derive(Clone, Copy, Debug)]
enum RoomType {
    Small,      // 小房间 - 短混响
    Medium,     // 中等房间 - 中等混响
    Large,      // 大房间/厅堂 - 长混响
    Cathedral,  // 大教堂 - 很长混响
}

/// 梳状滤波器 - 产生混响的"尾音"
#[derive(Clone)]
struct CombFilter {
    buffer: Vec<f32>,
    position: usize,
    feedback: f32,
    filter_store: f32,  // 低通滤波器状态
    damp1: f32,         // 阻尼系数 1
    damp2: f32,         // 阻尼系数 2
}

impl CombFilter {
    fn new(size: usize) -> Self {
        // 确保缓冲区至少有一个元素
        let safe_size = size.max(1);
        Self {
            buffer: vec![0.0; safe_size],
            position: 0,
            feedback: 0.84,
            filter_store: 0.0,
            damp1: 0.2,
            damp2: 0.8,
        }
    }

    fn set_damping(&mut self, damping: f32) {
        self.damp1 = damping;
        self.damp2 = 1.0 - damping;
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffer_len = self.buffer.len();
        if buffer_len == 0 {
            return input;
        }

        let output = self.buffer[self.position % buffer_len];

        // 低通滤波 (阻尼)
        self.filter_store = output * self.damp2 + self.filter_store * self.damp1;

        // 写入缓冲区
        self.buffer[self.position % buffer_len] = input + self.filter_store * self.feedback;

        self.position = (self.position + 1) % buffer_len;
        output
    }

    fn process_interpolated(&mut self, input: f32, position_offset: f32) -> f32 {
        let buffer_len = self.buffer.len();
        if buffer_len == 0 {
            return input;
        }

        let base_pos = self.position;

        // 线性插值读取
        let read_pos = (base_pos as f32 + position_offset) % buffer_len as f32;
        let pos_int = read_pos as usize;
        let pos_frac = read_pos - pos_int as f32;
        let next_pos = (pos_int + 1) % buffer_len;

        let output = self.buffer[pos_int] * (1.0 - pos_frac) + self.buffer[next_pos] * pos_frac;

        // 低通滤波 (阻尼)
        self.filter_store = output * self.damp2 + self.filter_store * self.damp1;

        // 写入缓冲区
        self.buffer[self.position % buffer_len] = input + self.filter_store * self.feedback;

        self.position = (self.position + 1) % buffer_len;
        output
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.position = 0;
        self.filter_store = 0.0;
    }
}

/// 全通滤波器 - 产生密集的早期反射
#[derive(Clone)]
struct AllPassFilter {
    buffer: Vec<f32>,
    position: usize,
    feedback: f32,
}

impl AllPassFilter {
    fn new(size: usize) -> Self {
        // 确保缓冲区至少有一个元素
        let safe_size = size.max(1);
        Self {
            buffer: vec![0.0; safe_size],
            position: 0,
            feedback: 0.5,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffer_len = self.buffer.len();
        if buffer_len == 0 {
            return input;
        }

        let buffer_out = self.buffer[self.position % buffer_len];
        let output = -input + buffer_out;
        self.buffer[self.position % buffer_len] = input + buffer_out * self.feedback;
        self.position = (self.position + 1) % buffer_len;
        output
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.position = 0;
    }
}

impl ReverbProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let mut reverb = Self {
            room_size: 0.5,
            damping: 0.5,
            wet_level: 0.3,
            dry_level: 0.7,
            width: 1.0,
            pre_delay_ms: 10.0,
            comb_l: Vec::new(),
            comb_r: Vec::new(),
            allpass_l: Vec::new(),
            allpass_r: Vec::new(),
            pre_delay_buffer: vec![0.0; 1000],
            pre_delay_pos: 0,
            pre_delay_length: 100,
            sample_rate,
            room_type: RoomType::Medium,
        };

        reverb.init_filters();
        reverb
    }

    fn init_filters(&mut self) {
        // Freeverb 标准延迟时间 (采样数，基于 44100Hz)
        // 这些数字是质数或接近质数，以避免共振
        let comb_tuning_l = [
            1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617
        ];
        let comb_tuning_r = [
            1116 + 23, 1188 + 23, 1277 + 23, 1356 + 23,
            1422 + 23, 1491 + 23, 1557 + 23, 1617 + 23
        ];
        let allpass_tuning_l = [556, 441, 341, 225];
        let allpass_tuning_r = [556 + 23, 441 + 23, 341 + 23, 225 + 23];

        let scale = self.sample_rate / 44100.0;

        // 创建梳状滤波器
        self.comb_l = comb_tuning_l.iter()
            .map(|&t| CombFilter::new((t as f32 * scale) as usize))
            .collect();
        self.comb_r = comb_tuning_r.iter()
            .map(|&t| CombFilter::new((t as f32 * scale) as usize))
            .collect();

        // 创建全通滤波器
        self.allpass_l = allpass_tuning_l.iter()
            .map(|&t| AllPassFilter::new((t as f32 * scale) as usize))
            .collect();
        self.allpass_r = allpass_tuning_r.iter()
            .map(|&t| AllPassFilter::new((t as f32 * scale) as usize))
            .collect();
    }

    fn update_params(&mut self) {
        // 根据 room_size 调整 feedback
        // Freeverb 的 feedback 范围约为 0.7 到 0.98
        let feedback = 0.7 + self.room_size * 0.28;

        // 根据阻尼调整滤波器参数
        let damp = self.damping * 0.4;

        for comb in &mut self.comb_l {
            comb.feedback = feedback;
            comb.set_damping(damp);
        }
        for comb in &mut self.comb_r {
            comb.feedback = feedback;
            comb.set_damping(damp);
        }

        // 更新预延迟长度
        let new_delay = ((self.pre_delay_ms / 1000.0) * self.sample_rate) as usize;
        let max_delay = self.pre_delay_buffer.len();
        self.pre_delay_length = new_delay.min(max_delay - 1).max(1);
    }

    /// 设置房间类型预设
    fn set_room_type(&mut self, room_type: RoomType) {
        self.room_type = room_type;
        match room_type {
            RoomType::Small => {
                self.room_size = 0.3;
                self.damping = 0.6;
                self.pre_delay_ms = 5.0;
            }
            RoomType::Medium => {
                self.room_size = 0.5;
                self.damping = 0.5;
                self.pre_delay_ms = 10.0;
            }
            RoomType::Large => {
                self.room_size = 0.7;
                self.damping = 0.4;
                self.pre_delay_ms = 20.0;
            }
            RoomType::Cathedral => {
                self.room_size = 0.9;
                self.damping = 0.3;
                self.pre_delay_ms = 30.0;
            }
        }
    }

    /// 处理单声道输入，输出立体声 (交错格式)
    fn process_stereo(&mut self, input: &[f32], output: &mut [f32]) {
        self.update_params();

        // 确保 pre_delay_length 至少为 1
        let delay_len = self.pre_delay_length.max(1);
        let delay_buf_len = self.pre_delay_buffer.len();

        let wet_gain = self.wet_level;
        let dry_gain = self.dry_level;
        let width_gain = self.width;

        for (i, &sample) in input.iter().enumerate() {
            // 预延迟
            let delayed = if self.pre_delay_pos < delay_buf_len {
                self.pre_delay_buffer[self.pre_delay_pos]
            } else {
                0.0
            };

            if self.pre_delay_pos < delay_buf_len {
                self.pre_delay_buffer[self.pre_delay_pos] = sample;
            }
            self.pre_delay_pos = (self.pre_delay_pos + 1) % delay_len;

            // 左声道梳状滤波器
            let mut comb_l_out = 0.0f32;
            for comb in &mut self.comb_l {
                comb_l_out += comb.process(delayed);
            }

            // 右声道梳状滤波器
            let mut comb_r_out = 0.0f32;
            for comb in &mut self.comb_r {
                comb_r_out += comb.process(delayed);
            }

            // 左声道全通滤波器
            let mut allpass_l_out = comb_l_out / 8.0;
            for allpass in &mut self.allpass_l {
                allpass_l_out = allpass.process(allpass_l_out);
            }

            // 右声道全通滤波器
            let mut allpass_r_out = comb_r_out / 8.0;
            for allpass in &mut self.allpass_r {
                allpass_r_out = allpass.process(allpass_r_out);
            }

            // 立体声宽度处理
            let wet_l = allpass_l_out * wet_gain;
            let wet_r = allpass_r_out * wet_gain;

            // 应用宽度 (中/侧处理)
            let mid = (wet_l + wet_r) * 0.5;
            let side = (wet_l - wet_r) * 0.5 * width_gain;
            let wet_l_out = mid + side;
            let wet_r_out = mid - side;

            // 输出 (交错立体声)
            let out_idx = i * 2;
            if out_idx + 1 < output.len() {
                output[out_idx] = sample * dry_gain + wet_l_out;
                output[out_idx + 1] = sample * dry_gain + wet_r_out;
            }
        }
    }

    /// 处理单声道输入，输出单声道
    fn process_mono(&mut self, input: &[f32], output: &mut [f32]) {
        self.update_params();

        // 确保 pre_delay_length 至少为 1
        let delay_len = self.pre_delay_length.max(1);
        let delay_buf_len = self.pre_delay_buffer.len();

        for (i, &sample) in input.iter().enumerate() {
            // 预延迟
            let delayed = if self.pre_delay_pos < delay_buf_len {
                self.pre_delay_buffer[self.pre_delay_pos]
            } else {
                0.0
            };

            if self.pre_delay_pos < delay_buf_len {
                self.pre_delay_buffer[self.pre_delay_pos] = sample;
            }
            self.pre_delay_pos = (self.pre_delay_pos + 1) % delay_len;

            // 梳状滤波器 (只使用左声道)
            let mut comb_out = 0.0f32;
            for comb in &mut self.comb_l {
                comb_out += comb.process(delayed);
            }

            // 全通滤波器
            let mut allpass_out = comb_out / 8.0;
            for allpass in &mut self.allpass_l {
                allpass_out = allpass.process(allpass_out);
            }

            // 输出
            output[i] = sample * self.dry_level + allpass_out * self.wet_level;
        }
    }
}

impl AudioProcessor for ReverbProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        // 默认使用单声道处理
        self.process_mono(input, output);
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "roomSize" => self.room_size = value / 100.0,
            "damping" => self.damping = value / 100.0,
            "wetLevel" => self.wet_level = value / 100.0,
            "dryLevel" => self.dry_level = value / 100.0,
            "preDelay" => self.pre_delay_ms = value,
            "width" => self.width = value / 100.0,

            // 房间类型预设
            "roomType" => {
                match value as i32 {
                    0 => self.set_room_type(RoomType::Small),
                    1 => self.set_room_type(RoomType::Medium),
                    2 => self.set_room_type(RoomType::Large),
                    3 => self.set_room_type(RoomType::Cathedral),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        for comb in &mut self.comb_l {
            comb.reset();
        }
        for comb in &mut self.comb_r {
            comb.reset();
        }
        for allpass in &mut self.allpass_l {
            allpass.reset();
        }
        for allpass in &mut self.allpass_r {
            allpass.reset();
        }
        self.pre_delay_buffer.fill(0.0);
        self.pre_delay_pos = 0;
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Reverb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverb_processor() {
        let mut processor = ReverbProcessor::new(44100.0);

        // 测试处理
        let input = vec![0.5; 256];
        let mut output = vec![0.0; 256];
        processor.process(&input, &mut output);

        // 输出不应该全是零
        assert!(output.iter().any(|&x| x != 0.0));
    }

    #[test]
    fn test_room_types() {
        let mut processor = ReverbProcessor::new(44100.0);

        // 测试房间类型预设
        processor.set_parameter("roomType", 0.0); // Small
        assert!((processor.room_size - 0.3).abs() < 0.01);

        processor.set_parameter("roomType", 2.0); // Large
        assert!((processor.room_size - 0.7).abs() < 0.01);
    }
}
