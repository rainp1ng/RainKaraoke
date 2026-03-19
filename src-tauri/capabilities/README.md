# RainKaraoke - 配置说明

## 权限配置

本应用需要以下权限：

### 文件系统
- 读取媒体文件（视频、音频、歌词）
- 扫描文件夹

### 音频
- 访问音频输入设备（麦克风）
- 访问音频输出设备

### MIDI
- 访问 MIDI 输入设备

## macOS 权限

在 macOS 上，应用首次运行时会请求以下权限：

1. **麦克风访问** - 用于人声效果器链
2. **文件访问** - 用于扫描和播放媒体文件

## 配置文件

配置存储在 `~/.rainkaraoke/` 目录下：

```
~/.rainkaraoke/
├── data/
│   └── rainkaraoke.db    # SQLite 数据库
├── cache/                 # 缓存目录
└── logs/                  # 日志目录
```

## 支持的文件格式

### 视频
- MP4, MKV, AVI, MOV, FLV, WebM, M4V, WMV

### 音频
- MP3, FLAC, APE, AAC, OGG, WAV, M4A, WMA

### 歌词
- LRC (时间轴歌词)
- KSC (逐字歌词)
- TXT (纯文本歌词)

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| Space | 播放/暂停 |
| Left | 上一首 |
| Right | 下一首 |
| Up | 音量增加 |
| Down | 音量减少 |
| V | 切换原唱/伴奏 |
| Cmd+F | 搜索 |
| Cmd+, | 设置 |
