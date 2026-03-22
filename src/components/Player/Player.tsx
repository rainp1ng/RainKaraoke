import { useRef, useCallback } from 'react'
import { Play, Pause, SkipForward, Volume2, Mic, MicOff, Repeat } from 'lucide-react'
import { usePlaybackStore, useQueueStore } from '@/stores'
import { formatDuration } from '@/utils/format'

function Player() {
  const progressRef = useRef<HTMLDivElement>(null)

  const {
    status,
    currentSong,
    currentTime,
    duration,
    isVocal,
    pitch,
    speed,
    volume,
    continuousPlay,
    play,
    pause,
    resume,
    stop,
    toggleVocal,
    setPitch,
    setSpeed,
    setVolume,
    setContinuousPlay,
    seek,
  } = usePlaybackStore()

  const { items, removeFromQueue, loadQueue } = useQueueStore()

  const isPlaying = status === 'playing'
  const isPaused = status === 'paused'
  const isIdle = status === 'idle'
  const hasSong = currentSong !== null

  // 处理播放/暂停
  const handlePlayPause = async () => {
    if (isPlaying) {
      // 正在播放 -> 暂停
      await pause()
    } else if (isPaused) {
      // 暂停中 -> 恢复播放
      await resume()
    } else if (isIdle && items.length > 0) {
      // 空闲状态 -> 播放队列第一首
      const firstItem = items[0]
      if (firstItem.song) {
        await play(firstItem.song)
      }
    }
  }

  // 处理停止
  const handleStop = () => {
    stop()
  }

  // 处理下一首 - 移除当前歌曲，播放下一首
  const handleNext = async () => {
    if (items.length === 0) return

    // 移除队列第一首
    const currentItem = items[0]
    await removeFromQueue(currentItem.id)

    // 重新加载队列并播放新的第一首
    await loadQueue()
    const { items: newItems } = useQueueStore.getState()
    if (newItems.length > 0 && newItems[0].song) {
      play(newItems[0].song)
    } else {
      // 队列已空，停止播放
      stop()
    }
  }

  // 点击进度条跳转
  const handleProgressClick = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (!progressRef.current || duration === 0) return

    const rect = progressRef.current.getBoundingClientRect()
    const clickX = e.clientX - rect.left
    const percent = clickX / rect.width
    const newTime = percent * duration

    seek(newTime)
  }, [duration, seek])

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
              -
            </button>
            <span className="px-2 text-sm text-dark-400 min-w-[3rem] text-center">
              {pitch > 0 ? '+' : ''}{pitch}
            </span>
            <button
              onClick={() => setPitch(Math.min(12, pitch + 1))}
              disabled={!hasSong}
              className="px-2 py-1 bg-dark-700 hover:bg-dark-600 disabled:opacity-50 disabled:cursor-not-allowed rounded text-sm transition-colors"
            >
              +
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
          ref={progressRef}
          className="flex-1 h-1.5 bg-dark-700 rounded-full overflow-hidden cursor-pointer group"
          onClick={handleProgressClick}
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
              (hasSong || items.length > 0)
                ? 'bg-primary-600 hover:bg-primary-700'
                : 'bg-dark-700 opacity-50 cursor-not-allowed'
            }`}
            title={isPlaying ? '暂停' : isPaused ? '继续播放' : '播放队列第一首'}
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
          {/* 连播开关 */}
          <button
            onClick={() => setContinuousPlay(!continuousPlay)}
            className={`w-10 h-10 rounded-full flex items-center justify-center transition-colors ${
              continuousPlay
                ? 'bg-primary-600 hover:bg-primary-700 text-white'
                : 'bg-dark-700 hover:bg-dark-600 text-dark-400'
            }`}
            title={continuousPlay ? '连播开启 - 点击关闭' : '连播关闭 - 点击开启'}
          >
            <Repeat className={`w-5 h-5 ${!continuousPlay ? 'opacity-50' : ''}`} />
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
            value={volume}
            onChange={(e) => setVolume(Number(e.target.value))}
            className="w-24 h-1 bg-dark-700 rounded-full appearance-none cursor-pointer"
          />
          <span className="text-xs text-dark-400 w-8">{Math.round(volume * 100)}%</span>
        </div>
      </div>
    </div>
  )
}

export default Player
