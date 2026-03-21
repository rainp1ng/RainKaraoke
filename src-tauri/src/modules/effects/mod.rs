pub mod reverb;
pub mod chorus;
pub mod eq;
pub mod compressor;
pub mod delay;
pub mod deesser;
pub mod exciter;
pub mod gate;
pub mod gain;
pub mod level_meter;
pub mod chain;

pub use reverb::ReverbProcessor;
pub use chorus::ChorusProcessor;
pub use eq::EQProcessor;
pub use compressor::CompressorProcessor;
pub use delay::DelayProcessor;
pub use deesser::DeEsserProcessor;
pub use exciter::ExciterProcessor;
pub use gate::GateProcessor;
pub use gain::GainProcessor;
pub use level_meter::LevelMeterProcessor;
pub use chain::EffectChain;

use serde::{Deserialize, Serialize};

/// 效果器类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EffectType {
    Reverb,
    Chorus,
    EQ,
    Compressor,
    Delay,
    DeEsser,
    Exciter,
    Gate,
    Gain,
    LevelMeter,
}

/// 从字符串创建效果器类型
pub fn effect_type_from_str(s: &str) -> Option<EffectType> {
    match s.to_lowercase().as_str() {
        "reverb" => Some(EffectType::Reverb),
        "chorus" => Some(EffectType::Chorus),
        "eq" => Some(EffectType::EQ),
        "compressor" => Some(EffectType::Compressor),
        "delay" => Some(EffectType::Delay),
        "deesser" => Some(EffectType::DeEsser),
        "exciter" => Some(EffectType::Exciter),
        "gate" => Some(EffectType::Gate),
        "gain" => Some(EffectType::Gain),
        "levelmeter" => Some(EffectType::LevelMeter),
        _ => None,
    }
}

/// 创建效果器处理器
pub fn create_processor(effect_type: EffectType, sample_rate: f32) -> Box<dyn AudioProcessor> {
    match effect_type {
        EffectType::Reverb => Box::new(ReverbProcessor::new(sample_rate)),
        EffectType::Chorus => Box::new(ChorusProcessor::new(sample_rate)),
        EffectType::EQ => Box::new(EQProcessor::new(sample_rate)),
        EffectType::Compressor => Box::new(CompressorProcessor::new(sample_rate)),
        EffectType::Delay => Box::new(DelayProcessor::new(sample_rate)),
        EffectType::DeEsser => Box::new(DeEsserProcessor::new(sample_rate)),
        EffectType::Exciter => Box::new(ExciterProcessor::new(sample_rate)),
        EffectType::Gate => Box::new(GateProcessor::new(sample_rate)),
        EffectType::Gain => Box::new(GainProcessor::new()),
        EffectType::LevelMeter => Box::new(LevelMeterProcessor::new(sample_rate)),
    }
}

/// 音频处理器接口
pub trait AudioProcessor: Send {
    /// 处理音频数据
    fn process(&mut self, input: &[f32], output: &mut [f32]);

    /// 设置参数
    fn set_parameter(&mut self, name: &str, value: f32);

    /// 重置状态
    fn reset(&mut self);

    /// 获取效果器类型
    fn effect_type(&self) -> EffectType;

    /// 获取电平表值（只有 LevelMeter 类型有效）
    fn get_level(&self) -> Option<f32> {
        None
    }
}
