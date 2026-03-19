use tauri::State;
use crate::db::Database;
use crate::models::{AudioConfig, UpdateAudioConfig};
use crate::modules::audio_router::AudioManager;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub is_default: bool,
    pub channels: u16,
}

/// 全局音频管理器
static AUDIO_MANAGER: std::sync::OnceLock<Mutex<AudioManager>> = std::sync::OnceLock::new();

fn get_audio_manager() -> &'static Mutex<AudioManager> {
    AUDIO_MANAGER.get_or_init(|| Mutex::new(AudioManager::new()))
}

#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let manager = get_audio_manager().lock().map_err(|e| e.to_string())?;
    let devices = manager.list_devices();

    println!("[Audio] Found {} devices", devices.len());
    for d in &devices {
        println!("[Audio] - {} ({}, {} channels, default: {})", d.name, d.device_type, d.channels, d.is_default);
    }

    Ok(devices
        .into_iter()
        .map(|d| AudioDevice {
            id: d.id,
            name: d.name,
            device_type: d.device_type,
            is_default: d.is_default,
            channels: d.channels,
        })
        .collect())
}

#[tauri::command]
pub fn get_audio_config(db: State<Database>) -> Result<AudioConfig, String> {
    let conn = crate::db::get_connection(&db);

    conn.query_row(
        "SELECT default_output_device, interlude_output_device, atmosphere_output_device, \
         master_volume, interlude_volume, atmosphere_volume, \
         ducking_enabled, ducking_threshold, ducking_ratio, ducking_attack_ms, ducking_release_ms, \
         midi_device_id, midi_enabled \
         FROM audio_config WHERE id = 1",
        [],
        |row| {
            Ok(AudioConfig {
                default_output_device: row.get(0)?,
                interlude_output_device: row.get(1)?,
                atmosphere_output_device: row.get(2)?,
                master_volume: row.get(3)?,
                interlude_volume: row.get(4)?,
                atmosphere_volume: row.get(5)?,
                ducking_enabled: row.get::<_, i32>(6)? == 1,
                ducking_threshold: row.get(7)?,
                ducking_ratio: row.get(8)?,
                ducking_attack_ms: row.get(9)?,
                ducking_release_ms: row.get(10)?,
                midi_device_id: row.get(11)?,
                midi_enabled: row.get::<_, i32>(12)? == 1,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_audio_config(db: State<Database>, config: UpdateAudioConfig) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE audio_config SET \
         default_output_device = COALESCE(?1, default_output_device), \
         interlude_output_device = COALESCE(?2, interlude_output_device), \
         atmosphere_output_device = COALESCE(?3, atmosphere_output_device), \
         master_volume = COALESCE(?4, master_volume), \
         interlude_volume = COALESCE(?5, interlude_volume), \
         atmosphere_volume = COALESCE(?6, atmosphere_volume), \
         ducking_enabled = COALESCE(?7, ducking_enabled), \
         ducking_threshold = COALESCE(?8, ducking_threshold), \
         ducking_ratio = COALESCE(?9, ducking_ratio), \
         ducking_attack_ms = COALESCE(?10, ducking_attack_ms), \
         ducking_release_ms = COALESCE(?11, ducking_release_ms), \
         midi_device_id = COALESCE(?12, midi_device_id), \
         midi_enabled = COALESCE(?13, midi_enabled), \
         updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        rusqlite::params![
            config.default_output_device,
            config.interlude_output_device,
            config.atmosphere_output_device,
            config.master_volume,
            config.interlude_volume,
            config.atmosphere_volume,
            config.ducking_enabled.map(|b| b as i32),
            config.ducking_threshold,
            config.ducking_ratio,
            config.ducking_attack_ms,
            config.ducking_release_ms,
            config.midi_device_id,
            config.midi_enabled.map(|b| b as i32),
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn get_default_input_device() -> Result<Option<AudioDevice>, String> {
    let manager = get_audio_manager().lock().map_err(|e| e.to_string())?;
    Ok(manager.default_input_device().map(|d| AudioDevice {
        id: d.id,
        name: d.name,
        device_type: d.device_type,
        is_default: d.is_default,
        channels: d.channels,
    }))
}

#[tauri::command]
pub fn get_default_output_device() -> Result<Option<AudioDevice>, String> {
    let manager = get_audio_manager().lock().map_err(|e| e.to_string())?;
    Ok(manager.default_output_device().map(|d| AudioDevice {
        id: d.id,
        name: d.name,
        device_type: d.device_type,
        is_default: d.is_default,
        channels: d.channels,
    }))
}
