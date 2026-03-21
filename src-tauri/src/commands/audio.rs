use tauri::State;
use crate::db::Database;
use crate::models::{AudioConfig, UpdateAudioConfig};
use crate::modules::audio_router::AudioManager;
use crate::commands::effect::AppAudioState;
use crate::modules::AppState;
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
         COALESCE(ducking_recovery_delay, 3), \
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
                ducking_recovery_delay: row.get(11)?,
                midi_device_id: row.get(12)?,
                midi_enabled: row.get::<_, i32>(13)? == 1,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_audio_config(
    db: State<Database>,
    audio_state: State<AppAudioState>,
    app_state: State<AppState>,
    config: UpdateAudioConfig,
) -> Result<bool, String> {
    println!("[Audio] save_audio_config called with: {:?}", config);
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
         ducking_recovery_delay = COALESCE(?12, ducking_recovery_delay), \
         midi_device_id = COALESCE(?13, midi_device_id), \
         midi_enabled = COALESCE(?14, midi_enabled), \
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
            config.ducking_recovery_delay,
            config.midi_device_id,
            config.midi_enabled.map(|b| b as i32),
        ],
    )
    .map_err(|e| e.to_string())?;

    println!("[Audio] Database updated successfully");

    // 更新运行时的 ducking 参数
    if config.ducking_enabled.is_some()
        || config.ducking_threshold.is_some()
        || config.ducking_ratio.is_some()
        || config.ducking_recovery_delay.is_some()
    {
        // 从数据库重新读取完整配置
        let (enabled, threshold, ratio, recovery_delay): (bool, f32, f32, u32) = conn
            .query_row(
                "SELECT ducking_enabled, ducking_threshold, ducking_ratio, COALESCE(ducking_recovery_delay, 8) FROM audio_config WHERE id = 1",
                [],
                |row| Ok((row.get::<_, i32>(0)? == 1, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| e.to_string())?;

        audio_state.global.set_ducking_params(enabled, threshold, ratio, recovery_delay);
        println!("[Audio] Updated ducking params: enabled={}, threshold={}, ratio={}, recovery_delay={}s",
            enabled, threshold, ratio, recovery_delay);
    }

    // 更新过场音乐音量
    if let Some(volume) = config.interlude_volume {
        let mut manager = app_state.interlude_manager.lock().unwrap();
        let _ = manager.set_volume(volume);
        println!("[Audio] Updated interlude volume: {}", volume);
    }

    // 更新气氛组音量
    if let Some(volume) = config.atmosphere_volume {
        let mut manager = app_state.atmosphere_manager.lock().unwrap();
        manager.set_volume(volume);
        println!("[Audio] Updated atmosphere volume: {}", volume);
    }

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
