import { useState, useEffect, useCallback, useRef } from 'react'
import { Outlet, useLocation, NavLink } from 'react-router-dom'
import { open } from '@tauri-apps/plugin-dialog'
import { Music, ListMusic, Settings, Mic2, GripVertical } from 'lucide-react'
import Player from '../Player/Player'
import VideoPlayer from '../Player/VideoPlayer'
import LyricsDisplay from '../Player/LyricsDisplay'
import InterludePanel from '../Interlude/InterludePanel'
import AtmospherePanel from '../Atmosphere/AtmospherePanel'
import Queue from '../Queue/Queue'
import { libraryApi, midiApi, effectApi } from '@/lib/api'
import { useLibraryStore, usePlaybackStore } from '@/stores'
import { OutputLevelMeter, ChainLevelMeter } from '../EffectChain/LevelMeter'

// 可拖拽分隔条组件
function Resizer({
  onResize,
  direction = 'vertical',
  minSize = 200,
  maxSize = 800,
}: {
  onResize: (delta: number) => void
  direction?: 'vertical' | 'horizontal'
  minSize?: number
  maxSize?: number
}) {
  const isDragging = useRef(false)
  const lastPos = useRef(0)

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    isDragging.current = true
    lastPos.current = direction === 'vertical' ? e.clientX : e.clientY
    document.body.style.cursor = direction === 'vertical' ? 'col-resize' : 'row-resize'
    document.body.style.userSelect = 'none'
  }, [direction])

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return
      const currentPos = direction === 'vertical' ? e.clientX : e.clientY
      const delta = lastPos.current - currentPos
      lastPos.current = currentPos
      onResize(delta)
    }

    const handleMouseUp = () => {
      isDragging.current = false
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)

    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [onResize, direction])

  return (
    <div
      className={`group flex items-center justify-center bg-dark-800 hover:bg-primary-600 transition-colors ${
        direction === 'vertical' ? 'w-1.5 cursor-col-resize' : 'h-1.5 cursor-row-resize'
      }`}
      onMouseDown={handleMouseDown}
    >
      <GripVertical
        className={`text-dark-500 group-hover:text-white transition-colors ${
          direction === 'vertical' ? 'w-3 h-3' : 'w-3 h-3 rotate-90'
        }`}
      />
    </div>
  )
}

// 从 localStorage 加载布局配置
function loadLayoutConfig() {
  try {
    const saved = localStorage.getItem('layoutConfig')
    if (saved) {
      return JSON.parse(saved)
    }
  } catch {}
  return {
    leftPanelWidth: 480,
    rightPanelWidth: 320,
    bottomPanelHeight: 200,
  }
}

// 保存布局配置到 localStorage
function saveLayoutConfig(config: { leftPanelWidth: number; rightPanelWidth: number; bottomPanelHeight: number }) {
  try {
    localStorage.setItem('layoutConfig', JSON.stringify(config))
  } catch {}
}

function MainLayout() {
  const location = useLocation()
  const isEffectsPage = location.pathname === '/effects'
  const [midiConnected, setMidiConnected] = useState(false)
  const [midiDeviceName, setMidiDeviceName] = useState<string | null>(null)
  const { loadSongs, loadArtists, loadGenres } = useLibraryStore()
  const { currentSong, currentTime } = usePlaybackStore()

  // 可调节的布局尺寸
  const [layoutConfig, setLayoutConfig] = useState(loadLayoutConfig)

  // 导航项
  const navItems = [
    { to: '/', icon: Music, label: '媒体库' },
    { to: '/queue', icon: ListMusic, label: '播放队列' },
    { to: '/effects', icon: Mic2, label: '效果器链' },
    { to: '/settings', icon: Settings, label: '设置' },
  ]

  // 效果器链中的 levelmeter 槽位索引
  const [levelMeterSlots, setLevelMeterSlots] = useState<number[]>([])

  // 调整左侧面板宽度
  const handleLeftPanelResize = useCallback((delta: number) => {
    setLayoutConfig(prev => {
      const newWidth = Math.max(300, Math.min(800, prev.leftPanelWidth - delta))
      const newConfig = { ...prev, leftPanelWidth: newWidth }
      saveLayoutConfig(newConfig)
      return newConfig
    })
  }, [])

  // 调整右侧面板宽度
  const handleRightPanelResize = useCallback((delta: number) => {
    setLayoutConfig(prev => {
      const newWidth = Math.max(250, Math.min(600, prev.rightPanelWidth + delta))
      const newConfig = { ...prev, rightPanelWidth: newWidth }
      saveLayoutConfig(newConfig)
      return newConfig
    })
  }, [])

  // 调整底部面板高度
  const handleBottomPanelResize = useCallback((delta: number) => {
    setLayoutConfig(prev => {
      const newHeight = Math.max(120, Math.min(400, prev.bottomPanelHeight + delta))
      const newConfig = { ...prev, bottomPanelHeight: newHeight }
      saveLayoutConfig(newConfig)
      return newConfig
    })
  }, [])

  // 加载效果器链中的 levelmeter
  useEffect(() => {
    const loadLevelMeters = async () => {
      try {
        const slots = await effectApi.getEffectSlots()
        const meterSlots = slots
          .filter(s => s.effectType === 'levelmeter' && s.isEnabled)
          .map(s => s.slotIndex)
        setLevelMeterSlots(meterSlots)
      } catch {
        // ignore
      }
    }

    loadLevelMeters()
    // 定期检查更新
    const interval = setInterval(loadLevelMeters, 2000)
    return () => clearInterval(interval)
  }, [])

  useEffect(() => {
    // 尝试自动连接 MIDI 设备
    const autoConnectMidi = async () => {
      try {
        const connected = await midiApi.autoConnectMidi()
        console.log('[MainLayout] MIDI auto-connect result:', connected)
      } catch (e) {
        console.error('[MainLayout] MIDI auto-connect failed:', e)
      }
    }

    // 检查 MIDI 状态
    const checkMidiStatus = async () => {
      try {
        const status = await midiApi.getMidiStatus()
        setMidiConnected(status.connected)
        setMidiDeviceName(status.deviceName)
      } catch (e) {
        console.error('Failed to get MIDI status:', e)
      }
    }

    // 先自动连接，然后检查状态
    autoConnectMidi().then(() => checkMidiStatus())

    const interval = setInterval(checkMidiStatus, 5000)
    return () => clearInterval(interval)
  }, [])

  const handleImportFolder = async () => {
    try {
      // 支持选择多个文件（包括歌曲和歌词）
      const selected = await open({
        multiple: true,
        title: '选择歌曲文件、歌词文件或文件夹',
        filters: [
          {
            name: '媒体文件',
            extensions: ['mp4', 'mkv', 'avi', 'mov', 'mp3', 'flac', 'wav', 'ogg', 'm4a', 'lrc', 'ksc', 'txt']
          }
        ]
      })

      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected]
        let totalSuccess = 0
        let totalSkipped = 0
        let totalFailed = 0

        for (const path of paths) {
          try {
            const result = await libraryApi.importSongs(path as string, true)
            totalSuccess += result.success
            totalSkipped += result.skipped
            totalFailed += result.failed
            if (result.errors.length > 0) {
              console.warn('导入警告:', result.errors)
            }
          } catch (err) {
            // 如果是歌词文件且找不到匹配歌曲，不算失败
            console.log('处理文件:', path, err)
          }
        }

        console.log(`导入完成: 成功 ${totalSuccess}, 跳过 ${totalSkipped}, 失败 ${totalFailed}`)
        loadSongs()
        loadArtists()
        loadGenres()
      }
    } catch (err) {
      console.error('导入失败:', err)
    }
  }

  return (
    <div className="flex h-screen bg-dark-950">
      {/* 左侧区域：侧边栏 + 电平表 */}
      <div className="w-20 flex flex-col bg-dark-900 border-r border-dark-700">
        {/* 侧边栏导航 - 垂直排列 */}
        <div className="flex flex-col items-center py-4 space-y-2">
          {/* Logo */}
          <div className="w-10 h-10 bg-primary-600 rounded-lg flex items-center justify-center mb-2">
            <Music className="w-6 h-6" />
          </div>
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                `w-10 h-10 rounded-lg flex items-center justify-center transition-colors ${
                  isActive
                    ? 'bg-primary-600 text-white'
                    : 'text-dark-400 hover:bg-dark-700 hover:text-white'
                }`
              }
              title={item.label}
            >
              <item.icon className="w-5 h-5" />
            </NavLink>
          ))}
        </div>

        {/* 弹性空间 */}
        <div className="flex-1" />

        {/* 电平表区域 - 放在底部 */}
        <div className="p-1 space-y-1 border-t border-dark-700">
          {/* 效果器链中的电平表 - 只有加载了才显示 */}
          {levelMeterSlots.map(slotIndex => (
            <ChainLevelMeter key={slotIndex} slotIndex={slotIndex} />
          ))}

          {/* 输出电平表 - 常驻 */}
          <OutputLevelMeter />
        </div>
      </div>

      {/* 主内容区 */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* 顶部工具栏 */}
        <header className="h-12 bg-dark-900 border-b border-dark-700 flex items-center px-4">
          <div className="flex items-center gap-4">
            <button
              onClick={handleImportFolder}
              className="px-3 py-1.5 bg-primary-600 hover:bg-primary-700 rounded text-sm font-medium transition-colors"
            >
              导入歌曲
            </button>
          </div>

          <div className="ml-auto flex items-center gap-3">
            <span className={`text-xs ${midiConnected ? 'text-green-400' : 'text-dark-400'}`}>
              MIDI: {midiConnected ? midiDeviceName : '未连接'}
            </span>
            <span className="text-xs text-dark-400">|</span>
            <span className="text-xs text-dark-400">CPU: --</span>
          </div>
        </header>

        {/* 内容区域 */}
        <div className="flex-1 flex overflow-hidden">
          {/* 左侧：视频播放器 + 歌词 或 纯歌词区域 */}
          <div
            className="flex flex-col overflow-hidden bg-dark-900 border-r border-dark-700"
            style={{ width: layoutConfig.leftPanelWidth }}
          >
            {/* 播放器 - 处理视频和音频 */}
            <VideoPlayer />

            {/* 歌词显示 */}
            <div className="flex-1 overflow-hidden">
              <LyricsDisplay song={currentSong} currentTime={currentTime} />
            </div>
          </div>

          {/* 左侧分隔条 */}
          <Resizer onResize={handleLeftPanelResize} direction="vertical" minSize={300} maxSize={800} />

          {/* 中间：媒体库/队列 或 效果器页面 */}
          <div className="flex-1 flex flex-col overflow-hidden min-w-[300px]">
            <Outlet />
          </div>

          {/* 右侧：队列 - 效果器页面时隐藏 */}
          {!isEffectsPage && (
            <>
              {/* 右侧分隔条 */}
              <Resizer onResize={handleRightPanelResize} direction="vertical" minSize={250} maxSize={600} />

              <aside
                className="bg-dark-900 border-l border-dark-700 flex flex-col"
                style={{ width: layoutConfig.rightPanelWidth }}
              >
                <Queue />
              </aside>
            </>
          )}
        </div>

        {/* 底部控制面板 - 效果器页面时隐藏 */}
        {!isEffectsPage && (
          <>
            {/* 底部分隔条 */}
            <Resizer onResize={handleBottomPanelResize} direction="horizontal" minSize={120} maxSize={400} />

            <footer
              className="bg-dark-900 border-t border-dark-700 flex flex-col"
              style={{ height: layoutConfig.bottomPanelHeight }}
            >
              {/* 播放控制 - 固定高度 */}
              <div className="flex-shrink-0">
                <Player />
              </div>

              {/* 过场音乐和气氛组 - 弹性高度 */}
              <div className="flex flex-1 min-h-0 border-t border-dark-700">
                <div className="flex-1 border-r border-dark-700 overflow-hidden">
                  <InterludePanel />
                </div>
                <div className="flex-1 overflow-hidden">
                  <AtmospherePanel />
                </div>
              </div>
            </footer>
          </>
        )}
      </div>
    </div>
  )
}

export default MainLayout
