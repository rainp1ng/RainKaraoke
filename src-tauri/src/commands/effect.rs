use tauri::State;
use crate::db::Database;
use crate::models::{EffectSlot, EffectChainConfig, SetEffectSlot, UpdateEffectParameters, EffectPreset, NewEffectPreset, get_default_parameters};
use crate::modules::audio_router::{LiveAudioConfig, LiveAudioState, EffectInput, GlobalAudioState, LiveAudioManager, AudioStreams, DeviceInfo};
use crate::modules::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

// 使用 thread-local 存储来管理音频流
thread_local! {
    static AUDIO_STREAMS: RefCell<Option<AudioStreams>> = RefCell::new(None);
}

/// 全局音频状态
pub struct AppAudioState {
    pub global: Arc<GlobalAudioState>,
}

impl AppAudioState {
    pub fn new() -> Self {
        Self {
            global: Arc::new(GlobalAudioState::new()),
        }
    }
}

impl Default for AppAudioState {
    fn default() -> Self {
        Self::new()
    }
}

/// 从数据库加载效果器槽位并更新效果器链
fn reload_effect_chain(db: &State<Database>, state: &Arc<GlobalAudioState>) {
    println!("[reload_effect_chain] ====== CALLED ======");

    let conn = crate::db::get_connection(db);

    let slots: Vec<(i32, String, bool, String)> = conn
        .prepare("SELECT slot_index, effect_type, is_enabled, parameters FROM effect_slots ORDER BY slot_index")
        .and_then(|mut stmt| {
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i32>(2)? == 1,
                    row.get::<_, String>(3)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    state.update_effect_chain(&slots);
}

// ============ 效果器链配置 ============

#[tauri::command]
pub fn get_effect_chain_config(db: State<Database>) -> Result<EffectChainConfig, String> {
    let conn = crate::db::get_connection(&db);

    let config = conn.query_row(
        "SELECT input_device_id, input_volume, monitor_device_id, stream_device_id, \
         monitor_volume, stream_volume, bypass_all, \
         COALESCE(vocal_input_device, NULL), COALESCE(instrument_input_device, NULL), \
         COALESCE(vocal_input_channel, 0), COALESCE(instrument_input_channel, 1), \
         COALESCE(vocal_volume, 0.8), COALESCE(instrument_volume, 0.8), \
         COALESCE(effect_input, 'vocal'), COALESCE(recording_path, NULL) \
         FROM effect_chain_config WHERE id = 1",
        [],
        |row| {
            Ok(EffectChainConfig {
                input_device_id: row.get(0)?,
                input_volume: row.get(1)?,
                monitor_device_id: row.get(2)?,
                stream_device_id: row.get(3)?,
                monitor_volume: row.get(4)?,
                stream_volume: row.get(5)?,
                bypass_all: row.get::<_, i32>(6)? == 1,
                vocal_input_device: row.get(7)?,
                instrument_input_device: row.get(8)?,
                vocal_input_channel: row.get(9)?,
                instrument_input_channel: row.get(10)?,
                vocal_volume: row.get::<_, f32>(11)?,
                instrument_volume: row.get::<_, f32>(12)?,
                effect_input: row.get::<_, String>(13)?,
                recording_path: row.get(14)?,
            })
        },
    )
    .map_err(|e| e.to_string())?;

    println!("[Command] get_effect_chain_config: {:?}", config);
    Ok(config)
}

/// 更新效果器链配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEffectChainConfig {
    pub input_device_id: Option<String>,
    pub input_volume: Option<f32>,
    pub monitor_device_id: Option<String>,
    pub stream_device_id: Option<String>,
    pub monitor_volume: Option<f32>,
    pub stream_volume: Option<f32>,
    pub bypass_all: Option<bool>,
    pub vocal_input_device: Option<String>,
    pub instrument_input_device: Option<String>,
    pub vocal_input_channel: Option<i32>,
    pub instrument_input_channel: Option<i32>,
    pub vocal_volume: Option<f32>,
    pub instrument_volume: Option<f32>,
    pub effect_input: Option<String>,
    pub recording_path: Option<String>,
}

#[tauri::command]
pub fn save_effect_chain_config(
    db: State<Database>,
    config: UpdateEffectChainConfig,
) -> Result<bool, String> {
    println!("[Command] save_effect_chain_config: {:?}", config);
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE effect_chain_config SET \
         input_device_id = COALESCE(?1, input_device_id), \
         input_volume = COALESCE(?2, input_volume), \
         monitor_device_id = COALESCE(?3, monitor_device_id), \
         stream_device_id = COALESCE(?4, stream_device_id), \
         monitor_volume = COALESCE(?5, monitor_volume), \
         stream_volume = COALESCE(?6, stream_volume), \
         bypass_all = COALESCE(?7, bypass_all), \
         vocal_input_device = COALESCE(?8, vocal_input_device), \
         instrument_input_device = COALESCE(?9, instrument_input_device), \
         vocal_input_channel = COALESCE(?10, vocal_input_channel), \
         instrument_input_channel = COALESCE(?11, instrument_input_channel), \
         vocal_volume = COALESCE(?12, vocal_volume), \
         instrument_volume = COALESCE(?13, instrument_volume), \
         effect_input = COALESCE(?14, effect_input), \
         recording_path = COALESCE(?15, recording_path), \
         updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        rusqlite::params![
            config.input_device_id,
            config.input_volume,
            config.monitor_device_id,
            config.stream_device_id,
            config.monitor_volume,
            config.stream_volume,
            config.bypass_all.map(|b| b as i32),
            config.vocal_input_device,
            config.instrument_input_device,
            config.vocal_input_channel,
            config.instrument_input_channel,
            config.vocal_volume,
            config.instrument_volume,
            config.effect_input,
            config.recording_path,
        ],
    )
    .map_err(|e| e.to_string())?;

    println!("[Command] save_effect_chain_config: saved successfully");
    Ok(true)
}

// ============ 效果器槽位 ============

#[tauri::command]
pub fn get_effect_slots(db: State<Database>) -> Result<Vec<EffectSlot>, String> {
    let conn = crate::db::get_connection(&db);

    let slots = conn
        .prepare("SELECT id, slot_index, effect_type, is_enabled, parameters, midi_note, COALESCE(midi_channel, 0) FROM effect_slots ORDER BY slot_index")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok(EffectSlot {
                id: row.get(0)?,
                slot_index: row.get(1)?,
                effect_type: row.get(2)?,
                is_enabled: row.get::<_, i32>(3)? == 1,
                parameters: row.get(4)?,
                midi_note: row.get(5)?,
                midi_channel: row.get::<_, i32>(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(slots)
}

#[tauri::command]
pub fn set_effect_slot(db: State<Database>, state: State<AppAudioState>, slot: SetEffectSlot) -> Result<bool, String> {
    let parameters = slot.parameters.clone().unwrap_or_else(|| get_default_parameters(&slot.effect_type));
    let is_enabled = slot.enabled.unwrap_or(true);

    {
        let conn = crate::db::get_connection(&db);
        conn.execute(
            "INSERT OR REPLACE INTO effect_slots (slot_index, effect_type, is_enabled, parameters) \
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                slot.slot_index,
                slot.effect_type,
                is_enabled as i32,
                parameters,
            ],
        )
        .map_err(|e| e.to_string())?;
    } // 释放数据库锁

    // 更新效果器链
    reload_effect_chain(&db, &state.global);

    Ok(true)
}

#[tauri::command]
pub fn update_effect_parameters(db: State<Database>, state: State<AppAudioState>, params: UpdateEffectParameters) -> Result<bool, String> {
    {
        let conn = crate::db::get_connection(&db);
        conn.execute(
            "UPDATE effect_slots SET parameters = ?1 WHERE slot_index = ?2",
            rusqlite::params![params.parameters, params.slot_index],
        )
        .map_err(|e| e.to_string())?;
    } // 释放数据库锁

    // 更新效果器链
    reload_effect_chain(&db, &state.global);

    Ok(true)
}

#[tauri::command]
pub fn toggle_effect(db: State<Database>, state: State<AppAudioState>, slot_index: i32, enabled: bool) -> Result<bool, String> {
    {
        let conn = crate::db::get_connection(&db);
        conn.execute(
            "UPDATE effect_slots SET is_enabled = ?1 WHERE slot_index = ?2",
            rusqlite::params![enabled as i32, slot_index],
        )
        .map_err(|e| e.to_string())?;
    } // 释放数据库锁

    // 更新效果器链
    reload_effect_chain(&db, &state.global);

    Ok(true)
}

#[tauri::command]
pub fn move_effect_slot(db: State<Database>, state: State<AppAudioState>, from_index: i32, to_index: i32) -> Result<bool, String> {
    println!("[move_effect_slot] from_index={}, to_index={}", from_index, to_index);

    if from_index == to_index {
        return Ok(true);
    }

    {
        let mut conn = crate::db::get_connection(&db);
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // 获取当前所有 slot_index，按顺序排列
        let slots: Vec<i32> = tx
            .prepare("SELECT slot_index FROM effect_slots ORDER BY slot_index")
            .map_err(|e| e.to_string())?
            .query_map([], |row| row.get::<_, i32>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        println!("[move_effect_slot] Current slots: {:?}", slots);

        // 找到 from_index 在数组中的位置
        let from_pos = slots.iter().position(|&s| s == from_index)
            .ok_or_else(|| format!("Slot index {} not found", from_index))?;

        // 计算 to_pos（目标位置）
        let to_pos = if to_index < 0 {
            0
        } else if to_index as usize >= slots.len() {
            slots.len() - 1
        } else {
            // 找到 to_index 在数组中的位置
            slots.iter().position(|&s| s == to_index).unwrap_or(from_pos)
        };

        println!("[move_effect_slot] from_pos={}, to_pos={}", from_pos, to_pos);

        if from_pos == to_pos {
            return Ok(true);
        }

        // 构建新的顺序
        let mut new_slots = slots.clone();
        let removed = new_slots.remove(from_pos);
        new_slots.insert(to_pos, removed);

        println!("[move_effect_slot] New slots order: {:?}", new_slots);

        // 第一步：先把所有记录移到临时索引（避免冲突）
        for (i, old_slot_idx) in slots.iter().enumerate() {
            tx.execute(
                "UPDATE effect_slots SET slot_index = -1000 - ? WHERE slot_index = ?",
                rusqlite::params![i as i32, old_slot_idx],
            ).map_err(|e| e.to_string())?;
        }

        // 第二步：更新为最终的连续索引
        for (new_idx, old_slot_idx) in new_slots.iter().enumerate() {
            let temp_idx = -1000 - slots.iter().position(|s| *s == *old_slot_idx).unwrap() as i32;
            tx.execute(
                "UPDATE effect_slots SET slot_index = ? WHERE slot_index = ?",
                rusqlite::params![new_idx as i32, temp_idx],
            ).map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;

        println!("[move_effect_slot] Successfully moved effect");
    }

    // 更新效果器链
    reload_effect_chain(&db, &state.global);

    Ok(true)
}

#[tauri::command]
pub fn clear_effect_slot(db: State<Database>, state: State<AppAudioState>, slot_index: i32) -> Result<bool, String> {
    {
        let conn = crate::db::get_connection(&db);
        conn.execute("DELETE FROM effect_slots WHERE slot_index = ?", [slot_index])
            .map_err(|e| e.to_string())?;
    } // 释放数据库锁

    // 更新效果器链
    reload_effect_chain(&db, &state.global);

    Ok(true)
}

// ============ 效果器预设 ============

#[tauri::command]
pub fn get_effect_presets(db: State<Database>) -> Result<Vec<EffectPreset>, String> {
    let conn = crate::db::get_connection(&db);

    let presets = conn
        .prepare("SELECT id, name, description, is_default FROM effect_presets ORDER BY name")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok(EffectPreset {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                is_default: row.get::<_, i32>(3)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(presets)
}

#[tauri::command]
pub fn save_effect_preset(db: State<Database>, preset: NewEffectPreset) -> Result<i64, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "INSERT INTO effect_presets (name, description) VALUES (?1, ?2)",
        rusqlite::params![preset.name, preset.description],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_effect_preset(db: State<Database>, preset_id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM effect_presets WHERE id = ?", [preset_id])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn bypass_all_effects(db: State<Database>, bypass: bool) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE effect_chain_config SET bypass_all = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        [bypass as i32],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

// ============ 效果器 MIDI 学习 ============

/// 设置效果器的 MIDI 控制映射
#[tauri::command]
pub fn set_effect_midi(db: State<Database>, slot_index: i32, midi_note: i32, midi_channel: i32) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE effect_slots SET midi_note = ?, midi_channel = ? WHERE slot_index = ?",
        rusqlite::params![midi_note, midi_channel, slot_index],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

/// 清除效果器的 MIDI 控制映射
#[tauri::command]
pub fn clear_effect_midi(db: State<Database>, slot_index: i32) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE effect_slots SET midi_note = NULL WHERE slot_index = ?",
        [slot_index],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

// ============ 音频设备 ============

#[tauri::command]
pub fn list_audio_input_devices(state: State<AppAudioState>) -> Result<Vec<DeviceInfo>, String> {
    let manager = LiveAudioManager::new(Arc::clone(&state.global));
    Ok(manager.list_input_devices())
}

#[tauri::command]
pub fn list_audio_output_devices(state: State<AppAudioState>) -> Result<Vec<DeviceInfo>, String> {
    let manager = LiveAudioManager::new(Arc::clone(&state.global));
    Ok(manager.list_output_devices())
}

// ============ 实时音频路由 ============

#[tauri::command]
pub fn start_live_audio(db: State<Database>, state: State<AppAudioState>, app_state: State<AppState>, config: LiveAudioConfig) -> Result<bool, String> {
    println!("[Command] start_live_audio called");

    // 检查流是否已经存在
    let streams_exist = AUDIO_STREAMS.with(|streams| {
        streams.borrow().is_some()
    });

    if streams_exist {
        // 流已存在，只需关闭旁通
        state.global.set_effect_bypass(false);
        state.global.is_running.store(true, std::sync::atomic::Ordering::SeqCst);
        println!("[Command] Streams exist, disabled bypass");
        return Ok(true);
    }

    // 首次启动：创建流
    println!("[Command] Creating new streams...");
    let mut manager = LiveAudioManager::new(Arc::clone(&state.global));
    let new_streams = manager.start(config)?;

    // 设置采样率并加载效果器链
    state.global.set_sample_rate(manager.sample_rate);
    reload_effect_chain(&db, &state.global);

    // 加载 ducking 参数
    {
        let conn = crate::db::get_connection(&db);
        let ducking_params: Option<(bool, f32, f32, i32)> = conn
            .query_row(
                "SELECT ducking_enabled, ducking_threshold, ducking_ratio, COALESCE(ducking_recovery_delay, 3) FROM audio_config WHERE id = 1",
                [],
                |row| Ok((row.get::<_, i32>(0)? == 1, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .ok();

        if let Some((enabled, threshold, ratio, recovery_delay)) = ducking_params {
            state.global.set_ducking_params(enabled, threshold, ratio, recovery_delay as u32);
            println!("[Command] Loaded ducking params from DB: enabled={}, threshold={}, ratio={}, recovery_delay={}s", enabled, threshold, ratio, recovery_delay);
        }
    }

    // 设置过场音乐管理器引用（用于 ducking）
    {
        let interlude_manager = app_state.interlude_manager.lock().unwrap();
        // 使用 Arc 和 Mutex 包装，因为 InterludeManager 本身不是 Send
        let manager_arc = Arc::new(std::sync::Mutex::new(interlude_manager.clone()));
        state.global.set_interlude_manager(manager_arc);
    }

    // 关闭旁通
    state.global.set_effect_bypass(false);

    AUDIO_STREAMS.with(|streams| {
        *streams.borrow_mut() = Some(new_streams);
    });

    println!("[Command] start_live_audio completed successfully");
    Ok(true)
}

#[tauri::command]
pub fn stop_live_audio(state: State<AppAudioState>) -> Result<bool, String> {
    println!("[Command] stop_live_audio called");

    // 停止录音
    let _ = state.global.recorder.lock().unwrap().stop_recording();

    // 只开启旁通，不销毁流
    state.global.set_effect_bypass(true);
    state.global.is_running.store(false, std::sync::atomic::Ordering::SeqCst);

    println!("[Command] stop_live_audio completed (bypass enabled)");
    Ok(true)
}

#[tauri::command]
pub fn set_effect_bypass(state: State<AppAudioState>, bypass: bool) -> Result<bool, String> {
    state.global.set_effect_bypass(bypass);
    Ok(true)
}

#[tauri::command]
pub fn get_output_level(state: State<AppAudioState>) -> Result<f32, String> {
    if let Ok(meter) = state.global.output_level_meter.lock() {
        Ok(meter.get_level())
    } else {
        Ok(0.0)
    }
}

#[tauri::command]
pub fn get_level_meter_value(state: State<AppAudioState>, slot_index: i32) -> Result<Option<f32>, String> {
    if let Ok(chain) = state.global.effect_chain.lock() {
        // 检查指定 slot_index 是否是 LevelMeter 类型
        let is_meter = chain.is_level_meter(slot_index);
        println!("[get_level_meter_value] slot_index={}, is_level_meter={}", slot_index, is_meter);

        if is_meter {
            let value = chain.get_level_meter_value_by_slot(slot_index);
            println!("[get_level_meter_value] returning value: {:?}", value);
            return Ok(value);
        }
    }
    Ok(None)
}

/// 向上移动效果器（与前一个效果器交换位置）
#[tauri::command]
pub fn move_effect_up(db: State<Database>, state: State<AppAudioState>, slot_index: i32) -> Result<bool, String> {
    println!("[move_effect_up] slot_index={}", slot_index);

    // 获取当前所有 slot_index，按顺序排列
    let slots: Vec<i32> = {
        let conn = crate::db::get_connection(&db);
        let mut stmt = conn
            .prepare("SELECT slot_index FROM effect_slots ORDER BY slot_index")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| row.get::<_, i32>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    println!("[move_effect_up] Current slots: {:?}", slots);

    // 找到当前位置
    let current_pos = slots.iter().position(|&s| s == slot_index)
        .ok_or_else(|| format!("Slot index {} not found", slot_index))?;

    if current_pos == 0 {
        println!("[move_effect_up] Already at first position");
        return Ok(false); // 已经是第一个
    }

    // 目标位置是前一个
    let target_pos = current_pos - 1;
    let target_slot_index = slots[target_pos];

    println!("[move_effect_up] Moving from pos {} to pos {}, target_slot_index={}",
             current_pos, target_pos, target_slot_index);

    move_effect_slot(db, state, slot_index, target_slot_index)
}

/// 向下移动效果器（与后一个效果器交换位置）
#[tauri::command]
pub fn move_effect_down(db: State<Database>, state: State<AppAudioState>, slot_index: i32) -> Result<bool, String> {
    println!("[move_effect_down] slot_index={}", slot_index);

    // 获取当前所有 slot_index，按顺序排列
    let slots: Vec<i32> = {
        let conn = crate::db::get_connection(&db);
        let mut stmt = conn
            .prepare("SELECT slot_index FROM effect_slots ORDER BY slot_index")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| row.get::<_, i32>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    println!("[move_effect_down] Current slots: {:?}", slots);

    // 找到当前位置
    let current_pos = slots.iter().position(|&s| s == slot_index)
        .ok_or_else(|| format!("Slot index {} not found", slot_index))?;

    if current_pos >= slots.len() - 1 {
        println!("[move_effect_down] Already at last position");
        return Ok(false); // 已经是最后一个
    }

    // 目标位置是后一个
    let target_pos = current_pos + 1;
    let target_slot_index = slots[target_pos];

    println!("[move_effect_down] Moving from pos {} to pos {}, target_slot_index={}",
             current_pos, target_pos, target_slot_index);

    move_effect_slot(db, state, slot_index, target_slot_index)
}

#[tauri::command]
pub fn get_live_audio_state(state: State<AppAudioState>) -> Result<LiveAudioState, String> {
    let manager = LiveAudioManager::new(Arc::clone(&state.global));
    Ok(manager.get_state())
}

#[tauri::command]
pub fn set_vocal_volume(state: State<AppAudioState>, volume: f32) -> Result<bool, String> {
    state.global.set_vocal_volume(volume);
    Ok(true)
}

#[tauri::command]
pub fn set_instrument_volume(state: State<AppAudioState>, volume: f32) -> Result<bool, String> {
    state.global.set_instrument_volume(volume);
    Ok(true)
}

#[tauri::command]
pub fn set_effect_input(state: State<AppAudioState>, effect_input: EffectInput) -> Result<bool, String> {
    state.global.set_effect_input(effect_input);
    Ok(true)
}

#[tauri::command]
pub fn set_vocal_channel(state: State<AppAudioState>, channel: i32) -> Result<bool, String> {
    state.global.set_vocal_channel(channel as usize);
    Ok(true)
}

#[tauri::command]
pub fn set_instrument_channel(state: State<AppAudioState>, channel: i32) -> Result<bool, String> {
    state.global.set_instrument_channel(channel as usize);
    Ok(true)
}

// ============ Ducking 调试 ============

#[tauri::command]
pub fn get_ducking_debug_state(state: State<AppAudioState>) -> Result<crate::modules::audio_router::live_router::DuckingDebugState, String> {
    Ok(state.global.get_ducking_debug_state())
}

// ============ 录音控制 ============

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingConfig {
    pub vocal_path: Option<String>,
    pub instrument_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingResult {
    pub vocal_path: Option<String>,
    pub instrument_path: Option<String>,
}

#[tauri::command]
pub fn start_recording(state: State<AppAudioState>, config: RecordingConfig) -> Result<bool, String> {
    let vocal_path = config.vocal_path.map(PathBuf::from);
    let instrument_path = config.instrument_path.map(PathBuf::from);

    let manager = LiveAudioManager::new(Arc::clone(&state.global));
    manager.start_recording(vocal_path, instrument_path)?;
    Ok(true)
}

#[tauri::command]
pub fn stop_recording(state: State<AppAudioState>) -> Result<RecordingResult, String> {
    let manager = LiveAudioManager::new(Arc::clone(&state.global));
    let (vocal_path, instrument_path) = manager.stop_recording()?;

    Ok(RecordingResult {
        vocal_path: vocal_path.map(|p| p.to_string_lossy().to_string()),
        instrument_path: instrument_path.map(|p| p.to_string_lossy().to_string()),
    })
}

#[tauri::command]
pub fn get_recording_state(state: State<AppAudioState>) -> Result<bool, String> {
    Ok(state.global.recorder.lock().unwrap().is_recording())
}
