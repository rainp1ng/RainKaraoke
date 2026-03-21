use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 支持的视频格式
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "flv", "webm", "m4v", "wmv"];

/// 支持的音频格式
pub const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ape", "aac", "ogg", "wav", "m4a", "wma"];

/// 支持的歌词格式
pub const LYRICS_EXTENSIONS: &[&str] = &["lrc", "ksc", "txt"];

/// 伴奏标识符
pub const INSTRUMENTAL_KEYWORDS: &[&str] = &[
    "instrumental", "伴奏", "karaoke", "offvocal", "inst",
    "backing", "minusone", "no_vocal", "伴唱"
];

/// 原唱标识符
pub const VOCAL_KEYWORDS: &[&str] = &[
    "vocal", "原唱", "original", "with_vocal", "唱歌"
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub file_type: FileType,
    pub file_name: String,
    pub extension: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Video,
    AudioVocal,
    AudioInstrumental,
    AudioUnknown,
    Lyrics,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongGroup {
    /// 基础文件名（用于匹配）
    pub base_name: String,
    /// 视频文件
    pub video: Option<ScannedFile>,
    /// 原唱音频
    pub vocal_audio: Option<ScannedFile>,
    /// 伴奏音频
    pub instrumental_audio: Option<ScannedFile>,
    /// 歌词文件
    pub lyrics: Option<ScannedFile>,
    /// 其他音频文件
    pub other_audio: Vec<ScannedFile>,
}

/// 扫描目录中的媒体文件
pub fn scan_directory(
    directory: &Path,
    recursive: bool,
) -> Result<Vec<ScannedFile>, String> {
    let mut files = Vec::new();

    let walk = if recursive {
        WalkDir::new(directory).into_iter()
    } else {
        WalkDir::new(directory).max_depth(1).into_iter()
    };

    for entry in walk {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(file) = scan_file(path) {
            files.push(file);
        }
    }

    Ok(files)
}

/// 扫描单个文件
pub fn scan_file(path: &Path) -> Option<ScannedFile> {
    let extension = path.extension()?
        .to_string_lossy()
        .to_lowercase();

    let file_type = determine_file_type(&extension);
    if file_type == FileType::Unknown {
        return None;
    }

    let file_name = path.file_name()?
        .to_string_lossy()
        .to_string();

    let size = path.metadata().ok()?.len();

    Some(ScannedFile {
        path: path.to_path_buf(),
        file_type,
        file_name,
        extension,
        size,
    })
}

/// 根据扩展名判断文件类型
pub fn determine_file_type(extension: &str) -> FileType {
    if VIDEO_EXTENSIONS.contains(&extension) {
        return FileType::Video;
    }

    if LYRICS_EXTENSIONS.contains(&extension) {
        return FileType::Lyrics;
    }

    if AUDIO_EXTENSIONS.contains(&extension) {
        return FileType::AudioUnknown; // 需要根据文件名进一步判断
    }

    FileType::Unknown
}

/// 根据文件名判断音频类型（原唱/伴奏）
pub fn determine_audio_type(file_name: &str) -> FileType {
    let lower_name = file_name.to_lowercase();

    // 检查是否为伴奏
    for keyword in INSTRUMENTAL_KEYWORDS {
        if lower_name.contains(keyword) {
            return FileType::AudioInstrumental;
        }
    }

    // 检查是否为原唱
    for keyword in VOCAL_KEYWORDS {
        if lower_name.contains(keyword) {
            return FileType::AudioVocal;
        }
    }

    FileType::AudioUnknown
}

/// 提取基础文件名（去除后缀和标识符）
pub fn extract_base_name(file_name: &str) -> String {
    let name = file_name
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(file_name);

    // 移除已知的标识符（不区分大小写）
    let mut cleaned = name.to_string();
    for keyword in INSTRUMENTAL_KEYWORDS.iter().chain(VOCAL_KEYWORDS.iter()) {
        // 构建要匹配的模式（带分隔符）
        let patterns = vec![
            format!("_{}", keyword),
            format!("-{}", keyword),
            format!(" {}", keyword),
            format!("[{}]", keyword),
            format!("({})", keyword),
        ];

        for pattern in patterns {
            // 不区分大小写查找并替换
            let lower_cleaned = cleaned.to_lowercase();
            let lower_pattern = pattern.to_lowercase();
            if let Some(pos) = lower_cleaned.find(&lower_pattern) {
                let end = pos + pattern.len();
                if end <= cleaned.len() {
                    cleaned = format!("{}{}", &cleaned[..pos], &cleaned[end..]);
                }
            }
        }
    }

    cleaned.trim().to_string()
}

/// 将扫描的文件分组为歌曲
pub fn group_files_into_songs(files: Vec<ScannedFile>) -> Vec<SongGroup> {
    let mut groups: HashMap<String, SongGroup> = HashMap::new();

    for mut file in files {
        // 更新音频类型
        if file.file_type == FileType::AudioUnknown {
            file.file_type = determine_audio_type(&file.file_name);
        }

        let base_name = extract_base_name(&file.file_name);

        let group = groups.entry(base_name.clone()).or_insert_with(|| SongGroup {
            base_name,
            video: None,
            vocal_audio: None,
            instrumental_audio: None,
            lyrics: None,
            other_audio: Vec::new(),
        });

        match file.file_type {
            FileType::Video => {
                // 如果已有视频，比较大小保留较大的
                if let Some(existing) = &group.video {
                    if file.size > existing.size {
                        group.video = Some(file);
                    }
                } else {
                    group.video = Some(file);
                }
            }
            FileType::AudioVocal => {
                group.vocal_audio = Some(file);
            }
            FileType::AudioInstrumental => {
                group.instrumental_audio = Some(file);
            }
            FileType::Lyrics => {
                // 优先使用 LRC 格式
                if group.lyrics.is_none() || file.extension == "lrc" {
                    group.lyrics = Some(file);
                }
            }
            FileType::AudioUnknown => {
                group.other_audio.push(file);
            }
            FileType::Unknown => {}
        }
    }

    // 如果没有伴奏但有其他音频，将第一个其他音频作为伴奏（默认导入的是伴奏）
    for group in groups.values_mut() {
        if group.instrumental_audio.is_none() && !group.other_audio.is_empty() {
            group.instrumental_audio = group.other_audio.pop();
        }
        // 如果仍有其他音频且没有原唱，可以作为原唱候选
        if group.vocal_audio.is_none() && !group.other_audio.is_empty() {
            group.vocal_audio = group.other_audio.pop();
        }
    }

    // 过滤掉没有任何有用文件的组（保留有音视频或歌词的组）
    groups.into_values()
        .filter(|g| g.video.is_some() || g.vocal_audio.is_some() || g.instrumental_audio.is_some() || g.lyrics.is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_name() {
        assert_eq!(extract_base_name("测试歌曲_伴奏.mp3"), "测试歌曲");
        assert_eq!(extract_base_name("测试歌曲(instrumental).mp3"), "测试歌曲");
        assert_eq!(extract_base_name("测试歌曲.mp3"), "测试歌曲");
    }

    #[test]
    fn test_determine_audio_type() {
        assert_eq!(determine_audio_type("歌曲_伴奏.mp3"), FileType::AudioInstrumental);
        assert_eq!(determine_audio_type("歌曲(instrumental).mp3"), FileType::AudioInstrumental);
        assert_eq!(determine_audio_type("歌曲_vocal.mp3"), FileType::AudioVocal);
        assert_eq!(determine_audio_type("歌曲.mp3"), FileType::AudioUnknown);
    }
}
