# RainKaraoke API 接口设计

## 概述

使用 Tauri 的 IPC 机制，前端通过 `invoke` 调用 Rust 后端命令。

## API 列表

### 一、媒体库管理 API

#### 1.1 获取歌曲列表

```typescript
// 命令名称
'get_songs'

// 参数
interface GetSongsParams {
  page?: number;        // 页码，默认1
  pageSize?: number;    // 每页数量，默认50
  search?: string;      // 搜索关键词
  artist?: string;      // 歌手筛选
  genre?: string;       // 风格筛选
  language?: string;    // 语言筛选
  tags?: string[];      // 标签筛选
  sortBy?: 'title' | 'artist' | 'play_count' | 'last_played_at' | 'created_at';
  sortOrder?: 'asc' | 'desc';
}

// 返回
interface GetSongsResult {
  songs: Song[];
  total: number;
  page: number;
  pageSize: number;
}
```

#### 1.2 获取单首歌曲详情

```typescript
// 命令名称
'get_song_by_id'

// 参数
interface GetSongByIdParams {
  id: number;
}

// 返回
Song | null
```

#### 1.3 添加歌曲

```typescript
// 命令名称
'add_song'

// 参数
interface AddSongParams {
  title: string;
  artist?: string;
  album?: string;
  videoPath?: string;
  vocalAudioPath?: string;
  instrumentalAudioPath?: string;
  lyricsPath?: string;
  genre?: string;
  language?: string;
  tags?: string[];
}

// 返回
number // 新歌曲ID
```

#### 1.4 批量导入歌曲

```typescript
// 命令名称
'import_songs'

// 参数
interface ImportSongsParams {
  directory: string;      // 扫描目录
  recursive?: boolean;    // 是否递归扫描子目录
}

// 返回
interface ImportResult {
  success: number;        // 成功导入数量
  skipped: number;        // 跳过数量（已存在）
  failed: number;         // 失败数量
  errors: string[];       // 错误信息列表
}
```

#### 1.5 更新歌曲信息

```typescript
// 命令名称
'update_song'

// 参数
interface UpdateSongParams {
  id: number;
  title?: string;
  artist?: string;
  album?: string;
  videoPath?: string;
  vocalAudioPath?: string;
  instrumentalAudioPath?: string;
  lyricsPath?: string;
  genre?: string;
  language?: string;
  tags?: string[];
}

// 返回
boolean // 是否成功
```

#### 1.6 删除歌曲

```typescript
// 命令名称
'delete_song'

// 参数
interface DeleteSongParams {
  id: number;
  deleteFiles?: boolean;  // 是否同时删除文件
}

// 返回
boolean
```

#### 1.7 获取所有标签

```typescript
// 命令名称
'get_tags'

// 参数
interface GetTagsParams {
  category?: string;  // 按分类筛选
}

// 返回
Tag[]
```

#### 1.8 添加标签

```typescript
// 命令名称
'add_tag'

// 参数
interface AddTagParams {
  name: string;
  category?: string;
  color?: string;
}

// 返回
number // 新标签ID
```

---

### 二、播放控制 API

#### 2.1 播放歌曲

```typescript
// 命令名称
'play_song'

// 参数
interface PlaySongParams {
  songId: number;
  startTime?: number;  // 起始时间(秒)
}

// 返回
boolean
```

#### 2.2 暂停播放

```typescript
// 命令名称
'pause_song'

// 返回
boolean
```

#### 2.3 继续播放

```typescript
// 命令名称
'resume_song'

// 返回
boolean
```

#### 2.4 停止播放

```typescript
// 命令名称
'stop_song'

// 返回
boolean
```

#### 2.5 跳转到指定时间

```typescript
// 命令名称
'seek_to'

// 参数
interface SeekToParams {
  time: number;  // 目标时间(秒)
}

// 返回
boolean
```

#### 2.6 切换原唱/伴唱

```typescript
// 命令名称
'toggle_vocal'

// 参数
interface ToggleVocalParams {
  enabled: boolean;  // true=原唱, false=伴唱
}

// 返回
boolean
```

#### 2.7 设置音调

```typescript
// 命令名称
'set_pitch'

// 参数
interface SetPitchParams {
  semitones: number;  // 半音数，范围 -12 到 +12
}

// 返回
boolean
```

#### 2.8 设置播放速度

```typescript
// 命令名称
'set_speed'

// 参数
interface SetSpeedParams {
  speed: number;  // 速度倍率，范围 0.5 到 2.0
}

// 返回
boolean
```

#### 2.9 获取播放状态

```typescript
// 命令名称
'get_playback_state'

// 返回
interface PlaybackState {
  status: 'idle' | 'playing' | 'paused';
  currentSongId: number | null;
  currentTime: number;      // 当前播放时间(秒)
  duration: number;         // 总时长(秒)
  isVocal: boolean;         // 是否原唱
  pitch: number;            // 当前音调
  speed: number;            // 当前速度
}
```

---

### 三、播放队列 API

#### 3.1 获取播放队列

```typescript
// 命令名称
'get_queue'

// 返回
interface QueueItem {
  id: number;
  songId: number;
  position: number;
  song?: Song;  // 包含歌曲详情
}[]
```

#### 3.2 添加到队列

```typescript
// 命令名称
'add_to_queue'

// 参数
interface AddToQueueParams {
  songId: number;
  position?: number;  // 指定位置，不传则添加到末尾
}

// 返回
number // 队列项ID
```

#### 3.3 批量添加到队列

```typescript
// 命令名称
'add_many_to_queue'

// 参数
interface AddManyToQueueParams {
  songIds: number[];
}

// 返回
number[] // 队列项ID列表
```

#### 3.4 从队列移除

```typescript
// 命令名称
'remove_from_queue'

// 参数
interface RemoveFromQueueParams {
  queueId: number;  // 队列项ID
}

// 返回
boolean
```

#### 3.5 移动队列项

```typescript
// 命令名称
'move_queue_item'

// 参数
interface MoveQueueItemParams {
  queueId: number;
  newPosition: number;
}

// 返回
boolean
```

#### 3.6 清空队列

```typescript
// 命令名称
'clear_queue'

// 返回
boolean
```

#### 3.7 播放下一首

```typescript
// 命令名称
'play_next'

// 返回
boolean
```

---

### 四、过场音乐 API

#### 4.1 获取过场音乐列表

```typescript
// 命令名称
'get_interlude_tracks'

// 返回
InterludeTrack[]
```

#### 4.2 添加过场音乐

```typescript
// 命令名称
'add_interlude_track'

// 参数
interface AddInterludeTrackParams {
  title?: string;
  filePath: string;
  volume?: number;
}

// 返回
number
```

#### 4.3 删除过场音乐

```typescript
// 命令名称
'delete_interlude_track'

// 参数
interface DeleteInterludeTrackParams {
  id: number;
}

// 返回
boolean
```

#### 4.4 设置过场音乐音量

```typescript
// 命令名称
'set_interlude_volume'

// 参数
interface SetInterludeVolumeParams {
  volume: number;  // 0-1
}

// 返回
boolean
```

#### 4.5 获取过场音乐播放状态

```typescript
// 命令名称
'get_interlude_state'

// 返回
interface InterludeState {
  isPlaying: boolean;
  currentTrackId: number | null;
  volume: number;
  duckingActive: boolean;
}
```

---

### 五、气氛组 API

#### 5.1 获取气氛组音频列表

```typescript
// 命令名称
'get_atmosphere_sounds'

// 返回
AtmosphereSound[]
```

#### 5.2 添加气氛组音频

```typescript
// 命令名称
'add_atmosphere_sound'

// 参数
interface AddAtmosphereSoundParams {
  name: string;
  filePath: string;
  volume?: number;
  midiNote?: number;
  midiChannel?: number;
  isOneShot?: boolean;
  color?: string;
}

// 返回
number
```

#### 5.3 更新气氛组音频

```typescript
// 命令名称
'update_atmosphere_sound'

// 参数
interface UpdateAtmosphereSoundParams {
  id: number;
  name?: string;
  volume?: number;
  midiNote?: number;
  midiChannel?: number;
  isOneShot?: boolean;
  color?: string;
  sortOrder?: number;
}

// 返回
boolean
```

#### 5.4 删除气氛组音频

```typescript
// 命令名称
'delete_atmosphere_sound'

// 参数
interface DeleteAtmosphereSoundParams {
  id: number;
}

// 返回
boolean
```

#### 5.5 播放气氛组音频

```typescript
// 命令名称
'play_atmosphere_sound'

// 参数
interface PlayAtmosphereSoundParams {
  id: number;
}

// 返回
boolean
```

#### 5.6 停止气氛组音频

```typescript
// 命令名称
'stop_atmosphere_sound'

// 参数
interface StopAtmosphereSoundParams {
  id?: number;  // 不传则停止所有
}

// 返回
boolean
```

---

### 六、MIDI API

#### 6.1 获取可用 MIDI 设备列表

```typescript
// 命令名称
'get_midi_devices'

// 返回
interface MidiDevice {
  id: string;
  name: string;
  inputCount: number;
  outputCount: number;
}[]
```

#### 6.2 连接 MIDI 设备

```typescript
// 命令名称
'connect_midi_device'

// 参数
interface ConnectMidiDeviceParams {
  deviceId: string;
}

// 返回
boolean
```

#### 6.3 断开 MIDI 设备

```typescript
// 命令名称
'disconnect_midi_device'

// 返回
boolean
```

#### 6.4 获取 MIDI 连接状态

```typescript
// 命令名称
'get_midi_status'

// 返回
interface MidiStatus {
  connected: boolean;
  deviceId: string | null;
  deviceName: string | null;
}
```

---

### 七、音频设置 API

#### 7.1 获取音频设备列表

```typescript
// 命令名称
'get_audio_devices'

// 返回
interface AudioDevice {
  id: string;
  name: string;
  type: 'input' | 'output';
  isDefault: boolean;
  channels: number;
}[]
```

#### 7.2 获取音频配置

```typescript
// 命令名称
'get_audio_config'

// 返回
AudioConfig
```

#### 7.3 保存音频配置

```typescript
// 命令名称
'save_audio_config'

// 参数
interface SaveAudioConfigParams {
  defaultOutputDevice?: string;
  interludeOutputDevice?: string;
  atmosphereOutputDevice?: string;
  masterVolume?: number;
  interludeVolume?: number;
  atmosphereVolume?: number;
  duckingEnabled?: boolean;
  duckingThreshold?: number;
  duckingRatio?: number;
  duckingAttackMs?: number;
  duckingReleaseMs?: number;
  midiDeviceId?: string;
  midiEnabled?: boolean;
}

// 返回
boolean
```

---

### 八、歌词 API

#### 8.1 获取歌词内容

```typescript
// 命令名称
'get_lyrics'

// 参数
interface GetLyricsParams {
  songId: number;
}

// 返回
interface Lyrics {
  format: 'lrc' | 'ksc' | 'txt';
  lines: LyricsLine[];
}

interface LyricsLine {
  time: number;      // 开始时间(毫秒)
  duration?: number; // 持续时间(毫秒)，KSC格式有
  text: string;
  words?: LyricsWord[];  // KSC格式的逐字信息
}

interface LyricsWord {
  time: number;      // 开始时间(毫秒)
  duration: number;  // 持续时间(毫秒)
  text: string;
}
```

#### 8.2 解析歌词文件

```typescript
// 命令名称
'parse_lyrics_file'

// 参数
interface ParseLyricsFileParams {
  filePath: string;
}

// 返回
Lyrics
```

---

## 事件 (Events)

后端向前端推送的事件，通过 `listen` 监听。

### 播放事件

```typescript
// 播放状态变化
'song:status-changed'
{ status: 'playing' | 'paused' | 'stopped' | 'ended' }

// 播放时间更新 (每秒触发)
'song:time-updated'
{ currentTime: number, duration: number }

// 歌曲切换
'song:changed'
{ songId: number | null }
```

### 过场音乐事件

```typescript
// 过场音乐状态变化
'interlude:status-changed'
{ isPlaying: boolean, trackId: number | null }

// Ducking 状态变化
'interlude:ducking-changed'
{ active: boolean }
```

### 气氛组事件

```typescript
// 气氛组音频播放
'atmosphere:played'
{ soundId: number }

// 气氛组音频停止
'atmosphere:stopped'
{ soundId: number }
```

### MIDI 事件

```typescript
// MIDI 设备连接状态变化
'midi:connection-changed'
{ connected: boolean, deviceName: string | null }

// MIDI 音符触发
'midi:note-triggered'
{ note: number, channel: number, velocity: number }
```

### 队列事件

```typescript
// 队列更新
'queue:updated'
{ items: QueueItem[] }
```

---

### 九、人声效果器链 API

#### 9.1 获取效果器链配置

```typescript
// 命令名称
'get_effect_chain_config'

// 返回
interface EffectChainConfig {
  inputDeviceId: string | null;
  inputVolume: number;
  monitorDeviceId: string | null;
  streamDeviceId: string | null;
  monitorVolume: number;
  streamVolume: number;
  bypassAll: boolean;
}
```

#### 9.2 保存效果器链配置

```typescript
// 命令名称
'save_effect_chain_config'

// 参数
interface SaveEffectChainConfigParams {
  inputDeviceId?: string;
  inputVolume?: number;
  monitorDeviceId?: string;
  streamDeviceId?: string;
  monitorVolume?: number;
  streamVolume?: number;
  bypassAll?: boolean;
}

// 返回
boolean
```

#### 9.3 获取所有效果器槽位

```typescript
// 命令名称
'get_effect_slots'

// 返回
EffectSlot[]
```

#### 9.4 设置效果器槽位

```typescript
// 命令名称
'set_effect_slot'

// 参数
interface SetEffectSlotParams {
  slotIndex: number;         // 槽位索引 0-7
  effectType: EffectType;    // 效果器类型
  enabled?: boolean;         // 是否启用
  parameters?: object;       // 效果器参数
}

// 返回
boolean
```

#### 9.5 更新效果器参数

```typescript
// 命令名称
'update_effect_parameters'

// 参数
interface UpdateEffectParametersParams {
  slotIndex: number;
  parameters: object;        // 要更新的参数
}

// 返回
boolean
```

#### 9.6 启用/禁用效果器

```typescript
// 命令名称
'toggle_effect'

// 参数
interface ToggleEffectParams {
  slotIndex: number;
  enabled: boolean;
}

// 返回
boolean
```

#### 9.7 移动效果器顺序

```typescript
// 命令名称
'move_effect_slot'

// 参数
interface MoveEffectSlotParams {
  fromIndex: number;
  toIndex: number;
}

// 返回
boolean
```

#### 9.8 清空效果器槽位

```typescript
// 命令名称
'clear_effect_slot'

// 参数
interface ClearEffectSlotParams {
  slotIndex: number;
}

// 返回
boolean
```

#### 9.9 获取效果器预设列表

```typescript
// 命令名称
'get_effect_presets'

// 返回
EffectPreset[]
```

#### 9.10 保存效果器预设

```typescript
// 命令名称
'save_effect_preset'

// 参数
interface SaveEffectPresetParams {
  name: string;
  description?: string;
}

// 返回
number  // 预设ID
```

#### 9.11 加载效果器预设

```typescript
// 命令名称
'load_effect_preset'

// 参数
interface LoadEffectPresetParams {
  presetId: number;
}

// 返回
boolean
```

#### 9.12 删除效果器预设

```typescript
// 命令名称
'delete_effect_preset'

// 参数
interface DeleteEffectPresetParams {
  presetId: number;
}

// 返回
boolean
```

#### 9.13 旁通所有效果器

```typescript
// 命令名称
'bypass_all_effects'

// 参数
interface BypassAllEffectsParams {
  bypass: boolean;
}

// 返回
boolean
```

---

## TypeScript 类型定义

```typescript
// types/index.ts

export interface Song {
  id: number;
  title: string;
  artist: string | null;
  album: string | null;
  duration: number | null;

  videoPath: string | null;
  vocalAudioPath: string | null;
  instrumentalAudioPath: string | null;
  lyricsPath: string | null;

  lyricsFormat: string | null;
  hasVocal: boolean;
  hasInstrumental: boolean;

  genre: string | null;
  language: string | null;
  tags: string[];
  difficulty: number | null;

  playCount: number;
  lastPlayedAt: string | null;

  createdAt: string;
  updatedAt: string;
}

export interface Tag {
  id: number;
  name: string;
  category: string | null;
  color: string | null;
}

export interface InterludeTrack {
  id: number;
  title: string | null;
  filePath: string;
  duration: number | null;
  volume: number;
  isActive: boolean;
  playCount: number;
}

export interface AtmosphereSound {
  id: number;
  name: string;
  filePath: string;
  duration: number | null;
  volume: number;
  midiNote: number | null;
  midiChannel: number;
  isOneShot: boolean;
  color: string | null;
  sortOrder: number;
}

export interface AudioConfig {
  defaultOutputDevice: string | null;
  interludeOutputDevice: string | null;
  atmosphereOutputDevice: string | null;

  masterVolume: number;
  interludeVolume: number;
  atmosphereVolume: number;

  duckingEnabled: boolean;
  duckingThreshold: number;
  duckingRatio: number;
  duckingAttackMs: number;
  duckingReleaseMs: number;

  midiDeviceId: string | null;
  midiEnabled: boolean;
}

// ============ 效果器链类型 ============

export type EffectType =
  | 'reverb'
  | 'chorus'
  | 'eq'
  | 'compressor'
  | 'delay'
  | 'deesser'
  | 'exciter'
  | 'gate'

export interface EffectSlot {
  id: number;
  slotIndex: number;
  effectType: EffectType;
  isEnabled: boolean;
  parameters: Record<string, any>;
}

export interface EffectChainConfig {
  inputDeviceId: string | null;
  inputVolume: number;
  monitorDeviceId: string | null;
  streamDeviceId: string | null;
  monitorVolume: number;
  streamVolume: number;
  bypassAll: boolean;
}

export interface EffectPreset {
  id: number;
  name: string;
  description: string | null;
  isDefault: boolean;
  slots: EffectSlot[];
}

// 效果器参数类型
export interface ReverbParams {
  roomSize: number;      // 0-100
  damping: number;       // 0-100
  wetLevel: number;      // 0-100
  dryLevel: number;      // 0-100
  preDelay: number;      // 0-100 ms
}

export interface ChorusParams {
  rate: number;          // 0.1-10 Hz
  depth: number;         // 0-100 %
  mix: number;           // 0-100 %
  voices: number;        // 1-8
  spread: number;        // 0-100 %
}

export interface EQBand {
  gain: number;          // -12 to +12 dB
  frequency: number;     // Hz
  q: number;             // 0.1-10
}

export interface EQParams {
  low: EQBand;
  lowMid: EQBand;
  highMid: EQBand;
  high: EQBand;
}

export interface CompressorParams {
  threshold: number;     // -60 to 0 dB
  ratio: number;         // 1-20
  attack: number;        // 0.1-100 ms
  release: number;       // 10-1000 ms
  makeupGain: number;    // 0-24 dB
}

export interface DelayParams {
  time: number;          // 1-1000 ms
  feedback: number;      // 0-90 %
  mix: number;           // 0-100 %
  pingPong: boolean;
}

export interface DeEsserParams {
  frequency: number;     // 2000-12000 Hz
  threshold: number;     // -40 to 0 dB
  range: number;         // 0-24 dB
}

export interface ExciterParams {
  frequency: number;     // 2000-16000 Hz
  harmonics: number;     // 0-100 %
  mix: number;           // 0-100 %
}

export interface NoiseGateParams {
  threshold: number;     // -80 to -20 dB
  attack: number;        // 0.1-50 ms
  release: number;       // 10-500 ms
  range: number;         // 0-80 dB
}
```
