use tauri::State;
use crate::db::Database;
use crate::models::{AtmosphereSound, NewAtmosphereSound, UpdateAtmosphereSound};

#[tauri::command]
pub fn get_atmosphere_sounds(db: State<Database>) -> Result<Vec<AtmosphereSound>, String> {
    let conn = crate::db::get_connection(&db);

    let sounds = conn
        .prepare("SELECT id, name, file_path, duration, volume, midi_note, midi_channel, is_one_shot, color, sort_order FROM atmosphere_sounds ORDER BY sort_order")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok(AtmosphereSound {
                id: row.get(0)?,
                name: row.get(1)?,
                file_path: row.get(2)?,
                duration: row.get(3)?,
                volume: row.get(4)?,
                midi_note: row.get(5)?,
                midi_channel: row.get(6)?,
                is_one_shot: row.get::<_, i32>(7)? == 1,
                color: row.get(8)?,
                sort_order: row.get(9)?,
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
    let midi_channel = sound.midi_channel.unwrap_or(0);
    let is_one_shot = sound.is_one_shot.unwrap_or(true);

    // 获取最大排序值
    let max_order: i32 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), 0) FROM atmosphere_sounds", [], |row| row.get(0))
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO atmosphere_sounds (name, file_path, volume, midi_note, midi_channel, is_one_shot, color, sort_order) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            sound.name,
            sound.file_path,
            volume,
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

    conn.execute(
        "UPDATE atmosphere_sounds SET \
         name = COALESCE(?1, name), \
         volume = COALESCE(?2, volume), \
         midi_note = COALESCE(?3, midi_note), \
         midi_channel = COALESCE(?4, midi_channel), \
         is_one_shot = COALESCE(?5, is_one_shot), \
         color = COALESCE(?6, color), \
         sort_order = COALESCE(?7, sort_order) \
         WHERE id = ?8",
        rusqlite::params![
            sound.name,
            sound.volume,
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
pub fn delete_atmosphere_sound(db: State<Database>, id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM atmosphere_sounds WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn play_atmosphere_sound(id: i64) -> Result<bool, String> {
    let _ = id;
    // TODO: 实现播放气氛组音效逻辑
    Ok(true)
}

#[tauri::command]
pub fn stop_atmosphere_sound(id: Option<i64>) -> Result<bool, String> {
    let _ = id;
    // TODO: 实现停止气氛组音效逻辑
    Ok(true)
}
