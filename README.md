# RainKaraoke - 直播K歌助手

## 项目简介

RainKaraoke 是一款专为直播设计的 K 歌软件，支持视频、音频、歌词的统一管理，具备过场音乐自动播放、气氛组音效控制、MIDI 触发、人声效果器链等功能。

## 核心功能

- **媒体库管理**：视频/音频/歌词存储、标签分类、搜索
- **点歌播放**：队列管理、原唱/伴唱切换、逐字歌词高亮
- **过场音乐**：自动播放、语音检测自动降低音量
- **气氛组**：MIDI 键盘触发音效、音频管理
- **人声效果器链**：混响、合唱、EQ、压缩器等8种效果器
- **音频路由**：多输入源选择、多输出设备支持

## 技术栈

- **前端**：React 18 + TypeScript + Zustand + Tailwind CSS
- **后端**：Tauri 2.0 + Rust
- **数据库**：SQLite
- **音视频**：FFmpeg + Rodio
- **MIDI**：midir
- **VAD**：webrtc-vad

## 文档

| 文档 | 说明 |
|------|------|
| [设计方案](./docs/DESIGN.md) | 系统架构与模块设计 |
| [开发方案](./docs/DEVELOPMENT_PLAN.md) | 详细技术实现方案 |
| [开发计划](./docs/SCHEDULE.md) | 排期、进度与里程碑 |
| [进度追踪](./docs/PROGRESS.md) | 每日进度与问题记录 |
| [数据库设计](./docs/DATABASE.md) | 表结构与关系 |
| [API 接口](./docs/API.md) | Tauri Commands 接口 |

## 开发阶段

| 阶段 | 内容 | 状态 | 进度 |
|------|------|------|------|
| Phase 1 | 项目搭建、数据库、媒体库基础功能 | 🔄 进行中 | 80% |
| Phase 2 | 播放引擎、原唱/伴唱切换、歌词显示 | ⏳ 待开始 | 0% |
| Phase 3 | 过场音乐、VAD Ducking | ⏳ 待开始 | 0% |
| Phase 4 | 气氛组、MIDI集成 | ⏳ 待开始 | 0% |
| Phase 5 | 效果器链、音频路由 | ⏳ 待开始 | 0% |
| Phase 6 | UI完善、测试、优化 | ⏳ 待开始 | 0% |

## 快速开始

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## 项目结构

```
RainKaraoke/
├── docs/                    # 设计文档
│   ├── DESIGN.md           # 详细设计方案
│   ├── DEVELOPMENT_PLAN.md # 开发方案
│   ├── SCHEDULE.md         # 开发计划
│   ├── PROGRESS.md         # 进度追踪
│   ├── DATABASE.md         # 数据库设计
│   └── API.md              # API 接口
├── src-tauri/               # Tauri 后端 (Rust)
│   └── src/
│       ├── commands/       # Tauri 命令
│       ├── db/             # 数据库
│       ├── models/         # 数据模型
│       └── modules/        # 核心模块
├── src/                     # React 前端
│   ├── components/         # UI 组件
│   ├── types/              # TypeScript 类型
│   └── App.tsx
└── package.json
```

## 许可证

MIT
