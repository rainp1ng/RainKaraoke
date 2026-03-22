use tauri::State;
use crate::db::Database;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub cover_path: Option<String>,
    pub song_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSong {
    pub id: i64,
    pub playlist_id: i64,
    pub song_id: i64,
    pub position: i32,
    pub added_at: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration: Option<i32>,
}

/// 获取所有歌单
#[tauri::command]
pub fn get_playlists(db: State<Database>) -> Result<Vec<Playlist>, String> {
    let conn = crate::db::get_connection(&db);

    let playlists = conn
        .prepare(
            "SELECT p.id, p.name, p.description, p.cover_path,
                    (SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = p.id) as song_count,
                    p.created_at, p.updated_at
             FROM playlists p
             ORDER BY p.updated_at DESC"
        )
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                cover_path: row.get(3)?,
                song_count: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(playlists)
}

/// 获取单个歌单详情
#[tauri::command]
pub fn get_playlist_by_id(db: State<Database>, id: i64) -> Result<Option<Playlist>, String> {
    let conn = crate::db::get_connection(&db);

    let result = conn
        .query_row(
            "SELECT p.id, p.name, p.description, p.cover_path,
                    (SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = p.id) as song_count,
                    p.created_at, p.updated_at
             FROM playlists p
             WHERE p.id = ?",
            [id],
            |row| {
                Ok(Playlist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    cover_path: row.get(3)?,
                    song_count: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        );

    match result {
        Ok(playlist) => Ok(Some(playlist)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 创建歌单
#[tauri::command]
pub fn create_playlist(db: State<Database>, name: String, description: Option<String>) -> Result<i64, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "INSERT INTO playlists (name, description) VALUES (?1, ?2)",
        rusqlite::params![name, description],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

/// 更新歌单
#[tauri::command]
pub fn update_playlist(
    db: State<Database>,
    id: i64,
    name: Option<String>,
    description: Option<String>,
) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE playlists SET name = COALESCE(?1, name), description = COALESCE(?2, description), updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        rusqlite::params![name, description, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

/// 删除歌单
#[tauri::command]
pub fn delete_playlist(db: State<Database>, id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM playlists WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// 获取歌单中的歌曲
#[tauri::command]
pub fn get_playlist_songs(db: State<Database>, playlistId: i64) -> Result<Vec<PlaylistSong>, String> {
    let conn = crate::db::get_connection(&db);

    let songs = conn
        .prepare(
            "SELECT ps.id, ps.playlist_id, ps.song_id, ps.position, ps.added_at,
                    s.title, s.artist, s.duration
             FROM playlist_songs ps
             LEFT JOIN songs s ON ps.song_id = s.id
             WHERE ps.playlist_id = ?
             ORDER BY ps.position",
        )
        .map_err(|e| e.to_string())?
        .query_map([playlistId], |row| {
            Ok(PlaylistSong {
                id: row.get(0)?,
                playlist_id: row.get(1)?,
                song_id: row.get(2)?,
                position: row.get(3)?,
                added_at: row.get(4)?,
                title: row.get(5)?,
                artist: row.get(6)?,
                duration: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(songs)
}

/// 添加歌曲到歌单
#[tauri::command]
pub fn add_song_to_playlist(db: State<Database>, playlistId: i64, songId: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    // 获取当前最大位置
    let max_position: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM playlist_songs WHERE playlist_id = ?",
            [playlistId],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    let next_position = max_position + 1;

    conn.execute(
        "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, position) VALUES (?1, ?2, ?3)",
        rusqlite::params![playlistId, songId, next_position],
    )
    .map_err(|e| e.to_string())?;

    // 更新歌单的 updated_at
    conn.execute(
        "UPDATE playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [playlistId],
    )
    .ok();

    Ok(true)
}

/// 从歌单移除歌曲
#[tauri::command]
pub fn remove_song_from_playlist(db: State<Database>, playlistId: i64, songId: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
        rusqlite::params![playlistId, songId],
    )
    .map_err(|e| e.to_string())?;

    // 重新排序位置
    reorder_playlist_positions(&conn, playlistId)?;

    // 更新歌单的 updated_at
    conn.execute(
        "UPDATE playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [playlistId],
    )
    .ok();

    Ok(true)
}

/// 批量添加歌曲到歌单
#[tauri::command]
pub fn add_songs_to_playlist(db: State<Database>, playlistId: i64, songIds: Vec<i64>) -> Result<i32, String> {
    let conn = crate::db::get_connection(&db);

    // 获取当前最大位置
    let mut max_position: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM playlist_songs WHERE playlist_id = ?",
            [playlistId],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    let mut added = 0;
    for song_id in songIds {
        max_position += 1;
        let result = conn.execute(
            "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, position) VALUES (?1, ?2, ?3)",
            rusqlite::params![playlistId, song_id, max_position],
        );

        if let Ok(rows) = result {
            if rows > 0 {
                added += 1;
            }
        }
    }

    // 更新歌单的 updated_at
    conn.execute(
        "UPDATE playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [playlistId],
    )
    .ok();

    Ok(added)
}

/// 移动歌单中的歌曲位置
#[tauri::command]
pub fn move_playlist_song(db: State<Database>, playlistId: i64, songId: i64, newPosition: i32) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    // 获取当前位置
    let current_position: i32 = conn
        .query_row(
            "SELECT position FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
            rusqlite::params![playlistId, songId],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if current_position == newPosition {
        return Ok(true);
    }

    // 获取歌曲总数
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = ?",
            [playlistId],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // 确保 newPosition 在有效范围内
    let new_position = newPosition.max(0).min(count - 1);

    // 更新其他歌曲的位置
    if new_position < current_position {
        // 向前移动：中间的歌曲后移
        conn.execute(
            "UPDATE playlist_songs SET position = position + 1
             WHERE playlist_id = ?1 AND position >= ?2 AND position < ?3",
            rusqlite::params![playlistId, new_position, current_position],
        )
        .map_err(|e| e.to_string())?;
    } else {
        // 向后移动：中间的歌曲前移
        conn.execute(
            "UPDATE playlist_songs SET position = position - 1
             WHERE playlist_id = ?1 AND position > ?2 AND position <= ?3",
            rusqlite::params![playlistId, current_position, new_position],
        )
        .map_err(|e| e.to_string())?;
    }

    // 更新目标歌曲位置
    conn.execute(
        "UPDATE playlist_songs SET position = ?1 WHERE playlist_id = ?2 AND song_id = ?3",
        rusqlite::params![new_position, playlistId, songId],
    )
    .map_err(|e| e.to_string())?;

    // 更新歌单的 updated_at
    conn.execute(
        "UPDATE playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [playlistId],
    )
    .ok();

    Ok(true)
}

/// 重新排序歌单中的所有歌曲位置
fn reorder_playlist_positions(conn: &rusqlite::Connection, playlistId: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE playlist_songs SET position = (
            SELECT COUNT(*) FROM playlist_songs ps2
            WHERE ps2.playlist_id = playlist_songs.playlist_id
            AND ps2.position < playlist_songs.position
        ) WHERE playlist_id = ?",
        [playlistId],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// 清空歌单
#[tauri::command]
pub fn clear_playlist(db: State<Database>, playlistId: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?",
        [playlistId],
    )
    .map_err(|e| e.to_string())?;

    // 更新歌单的 updated_at
    conn.execute(
        "UPDATE playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [playlistId],
    )
    .ok();

    Ok(true)
}
