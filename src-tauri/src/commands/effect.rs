use tauri::State;
use crate::db::Database;
use crate::models::{EffectSlot, EffectChainConfig, SetEffectSlot, UpdateEffectParameters, EffectPreset, NewEffectPreset, get_default_parameters};

// ============ 效果器链配置 ============

#[tauri::command]
pub fn get_effect_chain_config(db: State<Database>) -> Result<EffectChainConfig, String> {
    let conn = crate::db::get_connection(&db);

    conn.query_row(
        "SELECT input_device_id, input_volume, monitor_device_id, stream_device_id, \
         monitor_volume, stream_volume, bypass_all \
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
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_effect_chain_config(
    db: State<Database>,
    config: crate::models::UpdateAudioConfig,
) -> Result<bool, String> {
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
         updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        rusqlite::params![
            config.default_output_device,
            config.master_volume,
            config.interlude_output_device,
            config.atmosphere_output_device,
            config.interlude_volume,
            config.atmosphere_volume,
            config.ducking_enabled.map(|b| b as i32),
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

// ============ 效果器槽位 ============

#[tauri::command]
pub fn get_effect_slots(db: State<Database>) -> Result<Vec<EffectSlot>, String> {
    let conn = crate::db::get_connection(&db);

    let slots = conn
        .prepare("SELECT id, slot_index, effect_type, is_enabled, parameters FROM effect_slots ORDER BY slot_index")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok(EffectSlot {
                id: row.get(0)?,
                slot_index: row.get(1)?,
                effect_type: row.get(2)?,
                is_enabled: row.get::<_, i32>(3)? == 1,
                parameters: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(slots)
}

#[tauri::command]
pub fn set_effect_slot(db: State<Database>, slot: SetEffectSlot) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    let parameters = slot.parameters.unwrap_or_else(|| get_default_parameters(&slot.effect_type));
    let is_enabled = slot.enabled.unwrap_or(true);

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

    Ok(true)
}

#[tauri::command]
pub fn update_effect_parameters(db: State<Database>, params: UpdateEffectParameters) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE effect_slots SET parameters = ?1 WHERE slot_index = ?2",
        rusqlite::params![params.parameters, params.slot_index],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn toggle_effect(db: State<Database>, slot_index: i32, enabled: bool) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE effect_slots SET is_enabled = ?1 WHERE slot_index = ?2",
        rusqlite::params![enabled as i32, slot_index],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn move_effect_slot(db: State<Database>, from_index: i32, to_index: i32) -> Result<bool, String> {
    let mut conn = crate::db::get_connection(&db);

    // 交换两个槽位的位置
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE effect_slots SET slot_index = -1 WHERE slot_index = ?1",
        [from_index],
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE effect_slots SET slot_index = ?1 WHERE slot_index = ?2",
        rusqlite::params![from_index, to_index],
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE effect_slots SET slot_index = ?1 WHERE slot_index = -1",
        [to_index],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn clear_effect_slot(db: State<Database>, slot_index: i32) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM effect_slots WHERE slot_index = ?", [slot_index])
        .map_err(|e| e.to_string())?;

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

    // 获取当前效果器槽位配置
    let slots: Vec<(i32, String, bool, String)> = conn
        .prepare("SELECT slot_index, effect_type, is_enabled, parameters FROM effect_slots ORDER BY slot_index")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get::<_, i32>(2)? == 1,
                row.get(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 保存预设
    conn.execute(
        "INSERT INTO effect_presets (name, description) VALUES (?1, ?2)",
        rusqlite::params![preset.name, preset.description],
    )
    .map_err(|e| e.to_string())?;

    let preset_id = conn.last_insert_rowid();

    // TODO: 保存预设与槽位的关联

    Ok(preset_id)
}

#[tauri::command]
pub fn load_effect_preset(db: State<Database>, preset_id: i64) -> Result<bool, String> {
    let _ = db;
    let _ = preset_id;
    // TODO: 实现加载预设逻辑
    Ok(true)
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
