use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig, SupportedStreamConfig};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::modules::audio_router::DualTrackRecorder;
use crate::modules::effects::{EffectChain, create_processor, effect_type_from_str, AudioProcessor};

/// 效果器输入源选择
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffectInput {
    Vocal,
    Instrument,
    None,
}

impl Default for EffectInput {
    fn default() -> Self {
        Self::Vocal
    }
}

/// 实时音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveAudioConfig {
    /// 人声输入设备名称
    pub vocal_input_device: Option<String>,
    /// 人声输入通道（0-indexed，-1 表示使用所有通道混合）
    pub vocal_input_channel: i32,
    /// 乐器输入设备名称
    pub instrument_input_device: Option<String>,
    /// 乐器输入通道
    pub instrument_input_channel: i32,
    /// 监听输出设备名称
    pub monitor_output_device: String,
    /// 直播输出设备名称（可选）
    pub stream_output_device: Option<String>,
    /// 人声音量 (0.0 - 1.0)
    pub vocal_volume: f32,
    /// 乐器音量 (0.0 - 1.0)
    pub instrument_volume: f32,
    /// 效果器输入源
    pub effect_input: EffectInput,
    /// 监听音量 (0.0 - 1.0)
    pub monitor_volume: f32,
    /// 直播音量 (0.0 - 1.0)
    pub stream_volume: f32,
}

impl Default for LiveAudioConfig {
    fn default() -> Self {
        Self {
            vocal_input_device: None,
            vocal_input_channel: 0,
            instrument_input_device: None,
            instrument_input_channel: 1,
            monitor_output_device: String::new(),
            stream_output_device: None,
            vocal_volume: 0.8,
            instrument_volume: 0.8,
            effect_input: EffectInput::Vocal,
            monitor_volume: 0.8,
            stream_volume: 1.0,
        }
    }
}

/// 设备信息（包含通道数）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub name: String,
    pub channels: u16,
    pub sample_rate: u32,
    pub is_default: bool,
}

/// 实时音频状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveAudioState {
    pub is_running: bool,
    pub config: LiveAudioConfig,
    pub vocal_recording: bool,
    pub instrument_recording: bool,
}

/// 音频处理配置（在回调中使用）
struct AudioProcessConfig {
    vocal_volume: f32,
    instrument_volume: f32,
    monitor_volume: f32,
    effect_input: EffectInput,
    vocal_channel: usize,
    instrument_channel: usize,
}

/// 音频流容器
pub struct AudioStreams {
    pub vocal_input: Option<Stream>,
    pub instrument_input: Option<Stream>,
    pub monitor_output: Option<Stream>,
    pub stream_output: Option<Stream>,
}

impl AudioStreams {
    pub fn new() -> Self {
        Self {
            vocal_input: None,
            instrument_input: None,
            monitor_output: None,
            stream_output: None,
        }
    }

    pub fn stop(&mut self) {
        // 显式停止每个流
        if let Some(stream) = self.vocal_input.take() {
            drop(stream);
        }
        if let Some(stream) = self.instrument_input.take() {
            drop(stream);
        }
        if let Some(stream) = self.monitor_output.take() {
            drop(stream);
        }
        if let Some(stream) = self.stream_output.take() {
            drop(stream);
        }
        println!("[AudioStreams] All streams stopped and dropped");
    }
}

/// 简单的音频缓冲区 - 存储最新的音频数据
/// 输入写入数据，输出读取数据
/// 如果输出读取速度跟不上输入，会读到静音
/// 如果输入写入速度跟不上输出，会读到重复的数据
pub struct AudioBuffer {
    /// 存储最近的音频数据
    buffer: Vec<f32>,
    /// 缓冲区大小
    capacity: usize,
    /// 当前写入位置
    write_pos: usize,
    /// 当前读取位置
    read_pos: usize,
    /// 可供读取的样本数
    available: usize,
}

impl AudioBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_pos: 0,
            read_pos: 0,
            available: 0,
        }
    }

    /// 写入数据
    pub fn write(&mut self, data: &[f32]) {
        for &sample in data {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;
            if self.available < self.capacity {
                self.available += 1;
            } else {
                // 缓冲区满，读取位置也要跟着移动
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
        }
    }

    /// 读取数据（消耗式读取）
    pub fn read(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            if self.available > 0 {
                *sample = self.buffer[self.read_pos];
                self.read_pos = (self.read_pos + 1) % self.capacity;
                self.available -= 1;
            } else {
                *sample = 0.0;
            }
        }
    }

    /// 读取最新数据（非消耗式，用于多个输出流）
    /// 直接复制最新的数据，不移动读取位置
    pub fn read_latest(&self, output: &mut [f32]) {
        let len = output.len();

        if self.available == 0 {
            output.fill(0.0);
            return;
        }

        // 如果可用的数据足够，从最新的位置往前读取
        // 如果不够，读取所有可用的，其余填静音
        let to_read = len.min(self.available);

        // 计算读取起始位置（从最新数据往前推）
        let read_start = if self.available >= len {
            // 有足够的数据，从 write_pos - len 开始读
            (self.write_pos + self.capacity - len) % self.capacity
        } else {
            // 数据不够，从最老的数据开始读
            self.read_pos
        };

        // 复制数据
        for (i, sample) in output.iter_mut().enumerate() {
            if i < to_read {
                let pos = (read_start + i) % self.capacity;
                *sample = self.buffer[pos];
            } else {
                *sample = 0.0;
            }
        }
    }

    /// 清除缓冲区
    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.read_pos = 0;
        self.available = 0;
        self.buffer.fill(0.0);
    }
}

/// 全局音频状态（线程安全）
pub struct GlobalAudioState {
    /// 音量配置（使用 atomic 避免锁）
    vocal_volume: std::sync::atomic::AtomicU32,
    instrument_volume: std::sync::atomic::AtomicU32,
    monitor_volume: std::sync::atomic::AtomicU32,
    effect_input: Mutex<EffectInput>,
    vocal_channel: std::sync::atomic::AtomicU32,
    instrument_channel: std::sync::atomic::AtomicU32,
    /// 录音器
    pub recorder: Mutex<DualTrackRecorder>,
    /// 运行状态
    pub is_running: AtomicBool,
    /// 效果器旁通状态（true = 旁通，output = input）
    pub effect_bypass: AtomicBool,
    /// 原始音频缓冲区（输入写入，供处理读取）
    pub vocal_buffer: Mutex<AudioBuffer>,
    pub instrument_buffer: Mutex<AudioBuffer>,
    /// 处理后的音频缓冲区（效果器输出，供所有输出流读取）
    pub processed_buffer: Mutex<AudioBuffer>,
    /// 效果器链
    pub effect_chain: Mutex<EffectChain>,
    /// 输出电平表（常驻在效果器链最后）
    pub output_level_meter: Mutex<crate::modules::effects::LevelMeterProcessor>,
    /// 采样率
    pub sample_rate: std::sync::atomic::AtomicU32,
    /// 帧计数器（用于调试日志）
    frame_counter: AtomicU64,
    /// Ducking 状态
    pub ducking_enabled: AtomicBool,
    pub ducking_threshold: std::sync::atomic::AtomicU32,
    pub ducking_ratio: std::sync::atomic::AtomicU32,
    pub is_ducking: AtomicBool,
    /// Ducking 恢复延迟（秒，1-9秒）
    pub ducking_recovery_delay: std::sync::atomic::AtomicU32,
    /// Ducking 停止计时开始时间（用于延迟恢复）
    pub ducking_release_start: std::sync::atomic::AtomicU64,
    /// 过场音乐管理器引用（用于 ducking）
    pub interlude_manager: Mutex<Option<std::sync::Arc<Mutex<crate::modules::interlude::InterludeManager>>>>,
    /// 当前输入电平（用于 ducking 检测，避免在音频回调中获取锁）
    pub current_input_level: std::sync::atomic::AtomicU32,
    /// Ducking 检测线程停止信号
    pub ducking_stop_signal: std::sync::atomic::AtomicBool,
}

impl GlobalAudioState {
    pub fn new() -> Self {
        Self {
            vocal_volume: std::sync::atomic::AtomicU32::new(float_to_u32(0.8)),
            instrument_volume: std::sync::atomic::AtomicU32::new(float_to_u32(0.8)),
            monitor_volume: std::sync::atomic::AtomicU32::new(float_to_u32(0.8)),
            effect_input: Mutex::new(EffectInput::Vocal),
            vocal_channel: std::sync::atomic::AtomicU32::new(0),
            instrument_channel: std::sync::atomic::AtomicU32::new(1),
            recorder: Mutex::new(DualTrackRecorder::new()),
            is_running: AtomicBool::new(false),
            effect_bypass: AtomicBool::new(true), // 默认旁通
            vocal_buffer: Mutex::new(AudioBuffer::new(65536)),
            instrument_buffer: Mutex::new(AudioBuffer::new(65536)),
            processed_buffer: Mutex::new(AudioBuffer::new(65536)),
            effect_chain: Mutex::new(EffectChain::new()),
            output_level_meter: Mutex::new(crate::modules::effects::LevelMeterProcessor::new(44100.0)),
            sample_rate: std::sync::atomic::AtomicU32::new(44100),
            frame_counter: AtomicU64::new(0),
            ducking_enabled: AtomicBool::new(true),
            ducking_threshold: std::sync::atomic::AtomicU32::new(float_to_u32(0.01)),
            ducking_ratio: std::sync::atomic::AtomicU32::new(float_to_u32(0.1)),
            is_ducking: AtomicBool::new(false),
            ducking_recovery_delay: std::sync::atomic::AtomicU32::new(3), // 默认3秒
            ducking_release_start: std::sync::atomic::AtomicU64::new(0),
            interlude_manager: Mutex::new(None),
            current_input_level: std::sync::atomic::AtomicU32::new(0),
            ducking_stop_signal: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn set_effect_bypass(&self, bypass: bool) {
        self.effect_bypass.store(bypass, Ordering::SeqCst);
        println!("[GlobalState] Effect bypass set to: {}", bypass);
    }

    pub fn is_effect_bypass(&self) -> bool {
        self.effect_bypass.load(Ordering::SeqCst)
    }

    pub fn set_vocal_volume(&self, v: f32) {
        self.vocal_volume.store(float_to_u32(v), Ordering::Relaxed);
    }

    pub fn get_vocal_volume(&self) -> f32 {
        u32_to_float(self.vocal_volume.load(Ordering::Relaxed))
    }

    pub fn set_instrument_volume(&self, v: f32) {
        self.instrument_volume.store(float_to_u32(v), Ordering::Relaxed);
    }

    pub fn get_instrument_volume(&self) -> f32 {
        u32_to_float(self.instrument_volume.load(Ordering::Relaxed))
    }

    pub fn set_monitor_volume(&self, v: f32) {
        self.monitor_volume.store(float_to_u32(v), Ordering::Relaxed);
    }

    pub fn get_monitor_volume(&self) -> f32 {
        u32_to_float(self.monitor_volume.load(Ordering::Relaxed))
    }

    pub fn set_effect_input(&self, v: EffectInput) {
        if let Ok(mut guard) = self.effect_input.lock() {
            *guard = v;
        }
    }

    pub fn get_effect_input(&self) -> EffectInput {
        self.effect_input.lock().map(|g| *g).unwrap_or(EffectInput::Vocal)
    }

    pub fn set_vocal_channel(&self, ch: usize) {
        self.vocal_channel.store(ch as u32, Ordering::Relaxed);
    }

    pub fn get_vocal_channel(&self) -> usize {
        self.vocal_channel.load(Ordering::Relaxed) as usize
    }

    pub fn set_instrument_channel(&self, ch: usize) {
        self.instrument_channel.store(ch as u32, Ordering::Relaxed);
    }

    pub fn get_instrument_channel(&self) -> usize {
        self.instrument_channel.load(Ordering::Relaxed) as usize
    }

    pub fn set_sample_rate(&self, rate: u32) {
        self.sample_rate.store(rate, Ordering::Relaxed);
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::Relaxed)
    }

    /// 设置过场音乐管理器引用（用于 ducking）
    pub fn set_interlude_manager(&self, manager: std::sync::Arc<Mutex<crate::modules::interlude::InterludeManager>>) {
        if let Ok(mut guard) = self.interlude_manager.lock() {
            *guard = Some(manager);
        }
    }

    /// 启动 ducking 检测线程
    pub fn start_ducking_thread(self: Arc<Self>) {
        // 重置停止信号
        self.ducking_stop_signal.store(false, Ordering::Relaxed);

        let state = self.clone();
        std::thread::spawn(move || {
            println!("[DuckingThread] Started");
            while !state.ducking_stop_signal.load(Ordering::Relaxed) {
                state.check_ducking();
                // 每 50ms 检查一次
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            println!("[DuckingThread] Stopped");
        });
    }

    /// 停止 ducking 检测线程
    pub fn stop_ducking_thread(&self) {
        self.ducking_stop_signal.store(true, Ordering::Relaxed);
    }

    /// 设置 ducking 参数
    pub fn set_ducking_params(&self, enabled: bool, threshold: f32, ratio: f32, recovery_delay: u32) {
        println!("[GlobalState] set_ducking_params: enabled={}, threshold={}, ratio={}, recovery_delay={}", enabled, threshold, ratio, recovery_delay);
        self.ducking_enabled.store(enabled, Ordering::Relaxed);
        self.ducking_threshold.store(float_to_u32(threshold), Ordering::Relaxed);
        self.ducking_ratio.store(float_to_u32(ratio), Ordering::Relaxed);
        self.ducking_recovery_delay.store(recovery_delay.clamp(1, 9), Ordering::Relaxed);
    }

    /// 更新输入电平（在音频回调中调用，无锁）
    pub fn update_input_level(&self, level: f32) {
        self.current_input_level.store(float_to_u32(level), Ordering::Relaxed);
    }

    /// 检查并应用 ducking（根据输入电平）
    /// 使用 try_lock 避免阻塞，只在过场音乐播放期间进行 ducking 检测
    /// 这个方法应该从单独的线程定期调用，而不是从音频回调中调用
    pub fn check_ducking(&self) {
        if !self.ducking_enabled.load(Ordering::Relaxed) {
            return;
        }

        // 从原子变量读取输入电平
        let input_level = u32_to_float(self.current_input_level.load(Ordering::Relaxed));

        // 使用 try_lock 检查过场音乐是否正在播放，避免阻塞
        let interlude_playing = if let Ok(guard) = self.interlude_manager.try_lock() {
            if let Some(ref manager) = *guard {
                if let Ok(mgr) = manager.try_lock() {
                    mgr.get_state().is_playing
                } else {
                    // 无法获取锁，保持当前状态
                    return;
                }
            } else {
                false
            }
        } else {
            // 无法获取锁，直接返回，避免阻塞
            return;
        };

        // 过场音乐未播放时，不进行 ducking 检测
        if !interlude_playing {
            // 如果正在 ducking 状态，需要恢复
            if self.is_ducking.load(Ordering::Relaxed) {
                self.is_ducking.store(false, Ordering::Relaxed);
                self.ducking_release_start.store(0, Ordering::Relaxed);
            }
            return;
        }

        let threshold = u32_to_float(self.ducking_threshold.load(Ordering::Relaxed));
        let ratio = u32_to_float(self.ducking_ratio.load(Ordering::Relaxed));
        let recovery_delay = self.ducking_recovery_delay.load(Ordering::Relaxed) as u64;
        let was_ducking = self.is_ducking.load(Ordering::Relaxed);

        if input_level > threshold {
            // 输入超过阈值
            if !was_ducking {
                // 开始 ducking
                println!("[Ducking] Starting ducking! input_level={:.4} > threshold={:.4}", input_level, threshold);
                self.is_ducking.store(true, Ordering::Relaxed);
                // 使用 try_lock 避免阻塞
                if let Ok(guard) = self.interlude_manager.try_lock() {
                    if let Some(ref manager) = *guard {
                        if let Ok(mut mgr) = manager.try_lock() {
                            mgr.apply_ducking(ratio);
                        }
                    }
                }
            } else {
                // 重置释放计时（持续有声音）
                let prev_release = self.ducking_release_start.swap(0, Ordering::Relaxed);
                if prev_release != 0 {
                    println!("[Ducking] Resetting release timer (was {})", prev_release);
                }
            }
        } else if was_ducking {
            // 输入低于阈值，但正在 ducking
            let release_start = self.ducking_release_start.load(Ordering::Relaxed);

            if release_start == 0 {
                // 第一次检测到低于阈值，开始计时
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                println!("[Ducking] Starting release timer at {} (delay={}s)", now, recovery_delay);
                self.ducking_release_start.store(now, Ordering::Relaxed);
            } else {
                // 检查是否已过延迟时间
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let elapsed = now - release_start;
                if elapsed >= recovery_delay {
                    // 延迟时间已到，停止 ducking
                    println!("[Ducking] Release delay reached! elapsed={}s >= {}s", elapsed, recovery_delay);
                    self.is_ducking.store(false, Ordering::Relaxed);
                    self.ducking_release_start.store(0, Ordering::Relaxed);
                    // 使用 try_lock 避免阻塞
                    if let Ok(guard) = self.interlude_manager.try_lock() {
                        if let Some(ref manager) = *guard {
                            if let Ok(mut mgr) = manager.try_lock() {
                                mgr.release_ducking();
                            }
                        }
                    }
                }
            }
        }
    }

    /// 获取 ducking 调试状态
    pub fn get_ducking_debug_state(&self) -> DuckingDebugState {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let release_start = self.ducking_release_start.load(Ordering::Relaxed);
        let recovery_delay = self.ducking_recovery_delay.load(Ordering::Relaxed) as u64;

        // 检查过场音乐是否正在播放
        let interlude_playing = if let Ok(guard) = self.interlude_manager.lock() {
            if let Some(ref manager) = *guard {
                if let Ok(mgr) = manager.lock() {
                    mgr.get_state().is_playing
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        DuckingDebugState {
            enabled: self.ducking_enabled.load(Ordering::Relaxed),
            interlude_playing,
            is_ducking: self.is_ducking.load(Ordering::Relaxed),
            threshold: u32_to_float(self.ducking_threshold.load(Ordering::Relaxed)),
            ratio: u32_to_float(self.ducking_ratio.load(Ordering::Relaxed)),
            recovery_delay,
            release_start,
            elapsed_since_release_start: if release_start > 0 { now - release_start } else { 0 },
            remaining_time: if release_start > 0 {
                let elapsed = now - release_start;
                if elapsed < recovery_delay { recovery_delay - elapsed } else { 0 }
            } else {
                0
            },
        }
    }
}

/// Ducking 调试状态
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuckingDebugState {
    pub enabled: bool,
    pub interlude_playing: bool,
    pub is_ducking: bool,
    pub threshold: f32,
    pub ratio: f32,
    pub recovery_delay: u64,
    pub release_start: u64,
    pub elapsed_since_release_start: u64,
    pub remaining_time: u64,
}

impl GlobalAudioState {
    /// 更新效果器链配置
    /// 使用双缓冲策略：先在锁外创建新链，然后快速交换
    pub fn update_effect_chain(&self, slots: &[(i32, String, bool, String)]) {
        // 先在锁外创建新的效果器链，避免在锁内执行耗时操作
        let sample_rate = self.get_sample_rate() as f32;
        let mut new_chain = EffectChain::new();

        eprintln!("[EffectChain] ====== UPDATE CALLED ======");
        eprintln!("[EffectChain] Loading {} effect slots, sample_rate={}", slots.len(), sample_rate);

        for (slot_index, effect_type, enabled, parameters) in slots {
            if let Some(e_type) = effect_type_from_str(effect_type) {
                let mut processor = create_processor(e_type, sample_rate);

                // 应用参数
                if let Ok(params) = serde_json::from_str::<serde_json::Value>(parameters) {
                    apply_parameters(&mut *processor, &params);
                }

                eprintln!("[EffectChain] Added effect: {} (enabled={}, slot_index={})", effect_type, enabled, slot_index);
                let idx = new_chain.len();
                new_chain.add_processor_with_slot_index(processor, *slot_index);
                if !*enabled {
                    new_chain.set_enabled(idx, false);
                }
            }
        }

        eprintln!("[EffectChain] New chain ready with {} effects, swapping...", new_chain.len());

        // 快速获取锁并交换链（最小化锁持有时间）
        if let Ok(mut chain) = self.effect_chain.lock() {
            // 使用 std::mem::swap 来交换，而不是清空重建
            std::mem::swap(&mut *chain, &mut new_chain);
            eprintln!("[EffectChain] Swap complete, new chain has {} effects", chain.len());
        } else {
            eprintln!("[EffectChain] ERROR: Failed to acquire lock!");
        }
        // new_chain 在这里被 drop，这是旧的链，不阻塞音频处理
    }
}

/// 递归应用 JSON 参数到效果器
fn apply_parameters(processor: &mut dyn crate::modules::effects::AudioProcessor, params: &serde_json::Value) {
    match params {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(f) = n.as_f64() {
                            processor.set_parameter(key, f as f32);
                        }
                    }
                    serde_json::Value::Bool(b) => {
                        processor.set_parameter(key, if *b { 1.0 } else { 0.0 });
                    }
                    serde_json::Value::Object(_) => {
                        // 嵌套对象，如 EQ 的频段
                        let prefix = format!("{}.", key);
                        apply_parameters_with_prefix(processor, value, &prefix);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn apply_parameters_with_prefix(processor: &mut dyn crate::modules::effects::AudioProcessor, params: &serde_json::Value, prefix: &str) {
    match params {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                let full_key = format!("{}{}", prefix, key);
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(f) = n.as_f64() {
                            processor.set_parameter(&full_key, f as f32);
                        }
                    }
                    serde_json::Value::Bool(b) => {
                        processor.set_parameter(&full_key, if *b { 1.0 } else { 0.0 });
                    }
                    serde_json::Value::Object(_) => {
                        let new_prefix = format!("{}.", full_key);
                        apply_parameters_with_prefix(processor, value, &new_prefix);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

impl Default for GlobalAudioState {
    fn default() -> Self {
        Self::new()
    }
}

// 辅助函数：f32 <-> u32 转换（用于 atomic 存储）
fn float_to_u32(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * 10000.0) as u32
}

fn u32_to_float(v: u32) -> f32 {
    v as f32 / 10000.0
}

/// 实时音频管理器
pub struct LiveAudioManager {
    host: cpal::Host,
    pub global_state: Arc<GlobalAudioState>,
    pub sample_rate: u32,
}

impl LiveAudioManager {
    pub fn new(global_state: Arc<GlobalAudioState>) -> Self {
        Self {
            host: cpal::default_host(),
            global_state,
            sample_rate: 44100,
        }
    }

    /// 获取所有输入设备信息
    pub fn list_input_devices(&self) -> Vec<DeviceInfo> {
        let mut devices = Vec::new();
        let default_name = self.host.default_input_device()
            .and_then(|d| d.name().ok());

        if let Ok(input_devices) = self.host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    if let Ok(config) = device.default_input_config() {
                        println!("[LiveRouter] Found input device: {} ({}ch, {}Hz)",
                            name, config.channels(), config.sample_rate().0);
                        devices.push(DeviceInfo {
                            name: name.clone(),
                            channels: config.channels(),
                            sample_rate: config.sample_rate().0,
                            is_default: default_name.as_ref().map(|n| n == &name).unwrap_or(false),
                        });
                    }
                }
            }
        }
        println!("[LiveRouter] Total input devices: {}", devices.len());
        devices
    }

    /// 获取所有输出设备信息
    pub fn list_output_devices(&self) -> Vec<DeviceInfo> {
        let mut devices = Vec::new();
        let default_name = self.host.default_output_device()
            .and_then(|d| d.name().ok());

        if let Ok(output_devices) = self.host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    if let Ok(config) = device.default_output_config() {
                        println!("[LiveRouter] Found output device: {} ({}ch, {}Hz)",
                            name, config.channels(), config.sample_rate().0);
                        devices.push(DeviceInfo {
                            name: name.clone(),
                            channels: config.channels(),
                            sample_rate: config.sample_rate().0,
                            is_default: default_name.as_ref().map(|n| n == &name).unwrap_or(false),
                        });
                    }
                }
            }
        }
        println!("[LiveRouter] Total output devices: {}", devices.len());
        devices
    }

    fn find_device_by_name(&self, name: &str, is_input: bool) -> Option<Device> {
        let devices = if is_input {
            self.host.input_devices().ok()?
        } else {
            self.host.output_devices().ok()?
        };

        for device in devices {
            if let Ok(device_name) = device.name() {
                if &device_name == name {
                    return Some(device);
                }
            }
        }
        None
    }

    fn get_default_input_device(&self) -> Option<Device> {
        self.host.default_input_device()
    }

    fn get_default_output_device(&self) -> Option<Device> {
        self.host.default_output_device()
    }

    /// 启动实时音频
    pub fn start(&mut self, config: LiveAudioConfig) -> Result<AudioStreams, String> {
        // 使用 compare_exchange 来原子地检查并设置 is_running
        // 这可以防止竞态条件（比如 React StrictMode 导致的双重调用）
        if self.global_state.is_running.compare_exchange(
            false, true, Ordering::SeqCst, Ordering::SeqCst
        ).is_err() {
            return Err("Audio router is already running".to_string());
        }

        let result = self.start_internal(config);

        if result.is_err() {
            // 如果失败，重置 is_running
            self.global_state.is_running.store(false, Ordering::SeqCst);
            eprintln!("[LiveRouter] Start failed, reset is_running");
        } else {
            // 启动 ducking 检测线程
            self.global_state.clone().start_ducking_thread();
        }

        result
    }

    /// 内部启动逻辑
    fn start_internal(&mut self, config: LiveAudioConfig) -> Result<AudioStreams, String> {
        // 更新全局状态
        self.global_state.set_vocal_volume(config.vocal_volume);
        self.global_state.set_instrument_volume(config.instrument_volume);
        self.global_state.set_monitor_volume(config.monitor_volume);
        self.global_state.set_effect_input(config.effect_input);
        self.global_state.set_vocal_channel(config.vocal_input_channel as usize);
        self.global_state.set_instrument_channel(config.instrument_input_channel as usize);

        println!("[LiveRouter] Starting with config: {:?}", config);

        // 获取设备
        let vocal_device = if let Some(ref name) = config.vocal_input_device {
            self.find_device_by_name(name, true)
                .or_else(|| self.get_default_input_device())
        } else {
            self.get_default_input_device()
        };

        let instrument_device = config.instrument_input_device.as_ref()
            .and_then(|name| self.find_device_by_name(name, true));

        // 监听输出设备 - 如果为空，不创建输出流
        let monitor_device = if config.monitor_output_device.is_empty() {
            println!("[LiveRouter] No monitor device specified, will not create output stream");
            None
        } else {
            self.find_device_by_name(&config.monitor_output_device, false)
                .or_else(|| {
                    println!("[LiveRouter] Monitor device not found");
                    None
                })
        };

        // 直播输出设备 - 只有明确指定时才创建
        let stream_device = config.stream_output_device.as_ref()
            .filter(|name| !name.is_empty())
            .and_then(|name| self.find_device_by_name(name, false));

        // 检查设备名称
        let vocal_device_name = vocal_device.as_ref().and_then(|d| d.name().ok());
        let monitor_device_name = monitor_device.as_ref().and_then(|d| d.name().ok());
        let stream_device_name = stream_device.as_ref().and_then(|d| d.name().ok());

        println!("[LiveRouter] Devices - Vocal: {:?}, Monitor: {:?}, Stream: {:?}",
            vocal_device_name, monitor_device_name, stream_device_name);

        // 检查是否是同一个设备
        let same_output_device = monitor_device_name.is_some() && monitor_device_name == stream_device_name;

        let mut streams = AudioStreams::new();

        // 创建人声输入流
        if let Some(device) = vocal_device {
            if let Ok(dev_config) = device.default_input_config() {
                self.sample_rate = dev_config.sample_rate().0;
                let channels = dev_config.channels();
                println!("[LiveRouter] Vocal input: rate={}, channels={}", self.sample_rate, channels);

                if let Ok(stream) = self.create_input_stream(&device, dev_config, true) {
                    streams.vocal_input = Some(stream);
                }
            }
        }

        // 创建乐器输入流
        if let Some(device) = instrument_device {
            if let Ok(dev_config) = device.default_input_config() {
                println!("[LiveRouter] Instrument input: rate={}, channels={}",
                    dev_config.sample_rate().0, dev_config.channels());

                if let Ok(stream) = self.create_input_stream(&device, dev_config, false) {
                    streams.instrument_input = Some(stream);
                }
            }
        }

        // 创建监听输出流
        if let Some(device) = monitor_device {
            if let Ok(dev_config) = device.default_output_config() {
                println!("[LiveRouter] Monitor output: rate={}, channels={}",
                    dev_config.sample_rate().0, dev_config.channels());

                if let Ok(stream) = self.create_output_stream(&device, dev_config) {
                    streams.monitor_output = Some(stream);
                }
            }
        }

        // 创建直播输出流（如果与监听设备不同）
        if let Some(device) = stream_device {
            if !same_output_device {
                if let Ok(dev_config) = device.default_output_config() {
                    println!("[LiveRouter] Stream output: rate={}, channels={}",
                        dev_config.sample_rate().0, dev_config.channels());
                    if let Ok(stream) = self.create_output_stream(&device, dev_config) {
                        streams.stream_output = Some(stream);
                    }
                }
            } else {
                println!("[LiveRouter] Stream output same as monitor, skipping");
            }
        }

        // 设置采样率并初始化效果器链
        self.global_state.set_sample_rate(self.sample_rate);

        println!("[LiveRouter] Started successfully");

        Ok(streams)
    }

    fn create_input_stream(
        &self,
        device: &Device,
        config: SupportedStreamConfig,
        is_vocal: bool,
    ) -> Result<Stream, String> {
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();
        let channels = stream_config.channels;

        let err_fn = |err| eprintln!("[LiveRouter] Input stream error: {}", err);
        let state = Arc::clone(&self.global_state);

        let stream = match sample_format {
            SampleFormat::F32 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        process_input_f32(data, channels, &state, is_vocal);
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::I16 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        process_input_i16(data, channels, &state, is_vocal);
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        process_input_u16(data, channels, &state, is_vocal);
                    },
                    err_fn,
                    None,
                )
            }
            _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
        };

        let stream = stream.map_err(|e| format!("Failed to create input stream: {}", e))?;
        stream.play().map_err(|e| format!("Failed to start input stream: {}", e))?;

        Ok(stream)
    }

    fn create_output_stream(
        &self,
        device: &Device,
        config: SupportedStreamConfig,
    ) -> Result<Stream, String> {
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();
        let output_channels = stream_config.channels;

        let err_fn = |err| eprintln!("[LiveRouter] Output stream error: {}", err);
        let state = Arc::clone(&self.global_state);

        let stream = match sample_format {
            SampleFormat::F32 => {
                device.build_output_stream(
                    &stream_config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        process_output_f32(data, output_channels, &state);
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::I16 => {
                device.build_output_stream(
                    &stream_config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        process_output_i16(data, output_channels, &state);
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                device.build_output_stream(
                    &stream_config,
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        process_output_u16(data, output_channels, &state);
                    },
                    err_fn,
                    None,
                )
            }
            _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
        };

        let stream = stream.map_err(|e| format!("Failed to create output stream: {}", e))?;
        stream.play().map_err(|e| format!("Failed to start output stream: {}", e))?;

        Ok(stream)
    }

    /// 开始录音
    pub fn start_recording(&self, vocal_path: Option<PathBuf>, instrument_path: Option<PathBuf>) -> Result<(), String> {
        let mut recorder = self.global_state.recorder.lock().unwrap();
        recorder.start_recording(vocal_path, instrument_path, self.sample_rate, 2)
    }

    /// 停止录音
    pub fn stop_recording(&self) -> Result<(Option<PathBuf>, Option<PathBuf>), String> {
        let mut recorder = self.global_state.recorder.lock().unwrap();
        recorder.stop_recording()
    }

    pub fn is_recording(&self) -> bool {
        self.global_state.recorder.lock().unwrap().is_recording()
    }

    pub fn get_state(&self) -> LiveAudioState {
        let recorder = self.global_state.recorder.lock().unwrap();
        LiveAudioState {
            is_running: self.global_state.is_running.load(Ordering::SeqCst),
            config: LiveAudioConfig {
                vocal_input_device: None,
                vocal_input_channel: self.global_state.get_vocal_channel() as i32,
                instrument_input_device: None,
                instrument_input_channel: self.global_state.get_instrument_channel() as i32,
                monitor_output_device: String::new(),
                stream_output_device: None,
                vocal_volume: self.global_state.get_vocal_volume(),
                instrument_volume: self.global_state.get_instrument_volume(),
                effect_input: self.global_state.get_effect_input(),
                monitor_volume: self.global_state.get_monitor_volume(),
                stream_volume: 1.0,
            },
            vocal_recording: recorder.get_vocal_state().is_recording,
            instrument_recording: recorder.get_instrument_state().is_recording,
        }
    }
}

// ============ 输入处理 ============

fn process_input_f32(data: &[f32], input_channels: u16, state: &GlobalAudioState, is_vocal: bool) {
    let volume = if is_vocal {
        state.get_vocal_volume()
    } else {
        state.get_instrument_volume()
    };
    let target_channel = if is_vocal {
        state.get_vocal_channel()
    } else {
        state.get_instrument_channel()
    };

    // 提取目标通道的样本（交错格式: L0, R0, L1, R1, ...）
    let frames = data.len() / input_channels as usize;
    let mut mono_samples = Vec::with_capacity(frames);

    for frame_idx in 0..frames {
        let sample = if target_channel < input_channels as usize {
            // 选择指定通道
            data[frame_idx * input_channels as usize + target_channel]
        } else {
            // 混合所有通道
            let mut sum = 0.0;
            for ch in 0..input_channels as usize {
                sum += data[frame_idx * input_channels as usize + ch];
            }
            sum / input_channels as f32
        };
        mono_samples.push(sample * volume);
    }

    // 写入缓冲区供输出读取
    if is_vocal {
        if let Ok(mut buffer) = state.vocal_buffer.lock() {
            buffer.write(&mono_samples);
        }

        // 更新输入电平（用于 ducking 检测，无锁）
        if mono_samples.len() > 0 {
            let max_level = mono_samples.iter().map(|s| s.abs()).fold(0.0f32, |a, b| a.max(b));
            state.update_input_level(max_level);
        }
    } else {
        if let Ok(mut buffer) = state.instrument_buffer.lock() {
            buffer.write(&mono_samples);
        }
    }

    // 录音
    if let Ok(mut recorder) = state.recorder.lock() {
        if recorder.is_recording() {
            if is_vocal {
                let _ = recorder.write_vocal_samples(&mono_samples);
            } else {
                let _ = recorder.write_instrument_samples(&mono_samples);
            }
        }
    }

    // 处理效果器链并写入处理后的缓冲区
    // 只有选中的输入源会经过效果器处理
    let effect_input = state.get_effect_input();
    let should_process = (is_vocal && effect_input == EffectInput::Vocal)
        || (!is_vocal && effect_input == EffectInput::Instrument);

    if should_process {
        let processed = if state.is_effect_bypass() {
            // 旁通模式：直接输出输入
            mono_samples.clone()
        } else {
            // 正常模式：经过效果器处理
            let mut processed = mono_samples.clone();

            // 应用效果器链
            if let Ok(mut chain) = state.effect_chain.lock() {
                if chain.len() > 0 {
                    let mut output_samples = vec![0.0f32; processed.len()];
                    chain.process(&processed, &mut output_samples);
                    processed = output_samples;
                }
            }

            processed
        };

        // 测量输出电平
        if let Ok(mut meter) = state.output_level_meter.lock() {
            let mut meter_output = vec![0.0f32; processed.len()];
            meter.process(&processed, &mut meter_output);
        }

        // 写入处理后的缓冲区
        if let Ok(mut buffer) = state.processed_buffer.lock() {
            buffer.write(&processed);
        }
    }
}

fn process_input_i16(data: &[i16], input_channels: u16, state: &GlobalAudioState, is_vocal: bool) {
    let volume = if is_vocal {
        state.get_vocal_volume()
    } else {
        state.get_instrument_volume()
    };
    let target_channel = if is_vocal {
        state.get_vocal_channel()
    } else {
        state.get_instrument_channel()
    };

    let frames = data.len() / input_channels as usize;
    let mut mono_samples = Vec::with_capacity(frames);

    for frame_idx in 0..frames {
        let sample = if target_channel < input_channels as usize {
            data[frame_idx * input_channels as usize + target_channel] as f32 / 32767.0
        } else {
            let mut sum = 0.0;
            for ch in 0..input_channels as usize {
                sum += data[frame_idx * input_channels as usize + ch] as f32 / 32767.0;
            }
            sum / input_channels as f32
        };
        mono_samples.push(sample * volume);
    }

    // 写入缓冲区
    if is_vocal {
        if let Ok(mut buffer) = state.vocal_buffer.lock() {
            buffer.write(&mono_samples);
        }

        // 更新输入电平（用于 ducking 检测，无锁）
        if mono_samples.len() > 0 {
            let max_level = mono_samples.iter().map(|s| s.abs()).fold(0.0f32, |a, b| a.max(b));
            state.update_input_level(max_level);
        }
    } else {
        if let Ok(mut buffer) = state.instrument_buffer.lock() {
            buffer.write(&mono_samples);
        }
    }

    // 录音
    if let Ok(mut recorder) = state.recorder.lock() {
        if recorder.is_recording() {
            if is_vocal {
                let _ = recorder.write_vocal_samples(&mono_samples);
            } else {
                let _ = recorder.write_instrument_samples(&mono_samples);
            }
        }
    }

    // 处理效果器链
    let effect_input = state.get_effect_input();
    let should_process = (is_vocal && effect_input == EffectInput::Vocal)
        || (!is_vocal && effect_input == EffectInput::Instrument);

    if should_process {
        let processed = if state.is_effect_bypass() {
            // 旁通模式：直接输出输入
            mono_samples.clone()
        } else {
            // 正常模式：经过效果器处理
            let mut processed = mono_samples.clone();
            if let Ok(mut chain) = state.effect_chain.lock() {
                if chain.len() > 0 {
                    let mut output_samples = vec![0.0f32; processed.len()];
                    chain.process(&processed, &mut output_samples);
                    processed = output_samples;
                }
            }
            processed
        };

        // 测量输出电平
        if let Ok(mut meter) = state.output_level_meter.lock() {
            let mut meter_output = vec![0.0f32; processed.len()];
            meter.process(&processed, &mut meter_output);
        }

        if let Ok(mut buffer) = state.processed_buffer.lock() {
            buffer.write(&processed);
        }
    }
}

fn process_input_u16(data: &[u16], input_channels: u16, state: &GlobalAudioState, is_vocal: bool) {
    let volume = if is_vocal {
        state.get_vocal_volume()
    } else {
        state.get_instrument_volume()
    };
    let target_channel = if is_vocal {
        state.get_vocal_channel()
    } else {
        state.get_instrument_channel()
    };

    let frames = data.len() / input_channels as usize;
    let mut mono_samples = Vec::with_capacity(frames);

    for frame_idx in 0..frames {
        let sample = if target_channel < input_channels as usize {
            (data[frame_idx * input_channels as usize + target_channel] as f32 - 32768.0) / 32767.0
        } else {
            let mut sum = 0.0;
            for ch in 0..input_channels as usize {
                sum += (data[frame_idx * input_channels as usize + ch] as f32 - 32768.0) / 32767.0;
            }
            sum / input_channels as f32
        };
        mono_samples.push(sample * volume);
    }

    // 写入缓冲区
    if is_vocal {
        if let Ok(mut buffer) = state.vocal_buffer.lock() {
            buffer.write(&mono_samples);
        }

        // 更新输入电平（用于 ducking 检测，无锁）
        if mono_samples.len() > 0 {
            let max_level = mono_samples.iter().map(|s| s.abs()).fold(0.0f32, |a, b| a.max(b));
            state.update_input_level(max_level);
        }
    } else {
        if let Ok(mut buffer) = state.instrument_buffer.lock() {
            buffer.write(&mono_samples);
        }
    }

    // 录音
    if let Ok(mut recorder) = state.recorder.lock() {
        if recorder.is_recording() {
            if is_vocal {
                let _ = recorder.write_vocal_samples(&mono_samples);
            } else {
                let _ = recorder.write_instrument_samples(&mono_samples);
            }
        }
    }

    // 处理效果器链
    let effect_input = state.get_effect_input();
    let should_process = (is_vocal && effect_input == EffectInput::Vocal)
        || (!is_vocal && effect_input == EffectInput::Instrument);

    if should_process {
        let processed = if state.is_effect_bypass() {
            // 旁通模式：直接输出输入
            mono_samples.clone()
        } else {
            // 正常模式：经过效果器处理
            let mut processed = mono_samples.clone();
            if let Ok(mut chain) = state.effect_chain.lock() {
                if chain.len() > 0 {
                    let mut output_samples = vec![0.0f32; processed.len()];
                    chain.process(&processed, &mut output_samples);
                    processed = output_samples;
                }
            }
            processed
        };

        // 测量输出电平
        if let Ok(mut meter) = state.output_level_meter.lock() {
            let mut meter_output = vec![0.0f32; processed.len()];
            meter.process(&processed, &mut meter_output);
        }

        if let Ok(mut buffer) = state.processed_buffer.lock() {
            buffer.write(&processed);
        }
    }
}

// ============ 输出处理 ============

fn process_output_f32(data: &mut [f32], output_channels: u16, state: &GlobalAudioState) {
    let monitor_volume = state.get_monitor_volume();
    let effect_input = state.get_effect_input();

    // 计算需要的单声道样本数
    let mono_count = data.len() / output_channels as usize;

    // 从处理后的缓冲区读取（经过效果器处理的音频）
    let processed_samples: Vec<f32> = {
        if let Ok(buffer) = state.processed_buffer.lock() {
            let mut samples = vec![0.0f32; mono_count];
            buffer.read_latest(&mut samples);
            samples
        } else {
            vec![0.0; mono_count]
        }
    };

    // 从人声缓冲区读取（如果人声不是效果器输入，则直通）
    let vocal_samples: Vec<f32> = if effect_input != EffectInput::Vocal {
        if let Ok(buffer) = state.vocal_buffer.lock() {
            let mut samples = vec![0.0f32; mono_count];
            buffer.read_latest(&mut samples);
            samples
        } else {
            vec![0.0; mono_count]
        }
    } else {
        vec![0.0; mono_count]
    };

    // 从乐器缓冲区读取（如果乐器不是效果器输入，则直通）
    let instrument_samples: Vec<f32> = if effect_input != EffectInput::Instrument {
        if let Ok(buffer) = state.instrument_buffer.lock() {
            let mut samples = vec![0.0f32; mono_count];
            buffer.read_latest(&mut samples);
            samples
        } else {
            vec![0.0; mono_count]
        }
    } else {
        vec![0.0; mono_count]
    };

    // 将单声道数据转换为交错格式的立体声输出
    for i in 0..mono_count {
        // 混合：processed_samples（经过效果器） + 直通的输入
        let mixed = processed_samples[i] + vocal_samples[i] + instrument_samples[i];
        let output_sample = mixed * monitor_volume;

        let base_idx = i * output_channels as usize;
        if base_idx < data.len() {
            data[base_idx] = output_sample;
            if base_idx + 1 < data.len() {
                data[base_idx + 1] = output_sample;
            }
        }
    }
}

fn process_output_i16(data: &mut [i16], output_channels: u16, state: &GlobalAudioState) {
    // 先清零
    for sample in data.iter_mut() {
        *sample = 0;
    }

    let monitor_volume = state.get_monitor_volume();
    let effect_input = state.get_effect_input();
    let mono_count = data.len() / output_channels as usize;

    // 从处理后的缓冲区读取
    let processed_samples: Vec<f32> = {
        if let Ok(buffer) = state.processed_buffer.lock() {
            let mut samples = vec![0.0f32; mono_count];
            buffer.read_latest(&mut samples);
            samples
        } else {
            vec![0.0; mono_count]
        }
    };

    // 从人声缓冲区读取（如果人声不是效果器输入，则直通）
    let vocal_samples: Vec<f32> = if effect_input != EffectInput::Vocal {
        if let Ok(buffer) = state.vocal_buffer.lock() {
            let mut samples = vec![0.0f32; mono_count];
            buffer.read_latest(&mut samples);
            samples
        } else {
            vec![0.0; mono_count]
        }
    } else {
        vec![0.0; mono_count]
    };

    // 从乐器缓冲区读取（如果乐器不是效果器输入，则直通）
    let instrument_samples: Vec<f32> = if effect_input != EffectInput::Instrument {
        if let Ok(buffer) = state.instrument_buffer.lock() {
            let mut samples = vec![0.0f32; mono_count];
            buffer.read_latest(&mut samples);
            samples
        } else {
            vec![0.0; mono_count]
        }
    } else {
        vec![0.0; mono_count]
    };

    // 混合输出
    for i in 0..mono_count {
        let mixed = processed_samples[i] + vocal_samples[i] + instrument_samples[i];
        let output_sample = (mixed * monitor_volume * 32767.0).clamp(-32768.0, 32767.0) as i16;

        let base_idx = i * output_channels as usize;
        if base_idx < data.len() {
            data[base_idx] = output_sample;
            if base_idx + 1 < data.len() {
                data[base_idx + 1] = output_sample;
            }
        }
    }
}

fn process_output_u16(data: &mut [u16], output_channels: u16, state: &GlobalAudioState) {
    // 先清零（u16 静音值为 32768）
    for sample in data.iter_mut() {
        *sample = 32768;
    }

    let monitor_volume = state.get_monitor_volume();
    let effect_input = state.get_effect_input();
    let mono_count = data.len() / output_channels as usize;

    // 从处理后的缓冲区读取
    let processed_samples: Vec<f32> = {
        if let Ok(buffer) = state.processed_buffer.lock() {
            let mut samples = vec![0.0f32; mono_count];
            buffer.read_latest(&mut samples);
            samples
        } else {
            vec![0.0; mono_count]
        }
    };

    // 从人声缓冲区读取（如果人声不是效果器输入，则直通）
    let vocal_samples: Vec<f32> = if effect_input != EffectInput::Vocal {
        if let Ok(buffer) = state.vocal_buffer.lock() {
            let mut samples = vec![0.0f32; mono_count];
            buffer.read_latest(&mut samples);
            samples
        } else {
            vec![0.0; mono_count]
        }
    } else {
        vec![0.0; mono_count]
    };

    // 从乐器缓冲区读取（如果乐器不是效果器输入，则直通）
    let instrument_samples: Vec<f32> = if effect_input != EffectInput::Instrument {
        if let Ok(buffer) = state.instrument_buffer.lock() {
            let mut samples = vec![0.0f32; mono_count];
            buffer.read_latest(&mut samples);
            samples
        } else {
            vec![0.0; mono_count]
        }
    } else {
        vec![0.0; mono_count]
    };

    // 混合输出
    for i in 0..mono_count {
        let mixed = processed_samples[i] + vocal_samples[i] + instrument_samples[i];
        let output_sample = (mixed * monitor_volume * 32767.0 + 32768.0).clamp(0.0, 65535.0) as u16;

        let base_idx = i * output_channels as usize;
        if base_idx < data.len() {
            data[base_idx] = output_sample;
            if base_idx + 1 < data.len() {
                data[base_idx + 1] = output_sample;
            }
        }
    }
}
