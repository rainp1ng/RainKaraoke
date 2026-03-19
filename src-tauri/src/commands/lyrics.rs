use tauri::State;
use crate::db::Database;
use crate::modules::lyrics_parser::{self, Lyrics, LyricsLine};
use std::path::PathBuf;

#[tauri::command]
pub fn get_lyrics(db: State<Database>, song_id: i64) -> Result<Option<Lyrics>, String> {
    let conn = crate::db::get_connection(&db);

    // 获取歌词文件路径
    let result = conn.query_row(
        "SELECT lyrics_path, lyrics_format FROM songs WHERE id = ?",
        [song_id],
        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?)),
    );

    match result {
        Ok((Some(path), _)) => {
            let path = PathBuf::from(&path);
            Ok(lyrics_parser::parse_lyrics_file(&path))
        }
        Ok((None, _)) => Ok(None),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn parse_lyrics_content(content: String, format: String) -> Result<Lyrics, String> {
    let format = match format.as_str() {
        "lrc" => lyrics_parser::LyricsFormat::Lrc,
        "ksc" => lyrics_parser::LyricsFormat::Ksc,
        "txt" => lyrics_parser::LyricsFormat::Txt,
        _ => return Err("不支持的歌词格式".to_string()),
    };

    Ok(lyrics_parser::parse_lyrics(&content, format))
}

#[tauri::command]
pub fn get_current_lyrics_line(
    lines: Vec<LyricsLine>,
    time_ms: u64,
) -> Option<usize> {
    lyrics_parser::get_current_line_index(&lines, time_ms)
}
