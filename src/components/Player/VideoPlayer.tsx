import { useRef, useEffect, useCallback } from 'react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Maximize, PictureInPicture, X } from 'lucide-react'
import { usePlaybackStore, useQueueStore, useVideoStore } from '@/stores'
import { semitonesToRatio } from '@/utils/pitchShifter'

function VideoPlayer() {
  const videoRef = useRef<HTMLVideoElement>(null)
  const audioRef = useRef<HTMLAudioElement>(null)
  const separateAudioRef = useRef<HTMLAudioElement>(null)
  const lastTimeRef = useRef<number>(0)
  const lastFullscreenTriggerRef = useRef<number>(0)
  const lastPipTriggerRef = useRef<number>(0)
  const isSeekingRef = useRef<boolean>(false)

  const {
    currentSong,
    status,
    isVocal,
    pitch,
    speed,
    volume,
    currentTime,
    setCurrentTime,
    setDuration,
  } = usePlaybackStore()

  const {
    isFullscreen,
    isPiP,
    fullscreenTrigger,
    pipTrigger,
    setHasVideo,
    setIsFullscreen,
    setIsPiP,
  } = useVideoStore()

  // 获取视频路径
  const videoPath = currentSong?.videoPath

  // 根据原唱/伴唱模式选择音频路径
  const audioPath = isVocal
    ? (currentSong?.vocalAudioPath || currentSong?.instrumentalAudioPath || null)
    : (currentSong?.instrumentalAudioPath || currentSong?.vocalAudioPath || null)

  // 是否有视频
  const hasVideo = videoPath != null

  // 主要播放源
  const primaryMediaPath = hasVideo ? videoPath : audioPath
  const primaryMediaSrc = primaryMediaPath ? convertFileSrc(primaryMediaPath) : null

  // 是否使用独立音频（有视频且有独立音频轨时）
  const useSeparateAudio = hasVideo && audioPath != null

  // 获取主播放器
  const getMainPlayer = () => hasVideo ? videoRef.current : audioRef.current

  // 同步 hasVideo 到 store
  useEffect(() => {
    setHasVideo(hasVideo)
  }, [hasVideo, setHasVideo])

  // 全屏切换函数
  const toggleFullscreen = useCallback(async () => {
    try {
      const win = getCurrentWindow()
      const fs = await win.isFullscreen()
      await win.setFullscreen(!fs)
      // 状态更新由 onResized 事件监听器处理，避免竞态条件
    } catch (err) {
      console.error('全屏失败:', err)
    }
  }, [])

  // 画中画切换函数
  const togglePiP = useCallback(async () => {
    const video = videoRef.current
    if (!video || !hasVideo) return

    try {
      if (document.pictureInPictureElement) {
        await document.exitPictureInPicture()
        setIsPiP(false)
      } else {
        await video.requestPictureInPicture()
        setIsPiP(true)
      }
    } catch (err) {
      console.error('画中画失败:', err)
    }
  }, [hasVideo, setIsPiP])

  // 监听全屏触发
  useEffect(() => {
    // 只有 trigger 真正变化时才触发，避免 hasVideo 变化时重复触发
    if (fullscreenTrigger !== lastFullscreenTriggerRef.current && hasVideo) {
      lastFullscreenTriggerRef.current = fullscreenTrigger
      toggleFullscreen()
    }
  }, [fullscreenTrigger, hasVideo, toggleFullscreen])

  // 监听画中画触发
  useEffect(() => {
    // 只有 trigger 真正变化时才触发
    if (pipTrigger !== lastPipTriggerRef.current && hasVideo) {
      lastPipTriggerRef.current = pipTrigger
      togglePiP()
    }
  }, [pipTrigger, hasVideo, togglePiP])

  // 播放控制
  useEffect(() => {
    const player = getMainPlayer()
    if (!player) return

    if (status === 'playing') {
      player.play().catch(e => console.error('播放失败:', e))
      if (useSeparateAudio && separateAudioRef.current) {
        separateAudioRef.current.play().catch(e => console.error('独立音频播放失败:', e))
      }
    } else if (status === 'paused') {
      player.pause()
      if (useSeparateAudio && separateAudioRef.current) {
        separateAudioRef.current.pause()
      }
    } else if (status === 'idle') {
      player.pause()
      player.currentTime = 0
      if (useSeparateAudio && separateAudioRef.current) {
        separateAudioRef.current.pause()
        separateAudioRef.current.currentTime = 0
      }
    }
  }, [status, hasVideo, useSeparateAudio])

  // 速度控制
  useEffect(() => {
    const player = getMainPlayer()
    if (player) player.playbackRate = speed
    if (useSeparateAudio && separateAudioRef.current) {
      separateAudioRef.current.playbackRate = speed
    }
  }, [speed, hasVideo, useSeparateAudio])

  // 音调控制（升降调）
  useEffect(() => {
    const applyPitchShift = (player: HTMLMediaElement | null) => {
      if (!player) return

      // 使用 preservesPitch 属性实现真正的 pitch shifting
      // @ts-ignore - preservesPitch 是标准 API
      if ('preservesPitch' in player) {
        // @ts-ignore
        player.preservesPitch = false
      }

      // 计算 playbackRate
      // pitchRatio = 2^(semitones/12)
      // 总 playbackRate = speed * pitchRatio
      const pitchRatio = semitonesToRatio(pitch)
      player.playbackRate = speed * pitchRatio

      console.log(`[PitchShift] pitch=${pitch} semitones, ratio=${pitchRatio.toFixed(3)}, playbackRate=${(speed * pitchRatio).toFixed(3)}`)
    }

    applyPitchShift(getMainPlayer())
    if (useSeparateAudio && separateAudioRef.current) {
      applyPitchShift(separateAudioRef.current)
    }
  }, [pitch, speed, hasVideo, useSeparateAudio])

  // 音量控制
  useEffect(() => {
    const player = getMainPlayer()
    if (player) player.volume = volume
    if (useSeparateAudio && separateAudioRef.current) {
      separateAudioRef.current.volume = volume
    }
  }, [volume, hasVideo, useSeparateAudio])

  // 进度跳转
  useEffect(() => {
    const player = getMainPlayer()
    if (!player) return

    const diff = Math.abs(currentTime - lastTimeRef.current)
    // diff >= 0.5 表示非正常播放的时间变化（即 seek 操作）
    // 正常播放时 timeupdate 每 250ms 更新一次，diff 约 0.25 秒
    if (diff >= 0.5 && Math.abs(currentTime - player.currentTime) > 0.3) {
      isSeekingRef.current = true
      player.currentTime = currentTime
      if (useSeparateAudio && separateAudioRef.current) {
        separateAudioRef.current.currentTime = currentTime
      }
      // 短暂延迟后重置标志，让 timeupdate 事件有机会恢复更新
      setTimeout(() => {
        isSeekingRef.current = false
      }, 100)
    }
    lastTimeRef.current = currentTime
  }, [currentTime, hasVideo, useSeparateAudio])

  // 媒体加载完成
  const handleLoadedMetadata = () => {
    const player = getMainPlayer()
    if (player) {
      // 应用当前状态到新加载的媒体
      player.volume = volume  // 应用当前音量

      // 应用音调（升降调）
      // @ts-ignore - preservesPitch 是标准 API
      if ('preservesPitch' in player) {
        // @ts-ignore
        player.preservesPitch = false
      }
      const pitchRatio = semitonesToRatio(pitch)
      player.playbackRate = speed * pitchRatio

      setDuration(player.duration)
      if (status === 'playing') {
        player.play().catch(e => console.error('自动播放失败:', e))
      }
    }
    // 同样设置独立音频的音量和音调
    if (useSeparateAudio && separateAudioRef.current) {
      separateAudioRef.current.volume = volume
      // @ts-ignore
      if ('preservesPitch' in separateAudioRef.current) {
        // @ts-ignore
        separateAudioRef.current.preservesPitch = false
      }
      const pitchRatio = semitonesToRatio(pitch)
      separateAudioRef.current.playbackRate = speed * pitchRatio
    }
  }

  // 时间更新
  const handleTimeUpdate = () => {
    const player = getMainPlayer()

    // 如果正在进行 seek 操作，跳过 currentTime 更新避免竞争，但仍需同步独立音频
    if (!isSeekingRef.current && player) {
      setCurrentTime(player.currentTime)
    }

    // 同步独立音频轨道
    if (useSeparateAudio && videoRef.current && separateAudioRef.current) {
      videoRef.current.muted = true
      const diff = Math.abs(videoRef.current.currentTime - separateAudioRef.current.currentTime)
      if (diff > 0.3) {
        separateAudioRef.current.currentTime = videoRef.current.currentTime
      }
    }
  }

  // 播放结束
  const handleEnded = async () => {
    const { continuousPlay, onSongEnded } = usePlaybackStore.getState()
    const { items, removeFromQueue, loadQueue } = useQueueStore.getState()

    // 如果连播开启且队列中还有下一首
    if (continuousPlay && items.length > 0) {
      const currentItem = items[0]
      await removeFromQueue(currentItem.id)
      await loadQueue()

      const { items: newItems } = useQueueStore.getState()
      if (newItems.length > 0 && newItems[0].song) {
        // 播放下一首
        usePlaybackStore.getState().play(newItems[0].song)
      } else {
        // 队列已空
        onSongEnded()
      }
    } else {
      onSongEnded()
    }
  }

  // 媒体错误
  const handleError = (e: React.SyntheticEvent<HTMLMediaElement, Event>) => {
    const target = e.target as HTMLMediaElement
    console.error('媒体错误:', target.error)
  }

  // 监听窗口全屏状态变化
  useEffect(() => {
    const win = getCurrentWindow()

    // 初始化时同步全屏状态
    win.isFullscreen().then(fs => {
      setIsFullscreen(fs)
    }).catch(err => {
      console.error('初始化全屏状态失败:', err)
    })

    const unlisten = win.onResized(async () => {
      try {
        const fs = await win.isFullscreen()
        setIsFullscreen(fs)
      } catch (err) {
        console.error('检查全屏状态失败:', err)
      }
    })
    return () => { unlisten.then((f: () => void) => f()) }
  }, [setIsFullscreen])

  // 全屏模式下的视频播放器
  if (isFullscreen && hasVideo) {
    return (
      <div className="fixed inset-0 z-50 bg-black flex items-center justify-center">
        <video
          ref={videoRef}
          src={primaryMediaSrc!}
          className="w-full h-full object-contain"
          onLoadedMetadata={handleLoadedMetadata}
          onTimeUpdate={handleTimeUpdate}
          onEnded={handleEnded}
          onError={handleError}
          playsInline
        />
        {useSeparateAudio && audioPath && (
          <audio ref={separateAudioRef} src={convertFileSrc(audioPath)} />
        )}
        {/* 退出全屏按钮 */}
        <button
          onClick={toggleFullscreen}
          className="absolute top-4 right-4 p-3 bg-black/60 hover:bg-black/80 rounded-full z-10"
          title="退出全屏"
        >
          <X className="w-6 h-6" />
        </button>
      </div>
    )
  }

  if (!currentSong) {
    return (
      <div className="aspect-video bg-dark-900 flex items-center justify-center">
        <p className="text-dark-500">选择歌曲开始播放</p>
      </div>
    )
  }

  if (!primaryMediaSrc) {
    return (
      <div className="aspect-video bg-dark-900 flex items-center justify-center">
        <p className="text-dark-400">无可播放的媒体文件</p>
      </div>
    )
  }

  // 无视频时只渲染 audio 元素
  if (!hasVideo) {
    return (
      <audio
        ref={audioRef}
        src={primaryMediaSrc}
        onLoadedMetadata={handleLoadedMetadata}
        onTimeUpdate={handleTimeUpdate}
        onEnded={handleEnded}
        onError={handleError}
      />
    )
  }

  // 有视频时渲染视频播放器
  return (
    <div className="relative aspect-video bg-black group">
      <video
        ref={videoRef}
        src={primaryMediaSrc}
        className="w-full h-full object-contain"
        onLoadedMetadata={handleLoadedMetadata}
        onTimeUpdate={handleTimeUpdate}
        onEnded={handleEnded}
        onError={handleError}
        playsInline
        controls={false}
      />

      {/* 独立音频轨道 */}
      {useSeparateAudio && audioPath && (
        <audio ref={separateAudioRef} src={convertFileSrc(audioPath)} />
      )}

      {/* 控制按钮 */}
      <div className="absolute top-3 right-3 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          onClick={togglePiP}
          className="p-2 bg-black/60 hover:bg-black/80 rounded-lg"
          title="画中画"
        >
          <PictureInPicture className="w-4 h-4" />
        </button>
        <button
          onClick={toggleFullscreen}
          className="p-2 bg-black/60 hover:bg-black/80 rounded-lg"
          title="全屏"
        >
          <Maximize className="w-4 h-4" />
        </button>
      </div>
    </div>
  )
}

export default VideoPlayer
