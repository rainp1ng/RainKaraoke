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
pub fn add_to_queue(db: State<Database>, songId: i64, position: Option<i32>) -> Result<i64, String> {
    let conn = crate::db::get_connection(&db);

    // 检查歌曲是否已在队列中
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM play_queue WHERE song_id = ?",
            [songId],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if exists {
        return Err("该歌曲已在播放队列中".to_string());
    }

    let pos = position.unwrap_or_else(|| {
        let max: i32 = conn
            .query_row("SELECT COALESCE(MAX(position), 0) FROM play_queue", [], |row| row.get(0))
            .unwrap_or(0);
        max + 1
    });

    conn.execute(
        "INSERT INTO play_queue (song_id, position) VALUES (?, ?)",
        rusqlite::params![songId, pos],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn remove_from_queue(db: State<Database>, queueId: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM play_queue WHERE id = ?", [queueId])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn move_queue_item(db: State<Database>, queueId: i64, newPosition: i32) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE play_queue SET position = ? WHERE id = ?",
        rusqlite::params![newPosition, queueId],
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

/// 顶歌 - 将指定歌曲移到队列最前面
#[tauri::command]
pub fn move_to_top(db: State<Database>, queueId: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    // 获取当前最小位置
    let min_pos: i32 = conn
        .query_row("SELECT COALESCE(MIN(position), 0) FROM play_queue", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    // 将该歌曲的位置设为最小位置 - 1
    conn.execute(
        "UPDATE play_queue SET position = ? WHERE id = ?",
        rusqlite::params![min_pos - 1, queueId],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

/// 顶歌到下一首位置 - 将指定歌曲移到当前播放歌曲之后
#[tauri::command]
pub fn move_to_next(db: State<Database>, queueId: i64, currentSongId: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    // 获取当前播放歌曲的 position
    let current_pos: Option<i32> = conn
        .query_row(
            "SELECT position FROM play_queue WHERE song_id = ?",
            [currentSongId],
            |row| row.get(0),
        )
        .ok();

    // 获取所有队列项并按 position 排序
    let mut items: Vec<(i64, i32)> = conn
        .prepare("SELECT id, position FROM play_queue ORDER BY position")
        .map_err(|e| e.to_string())?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 找到当前播放歌曲的索引
    let current_index = items.iter().position(|(id, _)| {
        // 需要通过 song_id 查找，但我们只有 queue id
        // 重新查询获取当前播放歌曲的 queue id
        let current_queue_id: Option<i64> = conn
            .query_row("SELECT id FROM play_queue WHERE song_id = ?", [currentSongId], |row| row.get(0))
            .ok();
        current_queue_id.map_or(false, |qid| *id == qid)
    });

    // 找到要移动的歌曲
    let target_index = items.iter().position(|(id, _)| *id == queueId);

    if let (Some(current_idx), Some(target_idx)) = (current_index, target_index) {
        // 移动目标到当前播放歌曲之后
        if target_idx != current_idx + 1 && target_idx > current_idx {
            // 从原位置移除
            let target_item = items.remove(target_idx);
            // 插入到当前播放歌曲之后
            items.insert(current_idx + 1, target_item);
        } else if target_idx < current_idx {
            // 目标在当前播放歌曲之前
            let target_item = items.remove(target_idx);
            items.insert(current_idx, target_item);
        }

        // 重新设置所有 position
        for (i, (id, _)) in items.iter().enumerate() {
            conn.execute(
                "UPDATE play_queue SET position = ? WHERE id = ?",
                rusqlite::params![i as i32 + 1, id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(true)
}
