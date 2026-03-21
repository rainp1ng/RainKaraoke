use tauri::State;
use crate::db::Database;
use crate::models::{Song, NewSong, UpdateSong, Tag};
use crate::utils::file_scanner::{self, ScannedFile, SongGroup};
use crate::utils::metadata_extractor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: u32,
    pub skipped: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProgress {
    pub current: u32,
    pub total: u32,
    pub current_file: String,
}

#[tauri::command]
pub fn get_songs(
    db: State<Database>,
    page: Option<i32>,
    page_size: Option<i32>,
    search: Option<String>,
    artist: Option<String>,
    genre: Option<String>,
    language: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
) -> Result<Vec<Song>, String> {
    let conn = crate::db::get_connection(&db);
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(50);
    let offset = (page - 1) * page_size;

    let mut query = String::from(
        "SELECT id, title, artist, album, duration, video_path, vocal_audio_path, \
         instrumental_audio_path, lyrics_path, lyrics_format, has_vocal, has_instrumental, \
         genre, language, tags, difficulty, play_count, last_played_at, created_at, updated_at \
         FROM songs WHERE 1=1"
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref s) = search {
        query.push_str(" AND (title LIKE ? OR artist LIKE ? OR album LIKE ?)");
        let pattern = format!("%{}%", s);
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern));
    }

    if let Some(ref a) = artist {
        query.push_str(" AND artist = ?");
        params.push(Box::new(a.clone()));
    }

    if let Some(ref g) = genre {
        query.push_str(" AND genre = ?");
        params.push(Box::new(g.clone()));
    }

    if let Some(ref l) = language {
        query.push_str(" AND language = ?");
        params.push(Box::new(l.clone()));
    }

    // 排序
    let sort_column = match sort_by.as_deref() {
        Some("artist") => "artist",
        Some("play_count") => "play_count",
        Some("last_played_at") => "last_played_at",
        Some("created_at") => "created_at",
        _ => "title",
    };
    let order = if sort_order.as_deref() == Some("desc") { "DESC" } else { "ASC" };
    query.push_str(&format!(" ORDER BY {} {}", sort_column, order));

    query.push_str(" LIMIT ? OFFSET ?");

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter()
        .map(|p| p.as_ref())
        .chain(std::iter::once(&page_size as &dyn rusqlite::ToSql))
        .chain(std::iter::once(&offset as &dyn rusqlite::ToSql))
        .collect();

    let songs = conn
        .prepare(&query)
        .map_err(|e| e.to_string())?
        .query_map(params_refs.as_slice(), |row| {
            Ok(Song {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration: row.get(4)?,
                video_path: row.get(5)?,
                vocal_audio_path: row.get(6)?,
                instrumental_audio_path: row.get(7)?,
                lyrics_path: row.get(8)?,
                lyrics_format: row.get(9)?,
                has_vocal: row.get::<_, i32>(10)? == 1,
                has_instrumental: row.get::<_, i32>(11)? == 1,
                genre: row.get(12)?,
                language: row.get(13)?,
                tags: row.get(14)?,
                difficulty: row.get(15)?,
                play_count: row.get(16)?,
                last_played_at: row.get(17)?,
                created_at: row.get(18)?,
                updated_at: row.get(19)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(songs)
}

#[tauri::command]
pub fn get_songs_count(
    db: State<Database>,
    search: Option<String>,
    artist: Option<String>,
    genre: Option<String>,
    language: Option<String>,
) -> Result<i32, String> {
    let conn = crate::db::get_connection(&db);

    let mut query = String::from("SELECT COUNT(*) FROM songs WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref s) = search {
        query.push_str(" AND (title LIKE ? OR artist LIKE ?)");
        let pattern = format!("%{}%", s);
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern));
    }

    if let Some(ref a) = artist {
        query.push_str(" AND artist = ?");
        params.push(Box::new(a.clone()));
    }

    if let Some(ref g) = genre {
        query.push_str(" AND genre = ?");
        params.push(Box::new(g.clone()));
    }

    if let Some(ref l) = language {
        query.push_str(" AND language = ?");
        params.push(Box::new(l.clone()));
    }

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let count: i32 = conn
        .query_row(&query, params_refs.as_slice(), |row| row.get(0))
        .map_err(|e| e.to_string())?;

    Ok(count)
}

#[tauri::command]
pub fn get_song_by_id(db: State<Database>, id: i64) -> Result<Option<Song>, String> {
    let conn = crate::db::get_connection(&db);

    let result = conn
        .query_row(
            "SELECT id, title, artist, album, duration, video_path, vocal_audio_path, \
             instrumental_audio_path, lyrics_path, lyrics_format, has_vocal, has_instrumental, \
             genre, language, tags, difficulty, play_count, last_played_at, created_at, updated_at \
             FROM songs WHERE id = ?",
            [id],
            |row| {
                Ok(Song {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    album: row.get(3)?,
                    duration: row.get(4)?,
                    video_path: row.get(5)?,
                    vocal_audio_path: row.get(6)?,
                    instrumental_audio_path: row.get(7)?,
                    lyrics_path: row.get(8)?,
                    lyrics_format: row.get(9)?,
                    has_vocal: row.get::<_, i32>(10)? == 1,
                    has_instrumental: row.get::<_, i32>(11)? == 1,
                    genre: row.get(12)?,
                    language: row.get(13)?,
                    tags: row.get(14)?,
                    difficulty: row.get(15)?,
                    play_count: row.get(16)?,
                    last_played_at: row.get(17)?,
                    created_at: row.get(18)?,
                    updated_at: row.get(19)?,
                })
            },
        );

    match result {
        Ok(song) => Ok(Some(song)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn add_song(db: State<Database>, song: NewSong) -> Result<i64, String> {
    let conn = crate::db::get_connection(&db);
    let tags_json = song.tags.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default());

    conn.execute(
        "INSERT INTO songs (title, artist, album, video_path, vocal_audio_path, \
         instrumental_audio_path, lyrics_path, genre, language, tags, duration, \
         has_vocal, has_instrumental) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        rusqlite::params![
            song.title,
            song.artist,
            song.album,
            song.video_path,
            song.vocal_audio_path,
            song.instrumental_audio_path,
            song.lyrics_path,
            song.genre,
            song.language,
            tags_json,
            song.duration,
            song.vocal_audio_path.is_some() as i32,
            song.instrumental_audio_path.is_some() as i32,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_song(db: State<Database>, song: UpdateSong) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);
    let tags_json = song.tags.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default());

    conn.execute(
        "UPDATE songs SET title = COALESCE(?1, title), artist = COALESCE(?2, artist), \
         album = COALESCE(?3, album), video_path = COALESCE(?4, video_path), \
         vocal_audio_path = COALESCE(?5, vocal_audio_path), \
         instrumental_audio_path = COALESCE(?6, instrumental_audio_path), \
         lyrics_path = COALESCE(?7, lyrics_path), genre = COALESCE(?8, genre), \
         language = COALESCE(?9, language), tags = COALESCE(?10, tags), \
         has_vocal = CASE WHEN ?5 IS NOT NULL THEN 1 ELSE has_vocal END, \
         has_instrumental = CASE WHEN ?6 IS NOT NULL THEN 1 ELSE has_instrumental END, \
         updated_at = CURRENT_TIMESTAMP WHERE id = ?11",
        rusqlite::params![
            song.title,
            song.artist,
            song.album,
            song.video_path,
            song.vocal_audio_path,
            song.instrumental_audio_path,
            song.lyrics_path,
            song.genre,
            song.language,
            tags_json,
            song.id,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn delete_song(db: State<Database>, id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);
    conn.execute("DELETE FROM songs WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn import_songs(
    db: State<Database>,
    directory: String,
    recursive: bool,
) -> Result<ImportResult, String> {
    let path = std::path::Path::new(&directory);
    if !path.exists() {
        return Err("路径不存在".to_string());
    }

    // 判断是文件还是目录
    let files = if path.is_file() {
        // 单个文件
        match file_scanner::scan_file(path) {
            Some(file) => vec![file],
            None => return Err("不支持的文件格式".to_string()),
        }
    } else {
        // 目录
        file_scanner::scan_directory(path, recursive)?
    };

    // 分组
    let groups = file_scanner::group_files_into_songs(files);

    let conn = crate::db::get_connection(&db);
    let mut result = ImportResult {
        success: 0,
        skipped: 0,
        failed: 0,
        errors: Vec::new(),
    };

    // 先处理有音视频的组
    for group in &groups {
        if group.video.is_none() && group.vocal_audio.is_none() && group.instrumental_audio.is_none() {
            continue;
        }

        // 提取元数据
        let metadata = if let Some(ref video) = group.video {
            metadata_extractor::extract_metadata(&video.path)
        } else if let Some(ref audio) = group.vocal_audio {
            metadata_extractor::extract_metadata(&audio.path)
        } else if let Some(ref audio) = group.instrumental_audio {
            metadata_extractor::extract_metadata(&audio.path)
        } else {
            None
        };

        let title = metadata.as_ref()
            .and_then(|m| m.title.clone())
            .unwrap_or_else(|| group.base_name.clone());

        // 检查是否已存在
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM songs WHERE title = ? AND (artist = ? OR (artist IS NULL AND ? IS NULL))",
                rusqlite::params![&title, metadata.as_ref().and_then(|m| m.artist.as_ref()), metadata.as_ref().and_then(|m| m.artist.as_ref())],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if exists {
            result.skipped += 1;
            continue;
        }

        // 插入数据库
        let video_path = group.video.as_ref().map(|f| f.path.to_string_lossy().to_string());
        let vocal_path = group.vocal_audio.as_ref().map(|f| f.path.to_string_lossy().to_string());
        let inst_path = group.instrumental_audio.as_ref().map(|f| f.path.to_string_lossy().to_string());
        let lyrics_path = group.lyrics.as_ref().map(|f| f.path.to_string_lossy().to_string());
        let lyrics_format = group.lyrics.as_ref().and_then(|f| {
            metadata_extractor::detect_lyrics_format(&f.path)
        });

        match conn.execute(
            "INSERT INTO songs (title, artist, album, duration, video_path, vocal_audio_path, \
             instrumental_audio_path, lyrics_path, lyrics_format, has_vocal, has_instrumental, genre) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                title,
                metadata.as_ref().and_then(|m| m.artist.clone()),
                metadata.as_ref().and_then(|m| m.album.clone()),
                metadata.as_ref().and_then(|m| m.duration),
                video_path,
                vocal_path,
                inst_path,
                lyrics_path,
                lyrics_format,
                group.vocal_audio.is_some() as i32,
                group.instrumental_audio.is_some() as i32,
                metadata.as_ref().and_then(|m| m.genre.clone()),
            ],
        ) {
            Ok(_) => result.success += 1,
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("导入 '{}' 失败: {}", title, e));
            }
        }
    }

    // 处理独立的歌词文件（匹配到数据库中已有的歌曲）
    for group in &groups {
        if group.video.is_some() || group.vocal_audio.is_some() || group.instrumental_audio.is_some() {
            continue;
        }

        // 只处理有歌词但没有音视频的组
        if let Some(ref lyrics) = group.lyrics {
            let base_name = &group.base_name;
            let lyrics_format = metadata_extractor::detect_lyrics_format(&lyrics.path);

            println!("[Library] Processing standalone lyrics: '{}' (base_name: '{}')", lyrics.file_name, base_name);

            // 尝试匹配已有歌曲（不区分大小写）
            let matched: Option<i64> = conn
                .query_row(
                    "SELECT id FROM songs WHERE title = ? COLLATE NOCASE OR title LIKE ? COLLATE NOCASE LIMIT 1",
                    rusqlite::params![base_name, format!("%{}%", base_name)],
                    |row| row.get(0),
                )
                .ok();

            if let Some(song_id) = matched {
                // 检查是否已有歌词
                let has_lyrics: bool = conn
                    .query_row(
                        "SELECT lyrics_path IS NOT NULL FROM songs WHERE id = ?",
                        [song_id],
                        |row| row.get::<_, i32>(0).map(|v| v == 1),
                    )
                    .unwrap_or(false);

                if has_lyrics {
                    println!("[Library] Song id {} already has lyrics, skipping", song_id);
                    result.skipped += 1;
                } else {
                    conn.execute(
                        "UPDATE songs SET lyrics_path = ?, lyrics_format = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                        rusqlite::params![lyrics.path.to_string_lossy().to_string(), lyrics_format, song_id],
                    )
                    .map_err(|e| e.to_string())?;

                    result.success += 1;
                    println!("[Library] Matched lyrics '{}' to existing song id {}", base_name, song_id);
                }
            } else {
                // 没有匹配的歌曲，跳过
                println!("[Library] No matching song found for lyrics '{}'", base_name);
                result.skipped += 1;
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn import_single_file(
    db: State<Database>,
    file_path: String,
) -> Result<i64, String> {
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err("文件不存在".to_string());
    }

    let scanned = file_scanner::scan_file(path)
        .ok_or("不支持的文件格式")?;

    let conn = crate::db::get_connection(&db);

    // 如果是歌词文件，尝试匹配已有歌曲
    if scanned.file_type == file_scanner::FileType::Lyrics {
        let base_name = file_scanner::extract_base_name(&scanned.file_name);
        let lyrics_format = metadata_extractor::detect_lyrics_format(&scanned.path);

        println!("[Library] Trying to match lyrics file: '{}' (base_name: '{}')", scanned.file_name, base_name);

        // 尝试通过标题匹配歌曲（精确匹配或模糊匹配）
        let song_id: Option<i64> = conn
            .query_row(
                "SELECT id, title FROM songs WHERE title = ? COLLATE NOCASE OR title LIKE ? COLLATE NOCASE LIMIT 1",
                rusqlite::params![&base_name, format!("%{}%", &base_name)],
                |row| {
                    let id: i64 = row.get(0)?;
                    let title: String = row.get(1)?;
                    println!("[Library] Matched song: id={}, title='{}'", id, title);
                    Ok(id)
                },
            )
            .ok();

        if let Some(id) = song_id {
            // 找到匹配的歌曲，更新歌词
            conn.execute(
                "UPDATE songs SET lyrics_path = ?, lyrics_format = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                rusqlite::params![&file_path, lyrics_format, id],
            )
            .map_err(|e| e.to_string())?;

            println!("[Library] Successfully matched lyrics '{}' to song id {}", &base_name, id);
            return Ok(id);
        }

        // 没有找到匹配的歌曲，列出数据库中的歌曲标题用于调试
        let songs: Vec<String> = conn
            .prepare("SELECT title FROM songs LIMIT 10")
            .ok()
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get(0))
                    .ok()
                    .and_then(|rows| rows.collect::<Result<Vec<_>, _>>().ok())
            })
            .unwrap_or_default();
        println!("[Library] No match found. Database songs: {:?}", songs);

        return Err(format!("未找到与歌词文件 '{}' 匹配的歌曲", &base_name));
    }

    // 提取元数据
    let metadata = metadata_extractor::extract_metadata(&scanned.path);

    let title = metadata.as_ref()
        .and_then(|m| m.title.clone())
        .unwrap_or_else(|| file_scanner::extract_base_name(&scanned.file_name));

    let (video_path, vocal_path, inst_path) = match scanned.file_type {
        file_scanner::FileType::Video => (Some(file_path.clone()), None, None),
        file_scanner::FileType::AudioVocal => (None, Some(file_path.clone()), None),
        file_scanner::FileType::AudioInstrumental => (None, None, Some(file_path.clone())),
        _ => (None, Some(file_path.clone()), None),
    };

    conn.execute(
        "INSERT INTO songs (title, artist, album, duration, video_path, vocal_audio_path, \
         instrumental_audio_path, has_vocal, has_instrumental, genre) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            title,
            metadata.as_ref().and_then(|m| m.artist.clone()),
            metadata.as_ref().and_then(|m| m.album.clone()),
            metadata.as_ref().and_then(|m| m.duration),
            video_path,
            vocal_path,
            inst_path,
            vocal_path.is_some() as i32,
            inst_path.is_some() as i32,
            metadata.as_ref().and_then(|m| m.genre.clone()),
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn get_tags(db: State<Database>, category: Option<String>) -> Result<Vec<Tag>, String> {
    let conn = crate::db::get_connection(&db);

    let tags = if let Some(cat) = category {
        conn.prepare("SELECT id, name, category, color FROM tags WHERE category = ? ORDER BY name")
            .map_err(|e| e.to_string())?
            .query_map([&cat], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    color: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        conn.prepare("SELECT id, name, category, color FROM tags ORDER BY category, name")
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    color: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(tags)
}

#[tauri::command]
pub fn add_tag(db: State<Database>, name: String, category: Option<String>, color: Option<String>) -> Result<i64, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "INSERT OR IGNORE INTO tags (name, category, color) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, category, color],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn get_artists(db: State<Database>) -> Result<Vec<String>, String> {
    let conn = crate::db::get_connection(&db);

    let artists = conn
        .prepare("SELECT DISTINCT artist FROM songs WHERE artist IS NOT NULL AND artist != '' ORDER BY artist")
        .map_err(|e| e.to_string())?
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(artists)
}

#[tauri::command]
pub fn get_genres(db: State<Database>) -> Result<Vec<String>, String> {
    let conn = crate::db::get_connection(&db);

    let genres = conn
        .prepare("SELECT DISTINCT genre FROM songs WHERE genre IS NOT NULL AND genre != '' ORDER BY genre")
        .map_err(|e| e.to_string())?
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(genres)
}

#[tauri::command]
pub fn get_languages(db: State<Database>) -> Result<Vec<String>, String> {
    let conn = crate::db::get_connection(&db);

    let languages = conn
        .prepare("SELECT DISTINCT language FROM songs WHERE language IS NOT NULL AND language != '' ORDER BY language")
        .map_err(|e| e.to_string())?
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(languages)
}

/// 为歌曲导入原唱音频
#[tauri::command]
pub fn import_vocal(db: State<Database>, songId: i64, filePath: String) -> Result<bool, String> {
    let path = std::path::Path::new(&filePath);
    if !path.exists() {
        return Err("文件不存在".to_string());
    }

    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE songs SET vocal_audio_path = ?, has_vocal = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        rusqlite::params![filePath, songId],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

/// 为歌曲导入歌词文件
#[tauri::command]
pub fn import_lyrics(db: State<Database>, songId: i64, filePath: String) -> Result<bool, String> {
    let path = std::path::Path::new(&filePath);
    if !path.exists() {
        return Err("文件不存在".to_string());
    }

    // 检测歌词格式
    let lyrics_format = metadata_extractor::detect_lyrics_format(path);

    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE songs SET lyrics_path = ?, lyrics_format = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        rusqlite::params![filePath, lyrics_format, songId],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

/// 更新歌曲元数据（同时更新数据库和文件）
#[tauri::command]
pub fn update_song_metadata(
    db: State<Database>,
    songId: i64,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    // 获取歌曲信息
    let (video_path, vocal_path, inst_path): (Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT video_path, vocal_audio_path, instrumental_audio_path FROM songs WHERE id = ?",
            [songId],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("歌曲不存在: {}", e))?;

    // 确定要修改的文件（优先修改音频文件）
    let file_to_update = vocal_path
        .or(inst_path)
        .or(video_path);

    // 创建元数据
    let metadata = metadata_extractor::SongMetadata {
        title: title.clone(),
        artist: artist.clone(),
        album: album.clone(),
        ..Default::default()
    };

    // 尝试写入文件元数据
    if let Some(ref file_path_str) = file_to_update {
        let file_path = std::path::Path::new(file_path_str);
        if file_path.exists() && metadata_extractor::can_write_metadata(file_path) {
            match metadata_extractor::write_metadata(file_path, &metadata) {
                Ok(_) => {
                    println!("[Library] Successfully updated metadata in file: {}", file_path_str);
                }
                Err(e) => {
                    println!("[Library] Warning: Failed to update file metadata: {}", e);
                    // 文件写入失败不影响数据库更新
                }
            }
        }
    }

    // 更新数据库
    conn.execute(
        "UPDATE songs SET title = COALESCE(?1, title), artist = COALESCE(?2, artist), \
         album = COALESCE(?3, album), updated_at = CURRENT_TIMESTAMP WHERE id = ?4",
        rusqlite::params![title, artist, album, songId],
    )
    .map_err(|e| format!("更新数据库失败: {}", e))?;

    Ok(true)
}
