use std::path::Path;
use std::time::Duration;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SongMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<u32>,  // 秒
    pub genre: Option<String>,
    pub year: Option<u32>,
}

/// 从文件中提取元数据
pub fn extract_metadata(file_path: &Path) -> Option<SongMetadata> {
    let extension = file_path.extension()?
        .to_string_lossy()
        .to_lowercase();

    // 视频文件使用文件名解析
    if crate::utils::file_scanner::VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        return parse_from_filename(file_path);
    }

    // 音频文件尝试读取元数据
    if crate::utils::file_scanner::AUDIO_EXTENSIONS.contains(&extension.as_str()) {
        // 先尝试从文件读取
        if let Some(metadata) = read_audio_metadata(file_path) {
            // 如果标题为空，从文件名解析
            if metadata.title.is_none() {
                let from_filename = parse_from_filename(file_path);
                return Some(SongMetadata {
                    title: from_filename.as_ref().and_then(|m| m.title.clone())
                        .or(metadata.title),
                    artist: metadata.artist.or(from_filename.and_then(|m| m.artist)),
                    album: metadata.album,
                    duration: metadata.duration,
                    genre: metadata.genre,
                    year: metadata.year,
                });
            }
            return Some(metadata);
        }

        // 读取失败，从文件名解析
        return parse_from_filename(file_path);
    }

    // 其他文件类型从文件名解析
    parse_from_filename(file_path)
}

/// 从音频文件读取元数据
fn read_audio_metadata(file_path: &Path) -> Option<SongMetadata> {
    let probe = Probe::open(file_path).ok()?;
    let tagged_file = probe.read().ok()?;

    let properties = tagged_file.properties();
    let duration = properties.duration().as_secs() as u32;

    // 尝试获取标签
    let tag = tagged_file.primary_tag()?;

    let title = tag.title().map(|s| s.to_string());
    let artist = tag.artist().map(|s| s.to_string());
    let album = tag.album().map(|s| s.to_string());
    let genre = tag.genre().map(|s| s.to_string());
    let year = tag.year().map(|y| y as u32);

    Some(SongMetadata {
        title,
        artist,
        album,
        duration: Some(duration),
        genre,
        year,
    })
}

/// 从文件名解析元数据
pub fn parse_from_filename(file_path: &Path) -> Option<SongMetadata> {
    let file_name = file_path.file_stem()?
        .to_string_lossy()
        .to_string();

    // 清理标识符
    let cleaned_name = crate::utils::file_scanner::extract_base_name(&file_name);

    // 尝试匹配各种格式
    // 格式1: 歌手 - 歌曲名
    // 格式2: 歌手-歌曲名
    // 格式3: 歌手：歌曲名
    // 格式4: 歌曲名
    // 格式5: 歌手《歌曲名》
    // 格式6: 歌手【歌曲名】

    let patterns = vec![
        // "歌手 - 歌曲名" 或 "歌手-歌曲名"
        r"^(.+?)\s*[-—]\s*(.+)$",
        // "歌手：歌曲名" 或 "歌手:歌曲名"
        r"^(.+?)[:：]\s*(.+)$",
        // "歌手《歌曲名》"
        r"^(.+?)《(.+)》$",
        // "歌手【歌曲名】"
        r"^(.+?)【(.+)】$",
        // "歌手(歌曲名)" 或 "歌手（歌曲名）"
        r"^(.+?)[(（](.+)[)）]$",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(&cleaned_name) {
                let artist = caps.get(1).map(|m| m.as_str().trim().to_string());
                let title = caps.get(2).map(|m| m.as_str().trim().to_string());

                if let Some(t) = &title {
                    return Some(SongMetadata {
                        title: Some(t.clone()),
                        artist,
                        album: None,
                        duration: None,
                        genre: None,
                        year: None,
                    });
                }
            }
        }
    }

    // 无法解析，使用清理后的文件名作为标题
    Some(SongMetadata {
        title: Some(cleaned_name),
        artist: None,
        album: None,
        duration: None,
        genre: None,
        year: None,
    })
}

/// 检测歌词格式
pub fn detect_lyrics_format(file_path: &Path) -> Option<String> {
    let extension = file_path.extension()?
        .to_string_lossy()
        .to_lowercase();

    match extension.as_str() {
        "lrc" => Some("lrc".to_string()),
        "ksc" => Some("ksc".to_string()),
        "txt" => Some("txt".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_from_filename() {
        // 格式1
        let meta = parse_from_filename_stem("周杰伦 - 晴天");
        assert_eq!(meta.artist, Some("周杰伦".to_string()));
        assert_eq!(meta.title, Some("晴天".to_string()));

        // 格式2
        let meta = parse_from_filename_stem("邓紫棋《光年之外》");
        assert_eq!(meta.artist, Some("邓紫棋".to_string()));
        assert_eq!(meta.title, Some("光年之外".to_string()));

        // 纯文件名
        let meta = parse_from_filename_stem("测试歌曲");
        assert_eq!(meta.title, Some("测试歌曲".to_string()));
    }

    fn parse_from_filename_stem(name: &str) -> SongMetadata {
        let fake_path = std::path::Path::new(name).with_extension("mp3");
        parse_from_filename(&fake_path).unwrap_or_default()
    }
}
