import { useEffect, useCallback } from 'react'
import { usePlaybackStore, useQueueStore, useShortcutStore } from '@/stores'
import { listen } from '@tauri-apps/api/event'

export function useKeyboardShortcuts() {
  const { status, pause, resume, stop, currentSong, toggleVocal, isVocal, play } = usePlaybackStore()
  const { items, removeFromQueue, loadQueue } = useQueueStore()
  const { config, midiConfig, learningKey, learningMidi } = useShortcutStore()

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
  }, [config, learningKey, learningMidi, doPlayPause, doStop, doNextSong, doPrevSong, doToggleVocal])

  // 键盘监听
  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [handleKeyDown])

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
      }
    })

    return () => { unlisten.then(fn => fn()) }
  }, [midiConfig, learningMidi, doPlayPause, doNextSong, doPrevSong, doStop, doToggleVocal])
}
