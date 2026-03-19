use tauri::State;
use crate::db::Database;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItem {
    pub id: i64,
    pub song_id: i64,
    pub position: i32,
}

#[tauri::command]
pub fn get_queue(db: State<Database>) -> Result<Vec<QueueItem>, String> {
    let conn = crate::db::get_connection(&db);

    let items = conn
        .prepare("SELECT id, song_id, position FROM play_queue ORDER BY position")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok(QueueItem {
                id: row.get(0)?,
                song_id: row.get(1)?,
                position: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(items)
}

#[tauri::command]
pub fn add_to_queue(db: State<Database>, song_id: i64, position: Option<i32>) -> Result<i64, String> {
    let conn = crate::db::get_connection(&db);

    let pos = position.unwrap_or_else(|| {
        let max: i32 = conn
            .query_row("SELECT COALESCE(MAX(position), 0) FROM play_queue", [], |row| row.get(0))
            .unwrap_or(0);
        max + 1
    });

    conn.execute(
        "INSERT INTO play_queue (song_id, position) VALUES (?, ?)",
        rusqlite::params![song_id, pos],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn remove_from_queue(db: State<Database>, queue_id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM play_queue WHERE id = ?", [queue_id])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn move_queue_item(db: State<Database>, queue_id: i64, new_position: i32) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE play_queue SET position = ? WHERE id = ?",
        rusqlite::params![new_position, queue_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn clear_queue(db: State<Database>) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM play_queue", [])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn play_next(db: State<Database>) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    // 获取队列第一首歌曲
    let next_song = conn
        .query_row(
            "SELECT id, song_id FROM play_queue ORDER BY position LIMIT 1",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .ok();

    if let Some((queue_id, song_id)) = next_song {
        // 从队列移除
        conn.execute("DELETE FROM play_queue WHERE id = ?", [queue_id])
            .map_err(|e| e.to_string())?;

        // TODO: 播放歌曲
        let _ = song_id;
        Ok(true)
    } else {
        Ok(false)
    }
}
