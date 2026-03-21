import { useEffect, useState } from 'react'
import { Play, Pause, Volume2, Plus, Trash2, Music } from 'lucide-react'
import { useInterludeStore, usePlaybackStore } from '@/stores'
import { open } from '@tauri-apps/plugin-dialog'
import { interludeApi, effectApi } from '@/lib/api'

interface DuckingDebugState {
  enabled: boolean
  interludePlaying: boolean
  isDucking: boolean
  threshold: number
  ratio: number
  recoveryDelay: number
  releaseStart: number
  elapsedSinceReleaseStart: number
  remainingTime: number
}

function InterludePanel() {
  const { tracks, state, loadTracks, addTrack, deleteTrack, setVolume, loadState } = useInterludeStore()
  const { status } = usePlaybackStore()
  const [isPlaying, setIsPlaying] = useState(false)
  const [duckingDebug, setDuckingDebug] = useState<DuckingDebugState | null>(null)

  // 正在播放歌曲时禁用过场音乐
  const isSongPlaying = status === 'playing'

  useEffect(() => {
    loadTracks()
    loadState()
  }, [])

  // 同步播放状态
  useEffect(() => {
    setIsPlaying(state.isPlaying)
  }, [state.isPlaying])

  // 定期获取 ducking 调试状态
  useEffect(() => {
    const fetchDuckingDebug = async () => {
      try {
        const debug = await effectApi.getDuckingDebugState()
        setDuckingDebug(debug)
      } catch {
        // 忽略错误（可能实时音频未启动）
      }
    }

    fetchDuckingDebug()
    const interval = setInterval(fetchDuckingDebug, 100)
    return () => clearInterval(interval)
  }, [])

  const handleAddTrack = async () => {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'ogg', 'm4a'] }],
    })
    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected]
      for (const file of files) {
        await addTrack(file)
      }
    }
  }

  const handlePlay = async () => {
    // 歌曲播放中不允许播放过场音乐
    if (isSongPlaying) return

    if (isPlaying) {
      await interludeApi.pauseInterlude()
      setIsPlaying(false)
    } else {
      await interludeApi.playInterlude()
      setIsPlaying(true)
    }
  }

  const handleVolumeChange = async (volume: number) => {
    setVolume(volume)
  }

  return (
    <div className="p-3">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-medium">过场音乐</h3>
        <div className="flex items-center gap-2">
          {/* Ducking 调试显示 */}
          {duckingDebug && (
            <div className="text-xs">
              <span className={duckingDebug.isDucking ? 'text-yellow-400' : 'text-dark-400'}>
                Ducking: {duckingDebug.isDucking ? 'ON' : 'OFF'}
              </span>
              {duckingDebug.isDucking && duckingDebug.releaseStart > 0 && (
                <span className="text-dark-300 ml-1">
                  ({duckingDebug.elapsedSinceReleaseStart}s / {duckingDebug.recoveryDelay}s)
                </span>
              )}
            </div>
          )}
          {!duckingDebug && (
            <span className="text-xs text-dark-400">Ducking: {state.duckingActive ? 'ON' : 'OFF'}</span>
          )}
        </div>
      </div>

      {/* Ducking 详细调试信息 */}
      {duckingDebug && (
        <div className="mb-2 p-2 bg-dark-800 rounded text-xs space-y-1">
          <div className="flex justify-between">
            <span className="text-dark-400">过场音乐:</span>
            <span className={duckingDebug.interludePlaying ? 'text-green-400' : 'text-dark-500'}>
              {duckingDebug.interludePlaying ? '播放中' : '未播放'}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-dark-400">Ducking启用:</span>
            <span className={duckingDebug.enabled ? 'text-green-400' : 'text-dark-500'}>
              {duckingDebug.enabled ? '是' : '否'}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-dark-400">阈值:</span>
            <span className="text-dark-300">{(duckingDebug.threshold * 100).toFixed(0)}%</span>
          </div>
          <div className="flex justify-between">
            <span className="text-dark-400">恢复延迟:</span>
            <span className="text-dark-300">{duckingDebug.recoveryDelay}s</span>
          </div>
          <div className="flex justify-between">
            <span className="text-dark-400">计时器:</span>
            <span className={duckingDebug.releaseStart > 0 ? 'text-yellow-400' : 'text-dark-500'}>
              {duckingDebug.releaseStart > 0 ? `计时中 ${duckingDebug.elapsedSinceReleaseStart}s` : '未启动'}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-dark-400">剩余:</span>
            <span className={duckingDebug.remainingTime > 0 ? 'text-orange-400' : 'text-dark-500'}>
              {duckingDebug.remainingTime > 0 ? `${duckingDebug.remainingTime}s` : '-'}
            </span>
          </div>
        </div>
      )}

      {/* 当前播放 */}
      <div className="flex items-center gap-3 mb-3">
        <button
          onClick={handlePlay}
          disabled={tracks.length === 0 || isSongPlaying}
          className={`w-8 h-8 rounded-full flex items-center justify-center transition-colors ${
            isSongPlaying
              ? 'bg-dark-700 text-dark-500 cursor-not-allowed'
              : 'bg-dark-700 hover:bg-dark-600 disabled:opacity-50'
          }`}
          title={isSongPlaying ? '播放歌曲时不可用过场音乐' : undefined}
        >
          {isPlaying ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
        </button>

        <div className="flex-1">
          <p className="text-sm text-dark-300 truncate">
            {state.currentTrackId
              ? tracks.find((t) => t.id === state.currentTrackId)?.title || '未播放'
              : '未播放'}
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Volume2 className="w-4 h-4 text-dark-400" />
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={state.volume}
            onChange={(e) => handleVolumeChange(parseFloat(e.target.value))}
            className="w-20 h-1 bg-dark-700 rounded-full appearance-none cursor-pointer"
          />
        </div>
      </div>

      {/* 音轨列表 */}
      <div className="max-h-40 overflow-y-auto">
        {tracks.length === 0 ? (
          <div className="text-center py-4 text-dark-500 text-sm">
            <Music className="w-6 h-6 mx-auto mb-1 opacity-50" />
            <p>暂无过场音乐</p>
          </div>
        ) : (
          <div className="space-y-1">
            {tracks.map((track) => (
              <div
                key={track.id}
                className={`flex items-center gap-2 p-2 rounded hover:bg-dark-700 transition-colors group ${
                  state.currentTrackId === track.id ? 'bg-dark-700' : 'bg-dark-800'
                }`}
              >
                <Music className="w-4 h-4 text-dark-400 flex-shrink-0" />
                <span className="flex-1 text-sm truncate">{track.title || '未命名'}</span>
                <button
                  onClick={() => deleteTrack(track.id)}
                  className="opacity-0 group-hover:opacity-100 p-1 hover:bg-dark-600 rounded transition-all text-dark-400 hover:text-red-400"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 添加按钮 */}
      <button
        onClick={handleAddTrack}
        className="w-full mt-2 py-2 bg-dark-700 hover:bg-dark-600 rounded text-sm flex items-center justify-center gap-1 transition-colors"
      >
        <Plus className="w-4 h-4" />
        添加音轨
      </button>
    </div>
  )
}

export default InterludePanel
