use crate::modules::effects::{AudioProcessor, EffectType};
use std::collections::HashMap;

/// 效果器链管理器
/// 管理多个效果器处理器的串联处理
pub struct EffectChain {
    /// 效果器处理器列表
    processors: Vec<Box<dyn AudioProcessor>>,
    /// 每个效果器是否启用
    enabled: Vec<bool>,
    /// 效果器类型映射
    processor_types: Vec<EffectType>,
    /// 每个 effect 对应的数据库 slot_index
    slot_indices: Vec<i32>,
    /// 临时缓冲区
    temp_buffer: Vec<f32>,
}

impl EffectChain {
    /// 创建新的效果器链
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            enabled: Vec::new(),
            processor_types: Vec::new(),
            slot_indices: Vec::new(),
            temp_buffer: Vec::new(),
        }
    }

    /// 添加效果器处理器
    pub fn add_processor(&mut self, processor: Box<dyn AudioProcessor>) {
        self.add_processor_with_slot_index(processor, self.processors.len() as i32)
    }

    /// 添加效果器处理器并指定 slot_index
    pub fn add_processor_with_slot_index(&mut self, processor: Box<dyn AudioProcessor>, slot_index: i32) {
        let effect_type = processor.effect_type();
        self.processors.push(processor);
        self.enabled.push(true);
        self.processor_types.push(effect_type);
        self.slot_indices.push(slot_index);
    }

    /// 在指定位置插入效果器处理器
    pub fn insert_processor(&mut self, index: usize, processor: Box<dyn AudioProcessor>) {
        if index <= self.processors.len() {
            let effect_type = processor.effect_type();
            self.processors.insert(index, processor);
            self.enabled.insert(index, true);
            self.processor_types.insert(index, effect_type);
            self.slot_indices.insert(index, index as i32);
        }
    }

    /// 移除效果器处理器
    pub fn remove_processor(&mut self, index: usize) -> Option<Box<dyn AudioProcessor>> {
        if index < self.processors.len() {
            self.enabled.remove(index);
            self.processor_types.remove(index);
            self.slot_indices.remove(index);
            Some(self.processors.remove(index))
        } else {
            None
        }
    }

    /// 切换效果器启用状态
    pub fn toggle_processor(&mut self, index: usize) {
        if index < self.enabled.len() {
            self.enabled[index] = !self.enabled[index];
        }
    }

    /// 设置效果器启用状态
    pub fn set_enabled(&mut self, index: usize, enabled: bool) {
        if index < self.enabled.len() {
            self.enabled[index] = enabled;
        }
    }

    /// 获取效果器启用状态
    pub fn is_enabled(&self, index: usize) -> bool {
        self.enabled.get(index).copied().unwrap_or(false)
    }

    /// 获取效果器数量
    pub fn len(&self) -> usize {
        self.processors.len()
    }

    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.processors.is_empty()
    }

    /// 处理音频数据
    /// 输入和输出可以是同一个缓冲区（就地处理）
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        if self.processors.is_empty() {
            // 没有效果器，直接复制
            output.copy_from_slice(input);
            return;
        }

        // 确保临时缓冲区足够大
        if self.temp_buffer.len() != input.len() {
            self.temp_buffer = vec![0.0; input.len()];
        }

        // 初始复制到输出
        output.copy_from_slice(input);

        // 依次通过每个效果器
        for (i, processor) in self.processors.iter_mut().enumerate() {
            if self.enabled[i] {
                // 保存当前输出到临时缓冲区
                self.temp_buffer.copy_from_slice(output);

                // 处理
                processor.process(&self.temp_buffer, output);
            }
            // 如果效果器被禁用，跳过它
        }
    }

    /// 重置所有效果器状态
    pub fn reset(&mut self) {
        for processor in &mut self.processors {
            processor.reset();
        }
    }

    /// 设置效果器参数
    pub fn set_parameter(&mut self, index: usize, name: &str, value: f32) {
        if let Some(processor) = self.processors.get_mut(index) {
            processor.set_parameter(name, value);
        }
    }

    /// 批量设置效果器参数
    pub fn set_parameters(&mut self, index: usize, params: &HashMap<String, f32>) {
        if let Some(processor) = self.processors.get_mut(index) {
            for (name, value) in params {
                processor.set_parameter(name, *value);
            }
        }
    }

    /// 获取效果器类型
    pub fn get_effect_type(&self, index: usize) -> Option<EffectType> {
        self.processor_types.get(index).copied()
    }

    /// 移动效果器位置
    pub fn move_processor(&mut self, from: usize, to: usize) {
        if from >= self.processors.len() || to >= self.processors.len() || from == to {
            return;
        }

        // 移动处理器
        let processor = self.processors.remove(from);
        self.processors.insert(to, processor);

        // 移动启用状态
        let enabled = self.enabled.remove(from);
        self.enabled.insert(to, enabled);

        // 移动类型
        let effect_type = self.processor_types.remove(from);
        self.processor_types.insert(to, effect_type);

        // 移动 slot_index
        let slot_index = self.slot_indices.remove(from);
        self.slot_indices.insert(to, slot_index);
    }

    /// 清空所有效果器
    pub fn clear(&mut self) {
        self.processors.clear();
        self.enabled.clear();
        self.processor_types.clear();
        self.slot_indices.clear();
    }

    /// 根据 slot_index 查找在链中的位置
    fn find_index_by_slot_index(&self, slot_index: i32) -> Option<usize> {
        self.slot_indices.iter().position(|&si| si == slot_index)
    }

    /// 获取指定 slot_index 的 LevelMeter 电平值
    pub fn get_level_meter_value_by_slot(&self, slot_index: i32) -> Option<f32> {
        if let Some(index) = self.find_index_by_slot_index(slot_index) {
            if index < self.processors.len() {
                return self.processors[index].get_level();
            }
        }
        None
    }

    /// 检查指定 slot_index 是否是 LevelMeter 类型
    pub fn is_level_meter(&self, slot_index: i32) -> bool {
        if let Some(index) = self.find_index_by_slot_index(slot_index) {
            if let Some(effect_type) = self.processor_types.get(index) {
                let is_meter = *effect_type == EffectType::LevelMeter;
                eprintln!("[EffectChain.is_level_meter] slot_index={}, index={}, effect_type={:?}, is_meter={}",
                         slot_index, index, effect_type, is_meter);
                return is_meter;
            }
        }
        eprintln!("[EffectChain.is_level_meter] slot_index={} not found in slot_indices: {:?}", slot_index, self.slot_indices);
        false
    }
}

impl Default for EffectChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用的简单效果器
    struct TestProcessor {
        gain: f32,
    }

    impl AudioProcessor for TestProcessor {
        fn process(&mut self, input: &[f32], output: &mut [f32]) {
            for (i, sample) in input.iter().enumerate() {
                output[i] = sample * self.gain;
            }
        }

        fn set_parameter(&mut self, name: &str, value: f32) {
            if name == "gain" {
                self.gain = value;
            }
        }

        fn reset(&mut self) {}

        fn effect_type(&self) -> EffectType {
            EffectType::Reverb
        }
    }

    #[test]
    fn test_effect_chain_process() {
        let mut chain = EffectChain::new();

        // 添加一个增益为 0.5 的处理器
        let mut processor = Box::new(TestProcessor { gain: 0.5 });
        chain.add_processor(processor);

        let input = [1.0, 0.5, -0.5, -1.0];
        let mut output = [0.0; 4];

        chain.process(&input, &mut output);

        assert!((output[0] - 0.5).abs() < 0.001);
        assert!((output[1] - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_toggle_processor() {
        let mut chain = EffectChain::new();
        chain.add_processor(Box::new(TestProcessor { gain: 0.5 }));

        assert!(chain.is_enabled(0));

        chain.toggle_processor(0);
        assert!(!chain.is_enabled(0));

        chain.set_enabled(0, true);
        assert!(chain.is_enabled(0));
    }
}
