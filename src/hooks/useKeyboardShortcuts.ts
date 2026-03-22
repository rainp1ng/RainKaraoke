import { useEffect, useCallback, useRef } from 'react'
import { usePlaybackStore, useQueueStore, useShortcutStore, useVideoStore } from '@/stores'
import { listen } from '@tauri-apps/api/event'

// 长按检测配置
const LONG_PRESS_DELAY = 400 // 长按触发延迟 (ms)
const SEEK_INTERVAL = 200 // 持续 seek 间隔 (ms)
const SEEK_LONG_PRESS_STEP = 2 // 长按时每次 seek 的秒数
const SEEK_SINGLE_STEP = 10 // 单次按下的 seek 秒数

export function useKeyboardShortcuts() {
  const { status, pause, resume, stop, currentSong, toggleVocal, isVocal, play } = usePlaybackStore()
  const { items, removeFromQueue, loadQueue } = useQueueStore()
  const { config, midiConfig, learningKey, learningMidi } = useShortcutStore()
  const { triggerFullscreen, triggerPiP, hasVideo } = useVideoStore()

  // 长按检测 ref
  const seekTimeoutRef = useRef<number | null>(null)
  const seekIntervalRef = useRef<number | null>(null)
  const isSeekingRef = useRef<'forward' | 'backward' | null>(null)
  const seekStartTimeRef = useRef<number>(0)
  const seekTargetTimeRef = useRef<number>(0)

  // 执行播放/暂停
  const doPlayPause = useCallback(async () => {
    if (status === 'playing') {
      // 正在播放 -> 暂停
      await pause()
    } else if (status === 'paused') {
      // 暂停中 -> 恢复播放
      await resume()
    } else if (status === 'idle' && items.length > 0) {
      // 空闲状态 -> 播放队列第一首
      const firstItem = items[0]
      if (firstItem.song) {
        await play(firstItem.song)
      }
    }
  }, [status, pause, resume, items, play])

  // 执行停止
  const doStop = useCallback(async () => {
    if (status === 'playing' || status === 'paused') {
      await stop()
    }
  }, [status, stop])

  // 执行下一首 - 移除当前播放歌曲，播放下一首
  const doNextSong = useCallback(async () => {
    if (items.length > 0) {
      // 移除队列第一首（当前播放的歌曲）
      const firstItem = items[0]
      await removeFromQueue(firstItem.id)

      // 重新加载队列获取新的第一首
      await loadQueue()
      const { items: newItems } = useQueueStore.getState()

      // 播放新的第一首
      if (newItems.length > 0 && newItems[0].song) {
        await play(newItems[0].song)
      } else {
        // 没有更多歌曲，停止播放
        await stop()
      }
    }
  }, [items, removeFromQueue, loadQueue, play, stop])

  // 执行上一首/重播
  const doPrevSong = useCallback(async () => {
    if (currentSong) {
      const { seek } = usePlaybackStore.getState()
      await seek(0)
    }
  }, [currentSong])

  // 执行切换原唱/伴奏
  const doToggleVocal = useCallback(async () => {
    if (status === 'playing' || status === 'paused') {
      await toggleVocal(!isVocal)
    }
  }, [status, isVocal, toggleVocal])

  // 执行全屏切换
  const doToggleFullscreen = useCallback(() => {
    if (hasVideo) {
      triggerFullscreen()
    }
  }, [hasVideo, triggerFullscreen])

  // 执行画中画切换
  const doTogglePiP = useCallback(() => {
    if (hasVideo) {
      triggerPiP()
    }
  }, [hasVideo, triggerPiP])

  // 清除 seek 定时器
  const clearSeekTimers = useCallback(() => {
    if (seekTimeoutRef.current) {
      clearTimeout(seekTimeoutRef.current)
      seekTimeoutRef.current = null
    }
    if (seekIntervalRef.current) {
      clearInterval(seekIntervalRef.current)
      seekIntervalRef.current = null
    }
    isSeekingRef.current = null
  }, [])

  // 执行单次 seek (短按时用)
  const doSingleSeek = useCallback((direction: 'forward' | 'backward') => {
    const { currentTime, duration, seek } = usePlaybackStore.getState()
    if (duration <= 0) return

    const newTime = direction === 'forward'
      ? Math.min(currentTime + SEEK_SINGLE_STEP, duration)
      : Math.max(currentTime - SEEK_SINGLE_STEP, 0)
    seek(newTime)
  }, [])

  // 开始 seek（检测长按）
  const startSeek = useCallback((direction: 'forward' | 'backward') => {
    seekStartTimeRef.current = Date.now()

    // 记录初始时间
    const { currentTime, duration } = usePlaybackStore.getState()
    seekTargetTimeRef.current = currentTime

    // 设置长按检测：超过延迟时间后开始持续 seek
    seekTimeoutRef.current = window.setTimeout(() => {
      // 进入长按模式，开始持续 seek
      seekIntervalRef.current = window.setInterval(() => {
        const { duration, seek } = usePlaybackStore.getState()
        if (duration <= 0) {
          clearSeekTimers()
          return
        }

        // 使用 ref 追踪目标时间，避免异步问题
        seekTargetTimeRef.current = direction === 'forward'
          ? Math.min(seekTargetTimeRef.current + SEEK_LONG_PRESS_STEP, duration)
          : Math.max(seekTargetTimeRef.current - SEEK_LONG_PRESS_STEP, 0)

        seek(seekTargetTimeRef.current)
      }, SEEK_INTERVAL)
    }, LONG_PRESS_DELAY)
  }, [clearSeekTimers])

  // 停止 seek
  const stopSeek = useCallback((direction: 'forward' | 'backward') => {
    const elapsed = Date.now() - seekStartTimeRef.current

    // 如果松开时间小于长按延迟，说明是短按，执行单次 10s 跳转
    if (elapsed < LONG_PRESS_DELAY && seekTimeoutRef.current) {
      doSingleSeek(direction)
    }

    clearSeekTimers()
  }, [doSingleSeek, clearSeekTimers])

  // 键盘事件处理
  const handleKeyDown = useCallback(async (e: KeyboardEvent) => {
    // 如果正在学习快捷键，不触发任何动作
    if (learningKey || learningMidi) {
      return
    }

    // 忽略在输入框中的按键
    const target = e.target as HTMLElement
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      return
    }

    const code = e.code

    // 播放/暂停
    if (code === config.playPause) {
      e.preventDefault()
      await doPlayPause()
      return
    }

    // 停止
    if (code === config.stop) {
      e.preventDefault()
      await doStop()
      return
    }

    // 下一首
    if (code === config.nextSong) {
      e.preventDefault()
      await doNextSong()
      return
    }

    // 上一首/重播
    if (code === config.prevSong) {
      e.preventDefault()
      await doPrevSong()
      return
    }

    // 切换原唱/伴奏
    if (code === config.toggleVocal) {
      e.preventDefault()
      await doToggleVocal()
      return
    }

    // 全屏
    if (code === config.fullscreen) {
      e.preventDefault()
      doToggleFullscreen()
      return
    }

    // 画中画
    if (code === config.pip) {
      e.preventDefault()
      doTogglePiP()
      return
    }

    // 快进/快退（左右方向键）
    if (code === 'ArrowRight' && status !== 'idle') {
      e.preventDefault()
      if (e.repeat) return
      if (!isSeekingRef.current) {
        isSeekingRef.current = 'forward'
        startSeek('forward')
      }
      return
    }

    if (code === 'ArrowLeft' && status !== 'idle') {
      e.preventDefault()
      if (e.repeat) return
      if (!isSeekingRef.current) {
        isSeekingRef.current = 'backward'
        startSeek('backward')
      }
      return
    }
  }, [config, learningKey, learningMidi, status, doPlayPause, doStop, doNextSong, doPrevSong, doToggleVocal, doToggleFullscreen, doTogglePiP, startSeek])

  // 键盘松开事件处理
  const handleKeyUp = useCallback((e: KeyboardEvent) => {
    const code = e.code

    // 松开方向键时停止 seek
    if (code === 'ArrowRight' && isSeekingRef.current === 'forward') {
      stopSeek('forward')
    } else if (code === 'ArrowLeft' && isSeekingRef.current === 'backward') {
      stopSeek('backward')
    }
  }, [stopSeek])

  // 键盘监听
  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    window.addEventListener('keyup', handleKeyUp)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
      window.removeEventListener('keyup', handleKeyUp)
      clearSeekTimers()
    }
  }, [handleKeyDown, handleKeyUp, clearSeekTimers])

  // MIDI 事件监听
  useEffect(() => {
    const unlisten = listen('midi:event', async (event) => {
      // 如果正在学习 MIDI，不触发动作
      if (learningMidi) return

      const midiEvent = event.payload as {
        messageType: 'NOTE' | 'CC' | 'PC'
        channel: number
        data1: number
        data2: number
        isOn: boolean
      }

      // 只响应 Note On 事件
      if (midiEvent.messageType !== 'NOTE' || !midiEvent.isOn) return

      // 检查是否匹配任何 MIDI 绑定
      const checkMidi = (binding: { note: number; channel: number } | null) => {
        return binding && binding.note === midiEvent.data1 && binding.channel === midiEvent.channel
      }

      if (checkMidi(midiConfig.playPause)) {
        await doPlayPause()
      } else if (checkMidi(midiConfig.nextSong)) {
        await doNextSong()
      } else if (checkMidi(midiConfig.prevSong)) {
        await doPrevSong()
      } else if (checkMidi(midiConfig.stop)) {
        await doStop()
      } else if (checkMidi(midiConfig.toggleVocal)) {
        await doToggleVocal()
      } else if (checkMidi(midiConfig.fullscreen)) {
        doToggleFullscreen()
      } else if (checkMidi(midiConfig.pip)) {
        doTogglePiP()
      }
    })

    return () => { unlisten.then(fn => fn()) }
  }, [midiConfig, learningMidi, doPlayPause, doNextSong, doPrevSong, doStop, doToggleVocal, doToggleFullscreen, doTogglePiP])
}
