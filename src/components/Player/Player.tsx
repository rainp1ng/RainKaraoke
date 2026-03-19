import { Play, Pause, SkipBack, SkipForward, Volume2, Mic, MicOff } from 'lucide-react'
import { usePlaybackStore, useQueueStore } from '@/stores'
import { formatDuration } from '@/utils/format'

function Player() {
  const {
    status,
    currentSong,
    currentTime,
    duration,
    isVocal,
    pitch,
    speed,
    play,
    pause,
    resume,
    stop,
    seek,
    toggleVocal,
    setPitch,
    setSpeed,
    setCurrentTime,
  } = usePlaybackStore()

  const { items, playNext } = useQueueStore()

  const isPlaying = status === 'playing'
  const isPaused = status === 'paused'
  const hasSong = currentSong !== null

  // 处理播放/暂停
  const handlePlayPause = () => {
    if (!hasSong) return

    if (isPlaying) {
      pause()
    } else if (isPaused) {
      resume()
    }
  }

  // 处理停止
  const handleStop = () => {
    stop()
  }

  // 处理下一首
  const handleNext = async () => {
    if (items.length > 0) {
      const nextItem = items[0]
      if (nextItem.song) {
        await play(nextItem.song)
        await playNext()
      }
    }
  }

  // 处理进度条点击
  const handleSeek = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!hasSong || duration === 0) return

    const rect = e.currentTarget.getBoundingClientRect()
    const percent = (e.clientX - rect.left) / rect.width
    const newTime = percent * duration
    seek(newTime)
  }

  // 检查是否有独立伴奏
  const hasInstrumental = currentSong?.hasInstrumental || false

  // 进度百分比
  const progressPercent = duration > 0 ? (currentTime / duration) * 100 : 0

  return (
    <div className="p-4">
      {/* 当前歌曲信息 */}
      <div className="flex items-center justify-between mb-3">
        <div className="min-w-0 flex-1">
          {hasSong ? (
            <>
              <h3 className="font-medium text-white truncate">{currentSong.title}</h3>
              <p className="text-sm text-dark-400 truncate">{currentSong.artist || '未知歌手'}</p>
            </>
          ) : (
            <>
              <h3 className="font-medium text-dark-400">未选择歌曲</h3>
              <p className="text-sm text-dark-500">从媒体库选择歌曲开始播放</p>
            </>
          )}
        </div>

        <div className="flex items-center gap-2 ml-4">
          {/* 原唱/伴唱切换 */}
          <button
            onClick={() => toggleVocal(!isVocal)}
            disabled={!hasSong || !hasInstrumental}
            className={`px-3 py-1 rounded text-sm transition-colors ${
              !hasSong || !hasInstrumental
                ? 'bg-dark-700 text-dark-500 cursor-not-allowed'
                : isVocal
                ? 'bg-green-600 hover:bg-green-700 text-white'
                : 'bg-blue-600 hover:bg-blue-700 text-white'
            }`}
            title={hasInstrumental ? (isVocal ? '切换到伴奏' : '切换到原唱') : '无独立伴奏音轨'}
          >
            {isVocal ? (
              <span className="flex items-center gap-1">
                <Mic className="w-3 h-3" />
                原唱
              </span>
            ) : (
              <span className="flex items-center gap-1">
                <MicOff className="w-3 h-3" />
                伴奏
              </span>
            )}
          </button>

          {/* 音调调节 */}
          <div className="flex items-center gap-1">
            <button
              onClick={() => setPitch(Math.max(-12, pitch - 1))}
              disabled={!hasSong}
              className="px-2 py-1 bg-dark-700 hover:bg-dark-600 disabled:opacity-50 disabled:cursor-not-allowed rounded text-sm transition-colors"
            >
              -{Math.abs(pitch - 1) > 0 ? Math.abs(pitch - 1) : ''}
            </button>
            <span className="px-2 text-sm text-dark-400 min-w-[3rem] text-center">
              {pitch > 0 ? '+' : ''}{pitch}
            </span>
            <button
              onClick={() => setPitch(Math.min(12, pitch + 1))}
              disabled={!hasSong}
              className="px-2 py-1 bg-dark-700 hover:bg-dark-600 disabled:opacity-50 disabled:cursor-not-allowed rounded text-sm transition-colors"
            >
              +{pitch + 1 > 0 ? pitch + 1 : ''}
            </button>
          </div>

          {/* 速度调节 */}
          <select
            value={speed}
            onChange={(e) => setSpeed(Number(e.target.value))}
            disabled={!hasSong}
            className="bg-dark-700 hover:bg-dark-600 disabled:opacity-50 disabled:cursor-not-allowed rounded px-2 py-1 text-sm border-none outline-none"
          >
            <option value={0.5}>0.5x</option>
            <option value={0.75}>0.75x</option>
            <option value={1.0}>1.0x</option>
            <option value={1.25}>1.25x</option>
            <option value={1.5}>1.5x</option>
            <option value={2.0}>2.0x</option>
          </select>
        </div>
      </div>

      {/* 进度条 */}
      <div className="flex items-center gap-3 mb-3">
        <span className="text-xs text-dark-400 w-10 text-right">
          {formatDuration(Math.floor(currentTime))}
        </span>
        <div
          className="flex-1 h-1.5 bg-dark-700 rounded-full overflow-hidden cursor-pointer group"
          onClick={handleSeek}
        >
          <div
            className="h-full bg-primary-500 group-hover:bg-primary-400 transition-colors rounded-full relative"
            style={{ width: `${progressPercent}%` }}
          >
            <div className="absolute right-0 top-1/2 -translate-y-1/2 w-3 h-3 bg-white rounded-full opacity-0 group-hover:opacity-100 transition-opacity" />
          </div>
        </div>
        <span className="text-xs text-dark-400 w-10">
          {formatDuration(duration)}
        </span>
      </div>

      {/* 控制按钮 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <button
            onClick={handleStop}
            disabled={!hasSong}
            className="w-10 h-10 rounded-full bg-dark-700 hover:bg-dark-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center transition-colors"
            title="停止"
          >
            <div className="w-3 h-3 bg-current rounded-sm" />
          </button>
          <button
            onClick={handlePlayPause}
            disabled={!hasSong && items.length === 0}
            className={`w-12 h-12 rounded-full flex items-center justify-center transition-colors ${
              hasSong || items.length > 0
                ? 'bg-primary-600 hover:bg-primary-700'
                : 'bg-dark-700 opacity-50 cursor-not-allowed'
            }`}
            title={isPlaying ? '暂停' : '播放'}
          >
            {isPlaying ? (
              <Pause className="w-6 h-6" />
            ) : (
              <Play className="w-6 h-6 ml-0.5" />
            )}
          </button>
          <button
            onClick={handleNext}
            disabled={items.length === 0}
            className="w-10 h-10 rounded-full bg-dark-700 hover:bg-dark-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center transition-colors"
            title="下一首"
          >
            <SkipForward className="w-5 h-5" />
          </button>
        </div>

        {/* 音量控制 */}
        <div className="flex items-center gap-2">
          <Volume2 className="w-4 h-4 text-dark-400" />
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            defaultValue="0.8"
            className="w-24 h-1 bg-dark-700 rounded-full appearance-none cursor-pointer"
          />
          <select className="bg-dark-700 text-sm rounded px-2 py-1 border-none outline-none">
            <option>系统扬声器</option>
          </select>
        </div>
      </div>
    </div>
  )
}

export default Player
