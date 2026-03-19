pub mod reverb;
pub mod chorus;
pub mod eq;
pub mod compressor;
pub mod delay;
pub mod deesser;
pub mod exciter;
pub mod gate;

pub use reverb::ReverbProcessor;
pub use chorus::ChorusProcessor;
pub use eq::EQProcessor;
pub use compressor::CompressorProcessor;
pub use delay::DelayProcessor;
pub use deesser::DeEsserProcessor;
pub use exciter::ExciterProcessor;
pub use gate::GateProcessor;

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
}
