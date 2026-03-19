use serde::{Deserialize, Serialize};
use regex::Regex;
use std::fs;
use std::path::Path;

/// 歌词行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsLine {
    /// 开始时间（毫秒）
    pub time: u64,
    /// 持续时间（毫秒），可选
    pub duration: Option<u64>,
    /// 歌词文本
    pub text: String,
    /// 逐字信息，KSC格式有
    pub words: Option<Vec<LyricsWord>>,
}

/// 逐字歌词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsWord {
    /// 开始时间（毫秒）
    pub time: u64,
    /// 持续时间（毫秒）
    pub duration: u64,
    /// 文字
    pub text: String,
}

/// 歌词格式
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LyricsFormat {
    Lrc,
    Ksc,
    Txt,
}

/// 歌词内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lyrics {
    pub format: LyricsFormat,
    pub lines: Vec<LyricsLine>,
}

/// 从文件解析歌词
pub fn parse_lyrics_file(path: &Path) -> Option<Lyrics> {
    let content = fs::read_to_string(path).ok()?;
    let extension = path.extension()?.to_string_lossy().to_lowercase();

    let format = match extension.as_str() {
        "lrc" => LyricsFormat::Lrc,
        "ksc" => LyricsFormat::Ksc,
        "txt" => LyricsFormat::Txt,
        _ => return None,
    };

    Some(parse_lyrics(&content, format))
}

/// 解析歌词内容
pub fn parse_lyrics(content: &str, format: LyricsFormat) -> Lyrics {
    let lines = match format {
        LyricsFormat::Lrc => parse_lrc(content),
        LyricsFormat::Ksc => parse_ksc(content),
        LyricsFormat::Txt => parse_txt(content),
    };

    Lyrics { format, lines }
}

/// 解析 LRC 格式
pub fn parse_lrc(content: &str) -> Vec<LyricsLine> {
    let mut lines = Vec::new();
    let time_regex = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\]").unwrap();

    for line in content.lines() {
        let line = line.trim();

        // 跳过元数据行
        if line.starts_with("[ti:") || line.starts_with("[ar:")
            || line.starts_with("[al:") || line.starts_with("[by:")
            || line.starts_with("[offset:") {
            continue;
        }

        // 解析时间标签
        if let Some(caps) = time_regex.captures(line) {
            let minutes: u64 = caps[1].parse().unwrap_or(0);
            let seconds: u64 = caps[2].parse().unwrap_or(0);
            let ms_str = &caps[3];
            let milliseconds: u64 = if ms_str.len() == 2 {
                ms_str.parse::<u64>().unwrap_or(0) * 10
            } else {
                ms_str.parse().unwrap_or(0)
            };

            let time = minutes * 60 * 1000 + seconds * 1000 + milliseconds;

            // 获取歌词文本（移除时间标签）
            let text = time_regex.replace_all(line, "").to_string();
            let text = text.trim().to_string();

            if !text.is_empty() {
                lines.push(LyricsLine {
                    time,
                    duration: None,
                    text,
                    words: None,
                });
            }
        }
    }

    // 按时间排序
    lines.sort_by_key(|l| l.time);

    // 计算每行的持续时间
    for i in 0..lines.len().saturating_sub(1) {
        lines[i].duration = Some(lines[i + 1].time - lines[i].time);
    }

    lines
}

/// 解析 KSC 格式（逐字歌词）
pub fn parse_ksc(content: &str) -> Vec<LyricsLine> {
    let mut lines = Vec::new();

    // KSC 格式示例：
    // karaoke.song('歌曲名', '歌手名');
    // karaoke.add('00:00.000', '00:05.000', '歌词内容', '100,200,300,...');

    let line_regex = Regex::new(
        r"karaoke\.add\('(\d{2}):(\d{2})\.(\d{3})',\s*'(\d{2}):(\d{2})\.(\d{3})',\s*'([^']*)',\s*'([^']*)'\)"
    ).unwrap();

    for line in content.lines() {
        if let Some(caps) = line_regex.captures(line) {
            let start_min: u64 = caps[1].parse().unwrap_or(0);
            let start_sec: u64 = caps[2].parse().unwrap_or(0);
            let start_ms: u64 = caps[3].parse().unwrap_or(0);
            let start_time = start_min * 60 * 1000 + start_sec * 1000 + start_ms;

            let end_min: u64 = caps[4].parse().unwrap_or(0);
            let end_sec: u64 = caps[5].parse().unwrap_or(0);
            let end_ms: u64 = caps[6].parse().unwrap_or(0);
            let end_time = end_min * 60 * 1000 + end_sec * 1000 + end_ms;

            let text = caps[7].to_string();
            let durations_str = caps[8].to_string();

            // 解析逐字时长
            let words = parse_ksc_words(&text, &durations_str, start_time);

            lines.push(LyricsLine {
                time: start_time,
                duration: Some(end_time - start_time),
                text,
                words,
            });
        }
    }

    lines.sort_by_key(|l| l.time);
    lines
}

/// 解析 KSC 逐字时长
fn parse_ksc_words(text: &str, durations_str: &str, start_time: u64) -> Option<Vec<LyricsWord>> {
    let durations: Vec<u64> = durations_str
        .split(',')
        .filter_map(|s| s.parse().ok())
        .collect();

    let chars: Vec<char> = text.chars().collect();

    if durations.len() != chars.len() {
        return None;
    }

    let mut words = Vec::new();
    let mut current_time = start_time;

    for (i, &char) in chars.iter().enumerate() {
        let duration = durations.get(i).copied().unwrap_or(100);

        words.push(LyricsWord {
            time: current_time,
            duration,
            text: char.to_string(),
        });

        current_time += duration;
    }

    Some(words)
}

/// 解析纯文本格式
pub fn parse_txt(content: &str) -> Vec<LyricsLine> {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(i, line)| LyricsLine {
            time: (i as u64 + 1) * 5000, // 每行5秒
            duration: Some(5000),
            text: line.trim().to_string(),
            words: None,
        })
        .collect()
}

/// 根据时间获取当前歌词行索引
pub fn get_current_line_index(lines: &[LyricsLine], time_ms: u64) -> Option<usize> {
    for (i, line) in lines.iter().enumerate() {
        let next_time = lines.get(i + 1).map(|l| l.time).unwrap_or(u64::MAX);
        if time_ms >= line.time && time_ms < next_time {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc() {
        let content = r#"
[ti:测试歌曲]
[ar:测试歌手]
[00:00.00]第一行歌词
[00:05.50]第二行歌词
[00:10.30]第三行歌词
"#;

        let lines = parse_lrc(content);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text, "第一行歌词");
        assert_eq!(lines[0].time, 0);
        assert_eq!(lines[1].time, 5500);
    }

    #[test]
    fn test_parse_txt() {
        let content = "第一行\n第二行\n第三行";
        let lines = parse_txt(content);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text, "第一行");
        assert_eq!(lines[1].time, 10000);
    }
}
