import { useEffect, useState, useRef } from 'react'
import { Plus, Trash2, Settings, Piano, Volume2, Square } from 'lucide-react'
import { useAtmosphereStore } from '@/stores'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { atmosphereApi, audioApi } from '@/lib/api'
import type { AtmosphereSound, MidiMessageType } from '@/types'

// 预设颜色
const PRESET_COLORS = [
  '#22c55e', // green
  '#eab308', // yellow
  '#3b82f6', // blue
  '#ec4899', // pink
  '#f97316', // orange
  '#8b5cf6', // purple
  '#06b6d4', // cyan
  '#ef4444', // red
]

// MIDI 消息类型选项
const MIDI_MESSAGE_TYPES: { value: MidiMessageType; label: string; description: string }[] = [
  { value: 'NOTE', label: 'Note', description: '音符触发 (0-127)' },
  { value: 'CC', label: 'CC', description: '控制变化 (0-127)' },
  { value: 'PC', label: 'PC', description: '程序变化 (0-127)' },
]

interface MidiEvent {
  messageType: 'NOTE' | 'CC' | 'PC'
  channel: number
  data1: number
  data2: number
  isOn: boolean
}

function AtmospherePanel() {
  const {
    sounds,
    midiStatus,
    midiDevices,
    loadSounds,
    addSound,
    updateSound,
    deleteSound,
    playSound,
    stopSound,
    loadMidiDevices,
    connectMidi,
    disconnectMidi,
    loadMidiStatus,
  } = useAtmosphereStore()

  const [showMidiSettings, setShowMidiSettings] = useState(false)
  const [editingSound, setEditingSound] = useState<AtmosphereSound | null>(null)
  const [midiLearnMode, setMidiLearnMode] = useState<number | null>(null)
  const [masterVolume, setMasterVolume] = useState(0.8)

  // 停止按钮 MIDI 配置
  const [stopMidiConfig, setStopMidiConfig] = useState<{
    messageType: MidiMessageType
    note: number | null
    channel: number
  }>({
    messageType: 'NOTE',
    note: null,
    channel: 0,
  })
  const [stopMidiLearnMode, setStopMidiLearnMode] = useState(false)

  // 使用 ref 存储最新状态，避免 useEffect 依赖变化
  const soundsRef = useRef(sounds)
  const playSoundRef = useRef(playSound)
  const stopSoundRef = useRef(stopSound)
  const midiLearnModeRef = useRef(midiLearnMode)
  const editingSoundRef = useRef(editingSound)
  const stopMidiLearnModeRef = useRef(stopMidiLearnMode)
  const stopMidiConfigRef = useRef(stopMidiConfig)

  useEffect(() => {
    soundsRef.current = sounds
    playSoundRef.current = playSound
    stopSoundRef.current = stopSound
    midiLearnModeRef.current = midiLearnMode
    editingSoundRef.current = editingSound
    stopMidiLearnModeRef.current = stopMidiLearnMode
    stopMidiConfigRef.current = stopMidiConfig
  }, [sounds, playSound, stopSound, midiLearnMode, editingSound, stopMidiLearnMode, stopMidiConfig])

  useEffect(() => {
    loadSounds()
    loadMidiDevices()
    loadMidiStatus()

    // 加载气氛组音量和停止按钮 MIDI 配置
    const loadVolume = async () => {
      try {
        const config = await audioApi.getAudioConfig()
        setMasterVolume(config.atmosphereVolume ?? 0.8)

        // 加载停止按钮 MIDI 配置
        if (config.atmosphereStopMidiNote !== null) {
          setStopMidiConfig({
            messageType: (config.atmosphereStopMidiMessageType as MidiMessageType) || 'NOTE',
            note: config.atmosphereStopMidiNote,
            channel: config.atmosphereStopMidiChannel ?? 0,
          })
        }
      } catch (e) {
        console.error('Failed to load atmosphere volume:', e)
      }
    }
    loadVolume()

    // 设置音效播放完成监听器
    let unlistenPromise: Promise<() => void> | null = null
    const setupListener = async () => {
      unlistenPromise = useAtmosphereStore.getState().setupSoundEndedListener()
    }
    setupListener()

    return () => {
      if (unlistenPromise) {
        unlistenPromise.then((unlisten) => unlisten())
      }
    }
  }, [])

  // 监听 MIDI 事件 - 只订阅一次
  useEffect(() => {
    let mounted = true

    const setupListener = async () => {
      const unlisten = await listen<MidiEvent>('midi:event', (event) => {
        if (!mounted) return

        const midiEvent = event.payload
        const currentSounds = soundsRef.current
        const currentMidiLearnMode = midiLearnModeRef.current
        const currentEditingSound = editingSoundRef.current
        const currentStopMidiLearnMode = stopMidiLearnModeRef.current
        const currentStopMidiConfig = stopMidiConfigRef.current

        // 如果处于停止按钮 MIDI 学习模式
        if (currentStopMidiLearnMode) {
          const newConfig = {
            messageType: midiEvent.messageType,
            note: midiEvent.data1,
            channel: midiEvent.channel,
          }
          setStopMidiConfig(newConfig)
          setStopMidiLearnMode(false)
          // 保存到数据库
          audioApi.saveAudioConfig({
            atmosphereStopMidiMessageType: midiEvent.messageType,
            atmosphereStopMidiNote: midiEvent.data1,
            atmosphereStopMidiChannel: midiEvent.channel,
          }).catch(e => console.error('Failed to save stop MIDI config:', e))
          return
        }

        // 如果处于音效 MIDI 学习模式
        if (currentMidiLearnMode !== null && currentEditingSound?.id === currentMidiLearnMode) {
          setEditingSound({
            ...currentEditingSound,
            midiMessageType: midiEvent.messageType,
            midiNote: midiEvent.data1,
            midiChannel: midiEvent.channel,
          })
          setMidiLearnMode(null)
          return
        }

        // 检查是否匹配停止按钮
        if (currentStopMidiConfig.note !== null) {
          const matchesStop =
            currentStopMidiConfig.messageType === midiEvent.messageType &&
            currentStopMidiConfig.note === midiEvent.data1 &&
            currentStopMidiConfig.channel === midiEvent.channel &&
            (midiEvent.messageType !== 'NOTE' || midiEvent.isOn)

          if (matchesStop) {
            stopSoundRef.current()
            return
          }
        }

        // 触发对应的音效
        const matchedSound = currentSounds.find((s) => {
          if (s.midiNote === null) return false
          return (
            s.midiMessageType === midiEvent.messageType &&
            s.midiNote === midiEvent.data1 &&
            s.midiChannel === midiEvent.channel &&
            (midiEvent.messageType !== 'NOTE' || midiEvent.isOn)
          )
        })

        if (matchedSound) {
          playSoundRef.current(matchedSound.id)
        }
      })

      return unlisten
    }

    const promise = setupListener()

    return () => {
      mounted = false
      promise.then((unlisten) => unlisten())
    }
  }, []) // 空依赖数组，只订阅一次

  const handleAddSound = async () => {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'ogg', 'm4a'] }],
    })
    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected]
      for (let i = 0; i < files.length; i++) {
        const file = files[i]
        const name = file.split('/').pop()?.replace(/\.[^/.]+$/, '') || '未命名'
        await addSound({
          name,
          filePath: file,
          color: PRESET_COLORS[(sounds.length + i) % PRESET_COLORS.length],
          isOneShot: true,
          midiMessageType: 'NOTE',
        })
      }
    }
  }

  const handleSoundClick = (sound: AtmosphereSound) => {
    playSound(sound.id)
  }

  const handleConnectMidi = async (deviceName: string) => {
    await connectMidi(deviceName)
  }

  const handleDisconnectMidi = async () => {
    await disconnectMidi()
  }

  const handleVolumeChange = async (volume: number) => {
    setMasterVolume(volume)
    try {
      await atmosphereApi.setAtmosphereVolume(volume)
    } catch (e) {
      console.error('Failed to set atmosphere volume:', e)
    }
  }

  return (
    <div className="p-3 flex flex-col h-full">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-medium">气氛组</h3>
          {/* 停止按钮 */}
          <button
            onClick={() => stopSound()}
            className="flex items-center gap-1 px-1.5 py-0.5 bg-red-900/50 hover:bg-red-800/60 text-red-400 hover:text-red-300 rounded text-xs transition-colors"
            title="停止所有音效"
          >
            <Square className="w-3 h-3" />
          </button>
          <button
            onClick={() => setStopMidiLearnMode(!stopMidiLearnMode)}
            className={`text-[10px] px-1 py-0.5 rounded transition-colors ${
              stopMidiLearnMode
                ? 'bg-primary-600 text-white'
                : stopMidiConfig.note !== null
                  ? 'bg-green-900/50 text-green-400'
                  : 'bg-dark-700 text-dark-500 hover:text-white'
            }`}
            title={stopMidiLearnMode ? '等待 MIDI 信号...' : stopMidiConfig.note !== null ? `${stopMidiConfig.messageType}: ${stopMidiConfig.note}` : 'MIDI 学习'}
          >
            {stopMidiLearnMode ? '...' : 'M'}
          </button>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowMidiSettings(!showMidiSettings)}
            className={`text-xs px-2 py-0.5 rounded transition-colors ${
              midiStatus.connected
                ? 'bg-green-900/50 text-green-400'
                : 'bg-dark-700 text-dark-400 hover:text-white'
            }`}
          >
            <Piano className="w-3 h-3 inline mr-1" />
            {midiStatus.connected ? midiStatus.deviceName || 'MIDI' : 'MIDI'}
          </button>
        </div>
      </div>

      {/* MIDI 设置面板 */}
      {showMidiSettings && (
        <div className="mb-3 p-2 bg-dark-800 rounded text-xs">
          <div className="flex items-center justify-between mb-2">
            <span className="text-dark-400">MIDI 设备</span>
            {midiStatus.connected && (
              <button
                onClick={handleDisconnectMidi}
                className="text-red-400 hover:text-red-300"
              >
                断开
              </button>
            )}
          </div>
          {midiDevices.length === 0 ? (
            <p className="text-dark-500">未检测到 MIDI 设备</p>
          ) : (
            <div className="space-y-1">
              {midiDevices.map((device) => (
                <button
                  key={device.id}
                  onClick={() => handleConnectMidi(device.name)}
                  className={`w-full text-left px-2 py-1 rounded transition-colors ${
                    midiStatus.deviceName === device.name
                      ? 'bg-primary-600 text-white'
                      : 'bg-dark-700 hover:bg-dark-600'
                  }`}
                >
                  {device.name}
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {/* 音量控制 */}
      <div className="flex items-center gap-2 mb-2 px-1">
        <Volume2 className="w-3 h-3 text-dark-400" />
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={masterVolume}
          onChange={(e) => handleVolumeChange(parseFloat(e.target.value))}
          className="flex-1 h-1"
        />
        <span className="text-xs text-dark-400 w-8 text-right">
          {Math.round(masterVolume * 100)}%
        </span>
      </div>

      {/* 音效按钮网格 - 添加滚动支持 */}
      <div className="flex-1 overflow-y-auto min-h-0">
        <div className="grid grid-cols-4 gap-2 pb-2">
        {sounds.map((sound) => (
          <div key={sound.id} className="relative group">
            <button
              onClick={() => handleSoundClick(sound)}
              className="w-full p-2 bg-dark-700 hover:bg-dark-600 rounded text-xs font-medium transition-colors"
              style={{ borderLeft: `3px solid ${sound.color || '#666'}` }}
            >
              <span className="truncate block">{sound.name}</span>
              {sound.midiNote !== null && (
                <span className="text-dark-500 text-[10px] block">
                  {sound.midiMessageType}: {sound.midiNote}
                </span>
              )}
            </button>
            <div className="absolute top-0 right-0 opacity-0 group-hover:opacity-100 transition-opacity flex gap-0.5">
              <button
                onClick={(e) => {
                  e.stopPropagation()
                  setEditingSound(sound)
                }}
                className="p-1 bg-dark-600 hover:bg-dark-500 rounded text-dark-400 hover:text-white"
              >
                <Settings className="w-3 h-3" />
              </button>
              <button
                onClick={(e) => {
                  e.stopPropagation()
                  deleteSound(sound.id)
                }}
                className="p-1 bg-dark-600 hover:bg-dark-500 rounded text-dark-400 hover:text-red-400"
              >
                <Trash2 className="w-3 h-3" />
              </button>
            </div>
          </div>
        ))}
        <button
          onClick={handleAddSound}
          className="p-2 bg-dark-700 hover:bg-dark-600 rounded text-xs transition-colors text-dark-400"
        >
          <Plus className="w-4 h-4 mx-auto" />
        </button>
        </div>
      </div>

      {/* 编辑弹窗 */}
      {editingSound && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-dark-800 rounded-lg p-4 w-80">
            <h4 className="font-medium mb-3">编辑音效</h4>
            <div className="space-y-3">
              <div>
                <label className="text-xs text-dark-400 block mb-1">名称</label>
                <input
                  type="text"
                  value={editingSound.name}
                  onChange={(e) =>
                    setEditingSound({ ...editingSound, name: e.target.value })
                  }
                  className="w-full bg-dark-700 rounded px-3 py-2 text-sm border-none outline-none"
                />
              </div>
              <div>
                <label className="text-xs text-dark-400 block mb-1">音量</label>
                <div className="flex items-center gap-2">
                  <Volume2 className="w-4 h-4 text-dark-400" />
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={editingSound.volume}
                    onChange={(e) =>
                      setEditingSound({ ...editingSound, volume: parseFloat(e.target.value) })
                    }
                    className="flex-1"
                  />
                  <span className="text-xs text-dark-400 w-8">
                    {Math.round(editingSound.volume * 100)}%
                  </span>
                </div>
              </div>

              {/* MIDI 设置 */}
              <div className="border-t border-dark-700 pt-3">
                <div className="flex items-center justify-between mb-2">
                  <label className="text-xs text-dark-400">MIDI 映射</label>
                  <button
                    onClick={() => setMidiLearnMode(editingSound.id)}
                    className={`text-xs px-2 py-0.5 rounded ${
                      midiLearnMode === editingSound.id
                        ? 'bg-primary-600 text-white'
                        : 'bg-dark-700 text-dark-400 hover:text-white'
                    }`}
                  >
                    {midiLearnMode === editingSound.id ? '学习中...' : '学习'}
                  </button>
                </div>

                <div className="grid grid-cols-2 gap-2">
                  <div>
                    <label className="text-xs text-dark-400 block mb-1">类型</label>
                    <select
                      value={editingSound.midiMessageType}
                      onChange={(e) =>
                      setEditingSound({
                        ...editingSound,
                        midiMessageType: e.target.value as MidiMessageType,
                      })
                    }
                      className="w-full bg-dark-700 rounded px-2 py-1 text-sm border-none outline-none"
                    >
                      {MIDI_MESSAGE_TYPES.map((type) => (
                        <option key={type.value} value={type.value}>
                          {type.label}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div>
                    <label className="text-xs text-dark-400 block mb-1">
                      {editingSound.midiMessageType === 'NOTE' ? '音符' :
                       editingSound.midiMessageType === 'CC' ? '控制器' : '程序'}
                    </label>
                    <input
                      type="number"
                      min="0"
                      max="127"
                      value={editingSound.midiNote ?? ''}
                      onChange={(e) =>
                        setEditingSound({
                          ...editingSound,
                          midiNote: e.target.value ? parseInt(e.target.value) : null,
                        })
                      }
                      placeholder="未设置"
                      className="w-full bg-dark-700 rounded px-2 py-1 text-sm border-none outline-none"
                    />
                  </div>
                </div>

                <div className="mt-2">
                  <label className="text-xs text-dark-400 block mb-1">MIDI 通道 (0-15)</label>
                  <input
                    type="number"
                    min="0"
                    max="15"
                    value={editingSound.midiChannel}
                    onChange={(e) =>
                      setEditingSound({
                        ...editingSound,
                        midiChannel: parseInt(e.target.value) || 0,
                      })
                    }
                    className="w-full bg-dark-700 rounded px-2 py-1 text-sm border-none outline-none"
                  />
                </div>
              </div>

              <div>
                <label className="text-xs text-dark-400 block mb-1">颜色</label>
                <div className="flex gap-1 flex-wrap">
                  {PRESET_COLORS.map((color) => (
                    <button
                      key={color}
                      onClick={() => setEditingSound({ ...editingSound, color })}
                      className={`w-6 h-6 rounded transition-transform ${
                        editingSound.color === color ? 'scale-110 ring-2 ring-white' : ''
                      }`}
                      style={{ backgroundColor: color }}
                    />
                  ))}
                </div>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="oneShot"
                  checked={editingSound.isOneShot}
                  onChange={(e) =>
                    setEditingSound({ ...editingSound, isOneShot: e.target.checked })
                  }
                  className="rounded"
                />
                <label htmlFor="oneShot" className="text-sm">
                  一次性播放（不循环）
                </label>
              </div>
            </div>
            <div className="flex gap-2 mt-4">
              <button
                onClick={() => {
                  setEditingSound(null)
                  setMidiLearnMode(null)
                }}
                className="flex-1 py-2 bg-dark-700 hover:bg-dark-600 rounded text-sm transition-colors"
              >
                取消
              </button>
              <button
                onClick={async () => {
                  await updateSound({
                    id: editingSound.id,
                    name: editingSound.name,
                    volume: editingSound.volume,
                    midiMessageType: editingSound.midiMessageType,
                    midiNote: editingSound.midiNote ?? undefined,
                    midiChannel: editingSound.midiChannel,
                    isOneShot: editingSound.isOneShot,
                    color: editingSound.color ?? undefined,
                  })
                  setEditingSound(null)
                  setMidiLearnMode(null)
                }}
                className="flex-1 py-2 bg-primary-600 hover:bg-primary-700 rounded text-sm transition-colors"
              >
                保存
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default AtmospherePanel
