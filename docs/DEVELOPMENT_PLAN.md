# RainKaraoke 开发方案

## 一、技术架构详解

### 1.1 整体架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Application Layer                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                      Frontend (React + TypeScript)                   │   │
│   │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │   │
│   │  │   媒体库    │ │   播放器    │ │  效果器链   │ │   设置      │   │   │
│   │  │  Library    │ │   Player    │ │EffectChain │ │  Settings   │   │   │
│   │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │   │
│   │  ┌─────────────────────────────────────────────────────────────┐   │   │
│   │  │              State Management (Zustand)                      │   │   │
│   │  └─────────────────────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                          Tauri IPC │ (invoke / events)                      │
│                                    ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        Backend (Rust + Tauri)                       │   │
│   │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │   │
│   │  │  Commands   │ │   Models    │ │     DB      │ │   Modules   │   │   │
│   │  │   (API)     │ │  (Data)     │ │  (SQLite)   │ │  (Core)     │   │   │
│   │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         Native Libraries                             │   │
│   │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐      │   │
│   │  │ FFmpeg  │ │  Rodio  │ │  CPAL   │ │  Midir  │ │   VAD   │      │   │
│   │  │(Video)  │ │ (Audio) │ │(AudioIO)│ │ (MIDI)  │ │ (Voice) │      │   │
│   │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘      │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 数据流架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         数据流向                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  用户操作 ──> React组件 ──> Zustand Store ──> Tauri Invoke     │
│                                                    │            │
│                                                    ▼            │
│                                              Rust Command       │
│                                                    │            │
│                                    ┌───────────────┼────────┐   │
│                                    ▼               ▼        ▼   │
│                              SQLite DB      Media Engine  Audio │
│                                    │               │        │   │
│                                    └───────┬───────┴────────┘   │
│                                            │                    │
│                                            ▼                    │
│                                      Tauri Event                 │
│                                            │                    │
│                                            ▼                    │
│                             React组件 (状态更新)                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 二、模块详细设计

### 2.1 媒体库模块 (Media Library)

#### 2.1.1 功能需求
- 文件导入：拖拽、文件夹扫描、单个文件添加
- 自动识别：视频、原唱音频、伴唱音频、歌词文件的自动关联
- 元数据提取：从文件名、ID3标签、视频元数据提取歌曲信息
- 标签管理：自动标签（歌手、风格、语言）+ 自定义标签
- 搜索过滤：全文搜索 + 多条件组合筛选

#### 2.1.2 技术实现

**文件扫描策略**
```rust
// 支持的文件格式
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "flv", "webm"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ape", "aac", "ogg", "wav", "m4a"];
const LYRICS_EXTENSIONS: &[&str] = &["lrc", "ksc", "txt"];

// 文件关联规则
// 1. 同名文件自动关联（如：歌曲名.mp4 + 歌曲名.lrc）
// 2. 后缀识别（如：歌曲名_伴奏.mp3）
// 3. 目录结构识别（如：/歌手/专辑/歌曲名.mp4）
```

**元数据提取流程**
```
1. 文件名解析
   └─> 尝试匹配格式: "歌手 - 歌名.mp4", "歌名.mp4", "歌名(歌手).mp4"

2. ID3/元数据读取
   └─> 使用 ffmpeg/rodio 读取内嵌元数据

3. 歌词文件匹配
   └─> 查找同名 .lrc / .ksc 文件

4. 伴奏/原唱识别
   └─> 文件名包含: instrumental, 伴奏, karaoke, offvocal
   └─> 文件名包含: vocal, 原唱
```

**数据库索引优化**
```sql
-- 全文搜索虚拟表
CREATE VIRTUAL TABLE songs_fts USING fts5(
    title, artist, album, genre,
    content='songs',
    content_rowid='id'
);

-- 触发器保持同步
CREATE TRIGGER songs_ai AFTER INSERT ON songs BEGIN
    INSERT INTO songs_fts(rowid, title, artist, album, genre)
    VALUES (new.id, new.title, new.artist, new.album, new.genre);
END;
```

#### 2.1.3 前端组件设计
```typescript
// components/Library/
├── Library.tsx           // 主容器
├── SearchBar.tsx         // 搜索栏
├── FilterPanel.tsx       // 筛选面板
├── SongTable.tsx         // 歌曲列表表格
├── SongRow.tsx           // 单行歌曲
├── ImportDialog.tsx      // 导入对话框
├── SongEditDialog.tsx    // 歌曲编辑对话框
└── TagManager.tsx        // 标签管理
```

---

### 2.2 播放引擎模块 (Playback Engine)

#### 2.2.1 功能需求
- 多格式视频播放：MP4, MKV, AVI, MOV 等
- 音频切换：原唱/伴唱无缝切换，保持播放位置
- 播放控制：播放/暂停/停止/跳转
- 音调调节：-12 ~ +12 半音
- 速度调节：0.5x ~ 2.0x

#### 2.2.2 技术实现

**核心架构**
```rust
pub struct PlaybackEngine {
    // 播放状态
    state: Arc<Mutex<PlaybackState>>,

    // 视频解码器
    video_decoder: Option<VideoDecoder>,

    // 音频播放器
    audio_player: AudioPlayer,

    // 当前歌曲
    current_song: Option<Song>,

    // 输出设备
    output_device: OutputDevice,
}

pub struct PlaybackState {
    status: PlaybackStatus,      // Idle/Playing/Paused
    current_time: Duration,
    duration: Duration,
    is_vocal: bool,              // 原唱/伴唱
    pitch: i32,                  // 音调偏移
    speed: f32,                  // 播放速度
}
```

**音频切换实现**
```rust
impl PlaybackEngine {
    pub fn toggle_vocal(&mut self, is_vocal: bool) -> Result<()> {
        let current_pos = self.audio_player.position();

        // 淡出当前音频
        self.audio_player.fade_out(Duration::from_millis(50))?;

        // 切换音频源
        let audio_path = if is_vocal {
            &self.current_song.vocal_audio_path
        } else {
            &self.current_song.instrumental_audio_path
        };

        // 加载新音频
        self.audio_player.load(audio_path)?;

        // 跳转到相同位置
        self.audio_player.seek_to(current_pos)?;

        // 淡入新音频
        self.audio_player.fade_in(Duration::from_millis(50))?;

        self.state.is_vocal = is_vocal;
        Ok(())
    }
}
```

**音调调节 (Pitch Shifting)**
```rust
// 使用 rubato 进行音频重采样实现变调
pub fn apply_pitch_shift(
    input: &[f32],
    sample_rate: u32,
    semitones: i32,
) -> Vec<f32> {
    // 计算变调比率
    let ratio = 2f32.powf(semitones as f32 / 12.0);

    // 使用 rubato 进行重采样
    let resampler = rubato::FftFixedIn::<f32>::new(
        sample_rate as usize,
        (sample_rate as f32 * ratio) as usize,
        1024,
        2,
    );

    // ... 重采样处理
}
```

#### 2.2.3 前端状态管理
```typescript
// stores/playbackStore.ts
interface PlaybackStore {
  // 状态
  status: 'idle' | 'playing' | 'paused'
  currentSong: Song | null
  currentTime: number
  duration: number
  isVocal: boolean
  pitch: number
  speed: number

  // 操作
  play: (songId: number) => Promise<void>
  pause: () => Promise<void>
  resume: () => Promise<void>
  stop: () => Promise<void>
  seek: (time: number) => Promise<void>
  toggleVocal: (isVocal: boolean) => Promise<void>
  setPitch: (semitones: number) => Promise<void>
  setSpeed: (speed: number) => Promise<void>
}
```

---

### 2.3 歌词显示模块 (Lyrics Display)

#### 2.3.1 功能需求
- 支持格式：LRC（时间轴）、KSC（逐字）、TXT（纯文本）
- 逐字高亮：当前唱到的字高亮显示
- 自动滚动：跟随播放进度自动滚动

#### 2.3.2 技术实现

**LRC 格式解析**
```
[ti:歌曲名]
[ar:歌手]
[al:专辑]
[00:00.00]第一行歌词
[00:05.50]第二行歌词
[00:10.30]第三行歌词
```

**KSC 格式解析（逐字）**
```
karaoke := ('歌曲名')('歌手名')
line := [时间]歌词内容
word := <时间,时长>字
```

**歌词解析器**
```rust
pub enum LyricsFormat {
    Lrc,
    Ksc,
    Txt,
}

pub struct LyricsLine {
    pub time_ms: u64,
    pub duration_ms: Option<u64>,
    pub text: String,
    pub words: Option<Vec<LyricsWord>>,
}

pub struct LyricsWord {
    pub time_ms: u64,
    pub duration_ms: u64,
    pub text: String,
}

pub fn parse_lyrics(content: &str, format: LyricsFormat) -> Vec<LyricsLine> {
    match format {
        LyricsFormat::Lrc => parse_lrc(content),
        LyricsFormat::Ksc => parse_ksc(content),
        LyricsFormat::Txt => parse_txt(content),
    }
}
```

#### 2.3.3 前端渲染
```typescript
// components/Player/LyricsDisplay.tsx
function LyricsDisplay({ lyrics, currentTime }: Props) {
  // 找到当前行
  const currentLineIndex = lyrics.findIndex((line, i) => {
    const nextLine = lyrics[i + 1]
    return currentTime >= line.time &&
           (!nextLine || currentTime < nextLine.time)
  })

  return (
    <div className="lyrics-container">
      {lyrics.map((line, i) => (
        <div key={i} className={cn(
          "lyrics-line",
          i === currentLineIndex && "active",
          i < currentLineIndex && "passed"
        )}>
          {line.words ? (
            // 逐字渲染
            line.words.map((word, j) => (
              <span key={j} className={cn(
                "word",
                currentTime >= word.time && "highlighted"
              )}>
                {word.text}
              </span>
            ))
          ) : (
            // 整行渲染
            line.text
          )}
        </div>
      ))}
    </div>
  )
}
```

---

### 2.4 过场音乐模块 (Interlude Music)

#### 2.4.1 功能需求
- 自动播放：歌曲结束或暂停时自动播放
- 随机选择：从过场音乐库中随机选择
- Ducking：检测到人声时自动降低音量
- 音量控制：独立音量控制

#### 2.4.2 技术实现

**状态机设计**
```rust
pub enum InterludeState {
    Idle,           // 空闲，等待触发
    Playing,        // 正在播放
    Ducking,        // 正在降低音量
    Paused,         // 暂停（歌曲播放时）
}

impl InterludeManager {
    pub fn on_playback_event(&mut self, event: PlaybackEvent) {
        match event {
            PlaybackEvent::Started => {
                self.fade_out_and_pause();
            }
            PlaybackEvent::Paused => {
                self.fade_in_and_play();
            }
            PlaybackEvent::Stopped => {
                self.fade_in_and_play();
            }
            PlaybackEvent::Ended => {
                self.fade_in_and_play();
            }
        }
    }
}
```

**VAD Ducking 实现**
```rust
pub struct DuckingController {
    vad: WebRtcVad,
    threshold: f32,
    duck_ratio: f32,
    attack_ms: u32,
    release_ms: u32,
    current_volume: f32,
}

impl DuckingController {
    pub fn process(&mut self, input_samples: &[i16]) -> f32 {
        // 检测是否有人声
        let is_voice = self.vad.is_voice(input_samples);

        // 计算目标音量
        let target_volume = if is_voice {
            self.duck_ratio  // 降低到 20%
        } else {
            1.0              // 恢复到 100%
        };

        // 平滑过渡
        self.current_volume = self.smooth_transition(
            self.current_volume,
            target_volume,
        );

        self.current_volume
    }
}
```

---

### 2.5 气氛组模块 (Atmosphere Sounds)

#### 2.5.1 功能需求
- MIDI 触发：通过 MIDI 键盘触发音效
- 播放模式：一次性播放 / 循环播放
- 音量控制：独立音量控制
- 优先级：播放气氛组时暂停过场音乐

#### 2.5.2 技术实现

**MIDI 处理**
```rust
pub struct MidiHandler {
    input: MidiInputConnection,
    sound_map: HashMap<(u8, u8), AtmosphereSound>,  // (channel, note) -> sound
}

impl MidiHandler {
    pub fn on_midi_event(&mut self, message: &[u8]) {
        let status = message[0];
        let channel = status & 0x0F;
        let note = message[1];
        let velocity = message[2];

        match status & 0xF0 {
            0x90 => {  // Note On
                if velocity > 0 {
                    self.trigger_sound(channel, note, velocity);
                } else {
                    self.release_sound(channel, note);
                }
            }
            0x80 => {  // Note Off
                self.release_sound(channel, note);
            }
            _ => {}
        }
    }

    fn trigger_sound(&mut self, channel: u8, note: u8, velocity: u8) {
        if let Some(sound) = self.sound_map.get(&(channel, note)) {
            let volume = velocity as f32 / 127.0 * sound.volume;
            self.audio_player.play(&sound.file_path, volume);

            // 通知过场音乐暂停
            self.interlude_manager.pause();
        }
    }
}
```

---

### 2.6 人声效果器链模块 (Voice Effect Chain)

#### 2.6.1 功能需求
- 多槽位：最多8个效果器槽位，支持拖拽排序
- 效果器类型：混响、合唱、EQ、压缩器、延迟、去齿音、激励器、噪声门
- 实时调节：参数实时生效
- 预设管理：保存/加载预设
- 音频路由：输入设备选择，双输出（监听+直播）

#### 2.6.2 技术实现

**效果器链架构**
```rust
pub struct EffectChain {
    slots: Vec<Option<EffectSlot>>,
    input_stream: Option<AudioInputStream>,
    monitor_stream: Option<AudioOutputStream>,
    stream_output: Option<AudioOutputStream>,
}

pub struct EffectSlot {
    effect_type: EffectType,
    enabled: bool,
    processor: Box<dyn AudioProcessor>,
}

pub trait AudioProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]);
    fn set_parameter(&mut self, name: &str, value: f32);
    fn reset(&mut self);
}
```

**混响实现 (Freeverb)**
```rust
pub struct ReverbProcessor {
    // Freeverb 算法参数
    room_size: f32,
    damping: f32,
    wet: f32,
    dry: f32,

    // 延迟线
    comb_filters: Vec<CombFilter>,
    allpass_filters: Vec<AllPassFilter>,
}

impl AudioProcessor for ReverbProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (i, sample) in input.iter().enumerate() {
            // 并行梳状滤波器
            let mut comb_out = 0.0;
            for comb in &mut self.comb_filters {
                comb_out += comb.process(*sample);
            }

            // 串行全通滤波器
            let mut allpass_out = comb_out;
            for allpass in &mut self.allpass_filters {
                allpass_out = allpass.process(allpass_out);
            }

            // 混合输出
            output[i] = *sample * self.dry + allpass_out * self.wet;
        }
    }
}
```

**EQ 实现 (参数均衡)**
```rust
pub struct EQProcessor {
    bands: [EQBand; 4],  // Low, Low-Mid, High-Mid, High
}

struct EQBand {
    filter: BiquadFilter,
    frequency: f32,
    gain: f32,
    q: f32,
}

impl EQBand {
    fn calculate_coefficients(&mut self, sample_rate: f32) {
        let a = 10.0_f32.powf(self.gain / 40.0);
        let w0 = 2.0 * PI * self.frequency / sample_rate;
        let alpha = w0.sin() / (2.0 * self.q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * w0.cos();
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha / a;

        self.filter.set_coefficients(b0/a0, b1/a0, b2/a0, a1/a0, a2/a0);
    }
}
```

**压缩器实现**
```rust
pub struct CompressorProcessor {
    threshold: f32,     // dB
    ratio: f32,
    attack: f32,        // ms
    release: f32,       // ms
    makeup_gain: f32,   // dB

    envelope: f32,
    sample_rate: f32,
}

impl AudioProcessor for CompressorProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let attack_coef = (-1.0 / (self.attack * self.sample_rate / 1000.0)).exp();
        let release_coef = (-1.0 / (self.release * self.sample_rate / 1000.0)).exp();

        for (i, sample) in input.iter().enumerate() {
            // 计算包络
            let abs_sample = sample.abs();
            if abs_sample > self.envelope {
                self.envelope = attack_coef * self.envelope + (1.0 - attack_coef) * abs_sample;
            } else {
                self.envelope = release_coef * self.envelope + (1.0 - release_coef) * abs_sample;
            }

            // 计算增益衰减
            let db_level = 20.0 * self.envelope.log10();
            let gain_reduction = if db_level > self.threshold {
                (self.threshold - db_level) * (1.0 - 1.0 / self.ratio)
            } else {
                0.0
            };

            let gain = 10.0_f32.powf((gain_reduction + self.makeup_gain) / 20.0);
            output[i] = *sample * gain;
        }
    }
}
```

**音频路由**
```rust
pub struct AudioRouter {
    input_device: String,
    monitor_device: String,
    stream_device: String,

    input_stream: AudioInputStream,
    monitor_stream: AudioOutputStream,
    stream_stream: AudioOutputStream,
}

impl AudioRouter {
    pub fn start_processing(&mut self, effect_chain: &mut EffectChain) {
        // 从输入设备读取
        self.input_stream.read_callback = |input_samples| {
            // 经过效果器链处理
            let processed = effect_chain.process(input_samples);

            // 输出到监听设备
            self.monitor_stream.write(&processed);

            // 输出到直播设备
            self.stream_stream.write(&processed);
        };
    }
}
```

---

### 2.7 音频路由模块 (Audio Router)

#### 2.7.1 功能需求
- 设备枚举：列出所有输入/输出音频设备
- 设备选择：选择不同的输入/输出设备
- 音量控制：各通道独立音量控制
- 延迟优化：尽可能低的处理延迟

#### 2.7.2 技术实现

**设备枚举 (cpal)**
```rust
use cpal::traits::{DeviceTrait, HostTrait};

pub fn list_audio_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    // 输入设备
    if let Ok(input_devices) = host.input_devices() {
        for device in input_devices {
            if let Ok(name) = device.name() {
                devices.push(AudioDevice {
                    id: name.clone(),
                    name,
                    device_type: DeviceType::Input,
                    is_default: host.default_input_device()
                        .map(|d| d.name() == Ok(name))
                        .unwrap_or(false),
                });
            }
        }
    }

    // 输出设备
    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Ok(name) = device.name() {
                devices.push(AudioDevice {
                    id: name.clone(),
                    name,
                    device_type: DeviceType::Output,
                    is_default: host.default_output_device()
                        .map(|d| d.name() == Ok(name))
                        .unwrap_or(false),
                });
            }
        }
    }

    devices
}
```

---

## 三、数据存储方案

### 3.1 数据库设计原则
- 使用 SQLite 作为本地数据库
- 支持全文搜索 (FTS5)
- 使用外键约束保证数据一致性
- 适当的索引优化查询性能

### 3.2 文件存储结构
```
~/.rainkaraoke/
├── data/
│   └── rainkaraoke.db          # 数据库文件
├── cache/
│   ├── waveforms/              # 波形缓存
│   └── thumbnails/             # 缩略图缓存
└── logs/
    └── app.log                 # 日志文件
```

### 3.3 配置文件
```json
// ~/.rainkaraoke/config.json
{
  "version": "0.1.0",
  "library": {
    "watchFolders": [],
    "autoScan": true,
    "scanInterval": 300
  },
  "playback": {
    "autoPlayInterlude": true,
    "crossfadeDuration": 500
  },
  "effects": {
    "inputDevice": "default",
    "monitorDevice": "default",
    "streamDevice": "VB-Cable",
    "bufferSize": 256
  }
}
```

---

## 四、错误处理与日志

### 4.1 错误处理策略
```rust
// 统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Audio error: {0}")]
    Audio(#[from] cpal::BuildStreamError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

// Result 别名
pub type Result<T> = std::result::Result<T, AppError>;
```

### 4.2 日志系统
```rust
use tracing::{info, warn, error, debug};

pub fn init_logging() {
    let log_dir = dirs::data_local_dir()
        .unwrap()
        .join("rainkaraoke")
        .join("logs");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_rolling_file(log_dir, Rotation::DAILY)
        .init();
}
```

---

## 五、性能优化策略

### 5.1 音频处理优化
- 使用环形缓冲区减少内存分配
- SIMD 加速音频处理（使用 `packed_simd`）
- 低延迟音频流配置（buffer size = 256）

### 5.2 UI 渲染优化
- 虚拟列表渲染大量歌曲
- 使用 React.memo 减少不必要的重渲染
- Web Worker 处理搜索过滤

### 5.3 数据库优化
- 批量插入使用事务
- 预编译常用查询语句
- 连接池管理

---

## 六、测试策略

### 6.1 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lrc_parser() {
        let content = "[00:00.00]第一行\n[00:05.50]第二行";
        let lyrics = parse_lrc(content);
        assert_eq!(lyrics.len(), 2);
        assert_eq!(lyrics[0].text, "第一行");
    }

    #[test]
    fn test_reverb_processor() {
        let mut reverb = ReverbProcessor::new(44100);
        let input = vec![0.5; 1024];
        let mut output = vec![0.0; 1024];
        reverb.process(&input, &mut output);
        assert!(output.iter().any(|&x| x != 0.0));
    }
}
```

### 6.2 集成测试
- 测试完整的播放流程
- 测试效果器链处理
- 测试 MIDI 触发

### 6.3 性能测试
- 音频处理延迟测试
- 大量歌曲加载测试
- 内存使用监控
