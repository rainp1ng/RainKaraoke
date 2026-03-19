import { useState, useEffect } from 'react'
import { Outlet, useLocation } from 'react-router-dom'
import { open } from '@tauri-apps/plugin-dialog'
import Sidebar from './Sidebar'
import Player from '../Player/Player'
import InterludePanel from '../Interlude/InterludePanel'
import AtmospherePanel from '../Atmosphere/AtmospherePanel'
import Queue from '../Queue/Queue'
import { libraryApi, midiApi } from '@/lib/api'
import { useLibraryStore } from '@/stores'

function MainLayout() {
  const location = useLocation()
  const isEffectsPage = location.pathname === '/effects'
  const [midiConnected, setMidiConnected] = useState(false)
  const [midiDeviceName, setMidiDeviceName] = useState<string | null>(null)
  const { loadSongs, loadArtists, loadGenres } = useLibraryStore()

  useEffect(() => {
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
    checkMidiStatus()
    const interval = setInterval(checkMidiStatus, 5000)
    return () => clearInterval(interval)
  }, [])

  const handleImportFolder = async () => {
    try {
      // 支持选择单个文件或文件夹
      const selected = await open({
        multiple: true,
        title: '选择歌曲文件或文件夹',
        filters: [
          {
            name: '媒体文件',
            extensions: ['mp4', 'mkv', 'avi', 'mov', 'mp3', 'flac', 'wav', 'ogg', 'm4a']
          }
        ]
      })

      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected]
        let totalSuccess = 0
        let totalSkipped = 0
        let totalFailed = 0

        for (const path of paths) {
          const result = await libraryApi.importSongs(path as string, true)
          totalSuccess += result.success
          totalSkipped += result.skipped
          totalFailed += result.failed
        }

        console.log(`导入完成: 成功 ${totalSuccess} 首, 跳过 ${totalSkipped} 首, 失败 ${totalFailed} 首`)
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
      {/* 侧边栏 */}
      <Sidebar />

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

        {/* 效果器页面使用全屏布局 */}
        {isEffectsPage ? (
          <div className="flex-1 overflow-hidden">
            <Outlet />
          </div>
        ) : (
          <>
            {/* 内容区域 */}
            <div className="flex-1 flex overflow-hidden">
              {/* 左侧：播放器和歌词 */}
              <div className="flex-1 flex flex-col overflow-hidden">
                <Outlet />
              </div>

              {/* 右侧：队列 */}
              <aside className="w-80 bg-dark-900 border-l border-dark-700 flex flex-col">
                <Queue />
              </aside>
            </div>

            {/* 底部控制面板 */}
            <footer className="bg-dark-900 border-t border-dark-700">
              {/* 播放控制 */}
              <Player />

              {/* 过场音乐和气氛组 */}
              <div className="flex border-t border-dark-700">
                <div className="flex-1 border-r border-dark-700">
                  <InterludePanel />
                </div>
                <div className="flex-1">
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
