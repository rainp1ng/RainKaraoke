# RainKaraoke - 直播K歌助手

一款专为直播场景设计的 KTV 应用程序，支持视频/音频播放、歌词显示、实时音频效果处理、MIDI 控制等功能。

## 功能特性

### 媒体库管理
- 自动扫描并导入歌曲文件夹
- 支持视频格式：MP4, MKV, AVI, MOV
- 支持音频格式：MP3, FLAC, WAV, OGG, M4A, AAC
- 智能文件匹配：自动关联原唱、伴奏、歌词文件
- 歌曲元数据编辑（标题、歌手、专辑）
- 按歌手、流派筛选歌曲

### 播放控制
- 原唱/伴奏一键切换
- 变调功能（±12 半音）
- 变速功能（0.5x - 2.0x）
- 音量控制
- 歌词同步显示（支持 LRC、KSC 格式）
- 视频播放支持

### 播放队列
- 拖拽排序队列
- 顶歌功能（移到最前面/下一首）
- 一键添加到队列

### 实时音频效果器
- **混响**：房间大小、宽度、阻尼调节
- **延时**：延时时间、反馈、混合比例
- **合唱**：速率、深度、混合比例
- **均衡器**：低频、中频、高频调节
- **去齿音**：频率、阈值调节
- **激励器**：频率、混合比例
- **电平表**：实时音量监控
- 效果器预设保存/加载
- 一键旁路所有效果

### 实时音频路由
- 多输入设备支持（麦克风、乐器）
- 多输出设备支持（监听、直播）
- 独立的音轨录制（人声、乐器分离）
- 输入通道选择

### 过场音乐
- 自动播放过场音乐（歌曲间隙）
- 智能闪避（Ducking）：检测人声自动降低音量
- 可配置闪避参数：阈值、比例、恢复时间

### 气氛组音效
- 一键播放预设音效（掌声、欢呼等）
- 支持 MIDI 设备触发
- 自定义音效绑定

### MIDI 控制
- 自动检测并连接 MIDI 设备
- 音效 MIDI 映射
- 支持脚踏板等控制器

### 界面特性
- 可拖拽调节各模块大小
- 布局配置自动保存
- 深色主题设计

## 技术栈

- **前端**：React 18 + TypeScript + Tailwind CSS + Zustand
- **后端**：Rust + Tauri 2
- **音频处理**：Rodio + CPAL + Rubato
- **数据库**：SQLite (Rusqlite)
- **MIDI**：Midir

## 开发环境要求

### 通用依赖
- Node.js >= 18
- Rust >= 1.70
- npm / pnpm / yarn

### macOS
- Xcode Command Line Tools: `xcode-select --install`

### Windows
- Microsoft Visual Studio C++ Build Tools
- WebView2（Windows 10/11 已预装）

## 快速开始

### 1. 克隆项目
```bash
git clone https://github.com/rainp1ng/RainKaraoke.git
cd RainKaraoke
```

### 2. 安装依赖
```bash
# 安装 Node.js 依赖
npm install

# Rust 依赖会在首次构建时自动安装
```

### 3. 开发模式运行
```bash
npm run tauri:dev
```

## 编译打包

### macOS

```bash
# 编译 macOS 版本（生成 .app 和 .dmg）
npm run tauri:build

# 输出位置
# src-tauri/target/release/bundle/macos/RainKaraoke.app
# src-tauri/target/release/bundle/dmg/RainKaraoke_0.1.0_aarch64.dmg  (Apple Silicon)
# src-tauri/target/release/bundle/dmg/RainKaraoke_0.1.0_x64.dmg      (Intel)
```

### Windows

```bash
# 编译 Windows 版本（生成 .exe 和 .msi）
npm run tauri:build

# 输出位置
# src-tauri/target/release/bundle/msi/RainKaraoke_0.1.0_x64.msi
# src-tauri/target/release/bundle/nsis/RainKaraoke_0.1.0_x64-setup.exe
```

## 项目结构

```
RainKaraoke/
├── src/                      # React 前端代码
│   ├── components/           # UI 组件
│   │   ├── EffectChain/      # 效果器链
│   │   ├── Interlude/        # 过场音乐
│   │   ├── Library/          # 媒体库
│   │   ├── Player/           # 播放器
│   │   ├── Queue/            # 播放队列
│   │   └── ...
│   ├── stores/               # Zustand 状态管理
│   ├── lib/                  # API 封装
│   └── types/                # TypeScript 类型定义
├── src-tauri/                # Rust 后端代码
│   ├── src/
│   │   ├── commands/         # Tauri 命令
│   │   ├── db/               # 数据库操作
│   │   ├── models/           # 数据模型
│   │   ├── modules/          # 核心模块
│   │   │   ├── effects/      # 音频效果器
│   │   │   ├── media_engine/ # 媒体播放引擎
│   │   │   ├── audio_router/ # 实时音频路由
│   │   │   └── ...
│   │   └── utils/            # 工具函数
│   ├── Cargo.toml            # Rust 依赖配置
│   └── tauri.conf.json       # Tauri 配置
└── package.json              # Node.js 依赖配置
```

## 使用指南

### 导入歌曲
1. 点击顶部工具栏的「导入歌曲」按钮
2. 选择包含歌曲文件的文件夹
3. 程序会自动扫描并识别：
   - 视频文件（MP4/MKV/AVI）
   - 音频文件（MP3/FLAC/WAV）
   - 歌词文件（LRC/KSC）
4. 相同文件名会自动关联为同一首歌曲

### 文件命名建议
为了更好的自动识别效果，建议使用以下命名格式：
- `歌手 - 歌曲名.mp4`
- `歌手 - 歌曲名_原唱.mp3`
- `歌手 - 歌曲名_伴奏.mp3`
- `歌手 - 歌曲名.lrc`

### 配置效果器
1. 进入「效果器链」页面
2. 选择输入设备（麦克风）
3. 选择输出设备（监听耳机、直播推流）
4. 添加和配置效果器
5. 保存为预设方便下次使用

### 配置 MIDI 控制器
1. 连接 MIDI 设备（如脚踏板）
2. 在设置页面查看 MIDI 设备状态
3. 在效果器或气氛组中配置 MIDI 映射

## 配置文件

应用配置存储在以下位置：

- **macOS**: `~/Library/Application Support/com.rainp1ng.karaoke/`
- **Windows**: `%APPDATA%\com.rainp1ng.karaoke\`

包含：
- 数据库文件 (`rainkaraoke.db`)
- 音频配置
- 效果器预设

## 常见问题

### macOS 提示"无法打开，因为它来自身份不明的开发者"
```bash
# 移除隔离属性
xattr -cr RainKaraoke.app
```

### Windows 播放无声音
确保已安装最新版 WebView2 运行时。

### 某些音频格式无法播放
支持的格式有限，建议转换为 MP3 或 FLAC 格式。

## 开发计划

- [ ] 更多效果器（压缩器、限制器、噪声门）
- [ ] 歌词编辑器
- [ ] 歌曲评分系统
- [ ] 云端歌库同步
- [ ] 移动端遥控应用

## 文档

| 文档 | 说明 |
|------|------|
| [设计方案](./docs/DESIGN.md) | 系统架构与模块设计 |
| [开发方案](./docs/DEVELOPMENT_PLAN.md) | 详细技术实现方案 |
| [数据库设计](./docs/DATABASE.md) | 表结构与关系 |
| [API 接口](./docs/API.md) | Tauri Commands 接口 |

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

## 作者

rainp1ng

---

**致谢**：本项目使用了以下开源项目：
- [Tauri](https://tauri.app/)
- [Rodio](https://github.com/RustAudio/rodio)
- [React](https://react.dev/)
- [Tailwind CSS](https://tailwindcss.com/)
