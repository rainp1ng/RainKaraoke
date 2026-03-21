use tauri::{State, Manager};
use crate::db::Database;
use crate::models::{AtmosphereSound, NewAtmosphereSound, UpdateAtmosphereSound, MidiMessageType};
use crate::modules::AppState;

#[tauri::command]
pub fn get_atmosphere_sounds(db: State<Database>) -> Result<Vec<AtmosphereSound>, String> {
    let conn = crate::db::get_connection(&db);

    let sounds = conn
        .prepare("SELECT id, name, file_path, duration, volume, midi_message_type, midi_note, midi_channel, is_one_shot, color, sort_order FROM atmosphere_sounds ORDER BY sort_order")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            let midi_type_str: String = row.get(5).unwrap_or_else(|_| "NOTE".to_string());
            Ok(AtmosphereSound {
                id: row.get(0)?,
                name: row.get(1)?,
                file_path: row.get(2)?,
                duration: row.get(3)?,
                volume: row.get(4)?,
                midi_message_type: MidiMessageType::from(midi_type_str.as_str()),
                midi_note: row.get(6)?,
                midi_channel: row.get(7)?,
                is_one_shot: row.get::<_, i32>(8)? == 1,
                color: row.get(9)?,
                sort_order: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sounds)
}

#[tauri::command]
pub fn add_atmosphere_sound(db: State<Database>, sound: NewAtmosphereSound) -> Result<i64, String> {
    let conn = crate::db::get_connection(&db);

    let volume = sound.volume.unwrap_or(0.8);
    let midi_message_type = sound.midi_message_type.unwrap_or(MidiMessageType::Note);
    let midi_channel = sound.midi_channel.unwrap_or(0);
    let is_one_shot = sound.is_one_shot.unwrap_or(true);

    // 获取最大排序值
    let max_order: i32 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), 0) FROM atmosphere_sounds", [], |row| row.get(0))
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO atmosphere_sounds (name, file_path, volume, midi_message_type, midi_note, midi_channel, is_one_shot, color, sort_order) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            sound.name,
            sound.file_path,
            volume,
            midi_message_type.to_string(),
            sound.midi_note,
            midi_channel,
            is_one_shot as i32,
            sound.color,
            max_order + 1,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_atmosphere_sound(db: State<Database>, sound: UpdateAtmosphereSound) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    let midi_type_str = sound.midi_message_type.map(|t| t.to_string());

    conn.execute(
        "UPDATE atmosphere_sounds SET \
         name = COALESCE(?1, name), \
         volume = COALESCE(?2, volume), \
         midi_message_type = COALESCE(?3, midi_message_type), \
         midi_note = COALESCE(?4, midi_note), \
         midi_channel = COALESCE(?5, midi_channel), \
         is_one_shot = COALESCE(?6, is_one_shot), \
         color = COALESCE(?7, color), \
         sort_order = COALESCE(?8, sort_order) \
         WHERE id = ?9",
        rusqlite::params![
            sound.name,
            sound.volume,
            midi_type_str,
            sound.midi_note,
            sound.midi_channel,
            sound.is_one_shot.map(|b| b as i32),
            sound.color,
            sound.sort_order,
            sound.id,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn delete_atmosphere_sound(db: State<Database>, state: State<AppState>, id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM atmosphere_sounds WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;

    // 停止正在播放的音效
    let mut manager = state.atmosphere_manager.lock().unwrap();
    let _ = manager.stop_sound(Some(id));

    Ok(true)
}

#[tauri::command]
pub fn play_atmosphere_sound(db: State<Database>, state: State<AppState>, id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    // 从数据库获取音效信息
    let sound_data = conn
        .query_row(
            "SELECT name, file_path, volume FROM atmosphere_sounds WHERE id = ?",
            [id],
            |row| {
                Ok(crate::modules::atmosphere::AtmosphereSoundData {
                    id,
                    name: row.get(0)?,
                    file_path: row.get(1)?,
                    volume: row.get(2)?,
                })
            },
        )
        .map_err(|e| format!("找不到音效: {}", e))?;

    let mut manager = state.atmosphere_manager.lock().unwrap();
    manager.play_sound(&sound_data)?;

    Ok(true)
}

#[tauri::command]
pub fn stop_atmosphere_sound(state: State<AppState>, id: Option<i64>) -> Result<bool, String> {
    let mut manager = state.atmosphere_manager.lock().unwrap();
    manager.stop_sound(id)?;

    Ok(true)
}

#[tauri::command]
pub fn set_atmosphere_volume(db: State<Database>, state: State<AppState>, volume: f32) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE audio_config SET atmosphere_volume = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        [volume],
    )
    .map_err(|e| e.to_string())?;

    // 实时更新运行时音量
    let mut manager = state.atmosphere_manager.lock().unwrap();
    manager.set_volume(volume);

    println!("[Atmosphere] Volume set to: {}", volume);
    Ok(true)
}
