use tauri::State;
use crate::db::Database;
use crate::models::{InterludeTrack, NewInterludeTrack};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterludeState {
    pub is_playing: bool,
    pub current_track_id: Option<i64>,
    pub volume: f32,
    pub ducking_active: bool,
}

#[tauri::command]
pub fn get_interlude_tracks(db: State<Database>) -> Result<Vec<InterludeTrack>, String> {
    let conn = crate::db::get_connection(&db);

    let tracks = conn
        .prepare("SELECT id, title, file_path, duration, volume, is_active, play_count FROM interlude_tracks WHERE is_active = 1")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok(InterludeTrack {
                id: row.get(0)?,
                title: row.get(1)?,
                file_path: row.get(2)?,
                duration: row.get(3)?,
                volume: row.get(4)?,
                is_active: row.get::<_, i32>(5)? == 1,
                play_count: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tracks)
}

#[tauri::command]
pub fn add_interlude_track(db: State<Database>, track: NewInterludeTrack) -> Result<i64, String> {
    let conn = crate::db::get_connection(&db);
    let volume = track.volume.unwrap_or(0.5);

    conn.execute(
        "INSERT INTO interlude_tracks (title, file_path, volume) VALUES (?, ?, ?)",
        rusqlite::params![track.title, track.file_path, volume],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_interlude_track(db: State<Database>, id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM interlude_tracks WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn set_interlude_volume(db: State<Database>, volume: f32) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE audio_config SET interlude_volume = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        [volume],
    )
    .map_err(|e| e.to_string())?;

    // TODO: 实时更新播放音量
    Ok(true)
}

#[tauri::command]
pub fn get_interlude_state() -> Result<InterludeState, String> {
    // TODO: 实现获取过场音乐状态逻辑
    Ok(InterludeState {
        is_playing: false,
        current_track_id: None,
        volume: 0.3,
        ducking_active: false,
    })
}
