use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::utils::file_scanner::{ScannedFile, FileType};

/// 文件匹配器，用于关联相关文件
pub struct FileMatcher {
    /// 基础名称到文件的映射
    name_map: HashMap<String, Vec<ScannedFile>>,
}

impl FileMatcher {
    pub fn new() -> Self {
        Self {
            name_map: HashMap::new(),
        }
    }

    /// 添加文件到匹配器
    pub fn add_file(&mut self, file: ScannedFile) {
        let base_name = crate::utils::file_scanner::extract_base_name(&file.file_name);
        self.name_map.entry(base_name).or_default().push(file);
    }

    /// 批量添加文件
    pub fn add_files(&mut self, files: Vec<ScannedFile>) {
        for file in files {
            self.add_file(file);
        }
    }

    /// 获取所有匹配组
    pub fn get_groups(&self) -> Vec<FileGroup> {
        self.name_map.iter().map(|(name, files)| {
            let mut group = FileGroup::new(name.clone());

            for file in files {
                match file.file_type {
                    FileType::Video => {
                        if group.video.is_none() || file.size > group.video.as_ref().unwrap().size {
                            group.video = Some(file.clone());
                        }
                    }
                    FileType::AudioVocal => {
                        group.vocal_audio = Some(file.clone());
                    }
                    FileType::AudioInstrumental => {
                        group.instrumental_audio = Some(file.clone());
                    }
                    FileType::Lyrics => {
                        if group.lyrics.is_none() || file.extension == "lrc" {
                            group.lyrics = Some(file.clone());
                        }
                    }
                    FileType::AudioUnknown => {
                        // 如果还没有原唱或伴奏，尝试猜测
                        if group.vocal_audio.is_none() && group.instrumental_audio.is_none() {
                            group.vocal_audio = Some(file.clone());
                        }
                    }
                    FileType::Unknown => {}
                }
            }

            group
        })
        .filter(|g| g.has_media())
        .collect()
    }
}

/// 文件组，包含同一首歌的所有相关文件
#[derive(Debug, Clone)]
pub struct FileGroup {
    pub base_name: String,
    pub video: Option<ScannedFile>,
    pub vocal_audio: Option<ScannedFile>,
    pub instrumental_audio: Option<ScannedFile>,
    pub lyrics: Option<ScannedFile>,
}

impl FileGroup {
    pub fn new(base_name: String) -> Self {
        Self {
            base_name,
            video: None,
            vocal_audio: None,
            instrumental_audio: None,
            lyrics: None,
        }
    }

    /// 是否有媒体文件
    pub fn has_media(&self) -> bool {
        self.video.is_some() ||
        self.vocal_audio.is_some() ||
        self.instrumental_audio.is_some()
    }

    /// 是否有独立原唱
    pub fn has_vocal(&self) -> bool {
        self.vocal_audio.is_some()
    }

    /// 是否有独立伴奏
    pub fn has_instrumental(&self) -> bool {
        self.instrumental_audio.is_some()
    }

    /// 获取主要文件路径（优先视频，其次原唱音频）
    pub fn primary_file(&self) -> Option<&PathBuf> {
        if let Some(ref video) = self.video {
            return Some(&video.path);
        }
        if let Some(ref vocal) = self.vocal_audio {
            return Some(&vocal.path);
        }
        if let Some(ref inst) = self.instrumental_audio {
            return Some(&inst.path);
        }
        None
    }
}
