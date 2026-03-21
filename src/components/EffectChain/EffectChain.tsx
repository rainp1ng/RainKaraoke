import { useState, useEffect, useCallback, useRef } from 'react'
import { Plus, Power, Trash2, GripVertical, Save, FolderOpen, Settings, Mic, Headphones, Radio, Circle, Square, Volume2, Music, ChevronUp, ChevronDown, X } from 'lucide-react'
import { EFFECT_TYPES, EffectSlot, DeviceInfo, LiveAudioState } from '@/types'
import { effectApi } from '@/lib/api'
import { listen } from '@tauri-apps/api/event'

type EffectType = 'gain' | 'reverb' | 'chorus' | 'eq' | 'compressor' | 'delay' | 'deesser' | 'exciter' | 'gate' | 'levelmeter'

// 效果器默认参数
const DEFAULT_PARAMS: Record<EffectType, Record<string, any>> = {
  gain: { gainDb: 0 },
  reverb: { roomSize: 50, damping: 30, wetLevel: 30, dryLevel: 70, preDelay: 10, width: 100, roomType: 1 },
  chorus: { rate: 1.5, depth: 50, mix: 30, voices: 4, spread: 50 },
  eq: {
    low: { gain: 0, frequency: 100, q: 0.7 },
    lowMid: { gain: 0, frequency: 500, q: 0.7 },
    highMid: { gain: 0, frequency: 4000, q: 0.7 },
    high: { gain: 0, frequency: 12000, q: 0.7 },
    lowCut: { enabled: false, frequency: 80 },
    highCut: { enabled: false, frequency: 12000 },
  },
  compressor: { threshold: -24, ratio: 4, attack: 10, release: 100, makeupGain: 0 },
  delay: { time: 250, feedback: 30, mix: 20, pingPong: false },
  deesser: { frequency: 6000, threshold: -20, range: 6 },
  exciter: { frequency: 8000, harmonics: 30, mix: 20 },
  gate: { threshold: -50, attack: 1, release: 50, range: 40 },
  levelmeter: {},
}

const EFFECT_COLORS: Record<string, string> = {
  gain: '#f97316',
  reverb: '#3b82f6',
  chorus: '#8b5cf6',
  eq: '#10b981',
  compressor: '#f59e0b',
  delay: '#ec4899',
  deesser: '#ef4444',
  exciter: '#06b6d4',
  gate: '#6b7280',
  levelmeter: '#84cc16',
}

function EffectChain() {
  // 音频设备列表
  const [inputDevices, setInputDevices] = useState<DeviceInfo[]>([])
  const [outputDevices, setOutputDevices] = useState<DeviceInfo[]>([])

  // 效果器槽位
  const [slots, setSlots] = useState<EffectSlot[]>([])
  const [selectedSlot, setSelectedSlot] = useState<number | null>(null)
  const [showAddMenu, setShowAddMenu] = useState(false)
  const [bypassAll, setBypassAll] = useState(false)

  // 拖拽状态
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null)
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null)

  // MIDI 学习状态
  const [midiLearning, setMidiLearning] = useState<number | null>(null) // 正在学习的 slotIndex
  const midiLearningRef = useRef<number | null>(null) // 用于在事件处理器中访问最新值

  // 同步 midiLearning 到 ref
  useEffect(() => {
    midiLearningRef.current = midiLearning
  }, [midiLearning])

  // 音频路由配置
  const [vocalInputDevice, setVocalInputDevice] = useState<string>('')
  const [vocalInputChannel, setVocalInputChannel] = useState(0)
  const [instrumentInputDevice, setInstrumentInputDevice] = useState<string>('')
  const [instrumentInputChannel, setInstrumentInputChannel] = useState(1)
  const [monitorDevice, setMonitorDevice] = useState<string>('')
  const [streamDevice, setStreamDevice] = useState<string>('')
  const [vocalVolume, setVocalVolume] = useState(80)
  const [instrumentVolume, setInstrumentVolume] = useState(80)
  const [monitorVolume, setMonitorVolume] = useState(80)

  // 效果器输入源选择
  const [effectInput, setEffectInput] = useState<'vocal' | 'instrument' | 'none'>('vocal')

  // 实时音频状态
  const [liveState, setLiveState] = useState<LiveAudioState | null>(null)
  const [isRecording, setIsRecording] = useState(false)

  // 选中的设备通道数
  const selectedVocalDevice = inputDevices.find(d => d.name === vocalInputDevice)
  const selectedInstrumentDevice = inputDevices.find(d => d.name === instrumentInputDevice)

  // 加载初始数据
  useEffect(() => {
    loadData()
  }, [])

  // MIDI 事件监听 - 只设置一次，使用 ref 来访问最新状态
  useEffect(() => {
    let unlisten: (() => void) | null = null
    let mounted = true

    const setupMidiListener = async () => {
      try {
        const fn = await listen('midi:event', async (event) => {
          if (!mounted) return

          const midiEvent = event.payload as {
            messageType: 'NOTE' | 'CC' | 'PC'
            channel: number
            data1: number
            data2: number
            isOn: boolean
          }

          console.log('[EffectChain] MIDI event received:', midiEvent)

          // 使用 ref 获取最新的 midiLearning 值
          const currentMidiLearning = midiLearningRef.current

          // 如果正在学习 MIDI
          if (currentMidiLearning !== null && midiEvent.messageType === 'NOTE' && midiEvent.isOn) {
            try {
              console.log('[EffectChain] Setting MIDI for slot:', currentMidiLearning, 'note:', midiEvent.data1, 'channel:', midiEvent.channel)
              await effectApi.setEffectMidi(currentMidiLearning, midiEvent.data1, midiEvent.channel)
              // 重新加载槽位
              const slotsData = await effectApi.getEffectSlots()
              setSlots(slotsData.map(s => ({
                ...s,
                parameters: typeof s.parameters === 'string' ? JSON.parse(s.parameters) : s.parameters
              })))
              setMidiLearning(null)
              console.log('[EffectChain] MIDI learning complete')
            } catch (error) {
              console.error('Failed to set effect MIDI:', error)
            }
            return
          }

          // 处理效果器开关控制 - 使用函数式更新来避免依赖 slots
          if (midiEvent.messageType === 'NOTE' && midiEvent.isOn) {
            setSlots(prevSlots => {
              const slot = prevSlots.find(s =>
                s.midiNote === midiEvent.data1 && s.midiChannel === midiEvent.channel
              )
              if (slot) {
                console.log('[EffectChain] Toggling effect via MIDI:', slot.effectType)
                const newEnabled = !slot.isEnabled
                // 异步调用 API，但不等待
                effectApi.toggleEffect(slot.slotIndex, newEnabled).catch(err =>
                  console.error('Failed to toggle effect:', err)
                )
                return prevSlots.map(s =>
                  s.slotIndex === slot.slotIndex ? { ...s, isEnabled: newEnabled } : s
                )
              }
              return prevSlots
            })
          }
        })
        if (mounted) {
          unlisten = fn
        } else {
          fn()
        }
      } catch (error) {
        console.error('Failed to setup MIDI listener:', error)
      }
    }

    setupMidiListener()

    return () => {
      mounted = false
      if (unlisten) unlisten()
    }
  }, []) // 空依赖数组，只设置一次

  const loadData = async () => {
    try {
      // 加载设备列表
      const inputs = await effectApi.listAudioInputDevices()
      const outputs = await effectApi.listAudioOutputDevices()
      console.log('[EffectChain] Input devices:', inputs)
      console.log('[EffectChain] Output devices:', outputs)
      setInputDevices(inputs)
      setOutputDevices(outputs)

      // 加载效果器配置
      const config = await effectApi.getEffectChainConfig()
      console.log('[EffectChain] Loaded config from DB:', config)
      setVocalInputDevice(config.vocalInputDevice || '')
      setInstrumentInputDevice(config.instrumentInputDevice || '')
      setVocalInputChannel(config.vocalInputChannel ?? 0)
      setInstrumentInputChannel(config.instrumentInputChannel ?? 1)
      setMonitorDevice(config.monitorDeviceId || '')
      setStreamDevice(config.streamDeviceId || '')
      setVocalVolume(Math.round((config.vocalVolume || 0.8) * 100))
      setInstrumentVolume(Math.round((config.instrumentVolume || 0.8) * 100))
      setMonitorVolume(Math.round((config.monitorVolume || 0.8) * 100))
      setEffectInput(config.effectInput || 'vocal')
      setBypassAll(config.bypassAll)

      console.log('[EffectChain] After loading - vocalInputDevice:', config.vocalInputDevice || '')
      console.log('[EffectChain] After loading - monitorDevice:', config.monitorDeviceId || '')

      // 加载效果器槽位
      const slotsData = await effectApi.getEffectSlots()
      setSlots(slotsData.map(s => ({
        ...s,
        parameters: typeof s.parameters === 'string' ? JSON.parse(s.parameters) : s.parameters
      })))

      // 加载实时音频状态 - 确保每次都检查后端状态
      const state = await effectApi.getLiveAudioState()
      console.log('[EffectChain] Live audio state:', state)
      setLiveState(state.isRunning ? state : null)
    } catch (error) {
      console.error('Failed to load effect chain data:', error)
      setLiveState(null)
    }
  }

  // 启动/停止实时音频
  const toggleLiveAudio = async () => {
    try {
      if (liveState?.isRunning) {
        console.log('[Frontend] Stopping live audio...')
        await effectApi.stopLiveAudio()
        setLiveState(null)
        console.log('[Frontend] Live audio stopped')
      } else {
        // 防止重复点击
        if (!vocalInputDevice) {
          alert('请先选择人声输入设备')
          return
        }
        // 检查输出设备 - 空字符串也算未选择
        if (!monitorDevice && !streamDevice) {
          alert('请至少选择一个输出设备（监听或直播）')
          return
        }

        // 先检查是否已经在运行
        const currentState = await effectApi.getLiveAudioState()
        if (currentState.isRunning) {
          console.log('[Frontend] Audio already running, updating state')
          setLiveState(currentState)
          return
        }

        // 启动前先保存配置到数据库
        const configToSave = {
          vocalInputDevice: vocalInputDevice || null,
          instrumentInputDevice: instrumentInputDevice || null,
          vocalInputChannel,
          instrumentInputChannel,
          monitorDeviceId: monitorDevice || null,
          streamDeviceId: streamDevice || null,
          vocalVolume: vocalVolume / 100,
          instrumentVolume: instrumentVolume / 100,
          monitorVolume: monitorVolume / 100,
          effectInput,
        }
        console.log('[EffectChain] Saving config before start:', configToSave)
        await effectApi.saveEffectChainConfig(configToSave)

        const config = {
          vocalInputDevice: vocalInputDevice || null,
          vocalInputChannel,
          instrumentInputDevice: instrumentInputDevice || null,
          instrumentInputChannel,
          monitorOutputDevice: monitorDevice || '',
          streamOutputDevice: streamDevice || null,
          vocalVolume: vocalVolume / 100,
          instrumentVolume: instrumentVolume / 100,
          effectInput,
          monitorVolume: monitorVolume / 100,
          streamVolume: 1.0,
        }
        console.log('[Frontend] Starting live audio with config:', config)

        await effectApi.startLiveAudio(config)
        console.log('[Frontend] startLiveAudio returned')

        const state = await effectApi.getLiveAudioState()
        console.log('[Frontend] Live audio state:', state)
        setLiveState(state)
      }
    } catch (error) {
      console.error('Failed to toggle live audio:', error)
      alert('启动失败: ' + error)
    }
  }

  // 开始/停止录音
  const toggleRecording = async () => {
    try {
      if (isRecording) {
        const result = await effectApi.stopRecording()
        console.log('Recording saved:', result)
        setIsRecording(false)
      } else {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-')
        await effectApi.startRecording(
          `recordings/vocal-${timestamp}.wav`,
          `recordings/instrument-${timestamp}.wav`
        )
        setIsRecording(true)
      }
    } catch (error) {
      console.error('Failed to toggle recording:', error)
    }
  }

  // 保存配置
  const saveConfig = useCallback(async () => {
    try {
      await effectApi.saveEffectChainConfig({
        vocalInputDevice: vocalInputDevice || null,
        instrumentInputDevice: instrumentInputDevice || null,
        monitorDeviceId: monitorDevice || null,
        streamDeviceId: streamDevice || null,
        vocalVolume: vocalVolume / 100,
        instrumentVolume: instrumentVolume / 100,
        monitorVolume: monitorVolume / 100,
        effectInput,
        bypassAll,
      })
    } catch (error) {
      console.error('Failed to save config:', error)
    }
  }, [vocalInputDevice, instrumentInputDevice, monitorDevice, streamDevice, vocalVolume, instrumentVolume, monitorVolume, effectInput, bypassAll])

  // 音量变化处理
  const handleVocalVolumeChange = async (value: number) => {
    setVocalVolume(value)
    if (liveState?.isRunning) {
      await effectApi.setVocalVolume(value / 100)
    }
  }

  const handleInstrumentVolumeChange = async (value: number) => {
    setInstrumentVolume(value)
    if (liveState?.isRunning) {
      await effectApi.setInstrumentVolume(value / 100)
    }
  }

  const handleEffectInputChange = async (value: 'vocal' | 'instrument' | 'none') => {
    setEffectInput(value)
    if (liveState?.isRunning) {
      await effectApi.setEffectInput(value)
    }
  }

  const handleVocalChannelChange = async (ch: number) => {
    setVocalInputChannel(ch)
    if (liveState?.isRunning) {
      await effectApi.setVocalChannel(ch)
    }
  }

  const handleInstrumentChannelChange = async (ch: number) => {
    setInstrumentInputChannel(ch)
    if (liveState?.isRunning) {
      await effectApi.setInstrumentChannel(ch)
    }
  }

  const toggleSlot = async (index: number) => {
    const slot = slots.find(s => s.slotIndex === index)
    if (!slot) return

    const newEnabled = !slot.isEnabled
    setSlots(slots.map(s =>
      s.slotIndex === index ? { ...s, isEnabled: newEnabled } : s
    ))

    try {
      await effectApi.toggleEffect(index, newEnabled)
    } catch (error) {
      console.error('Failed to toggle effect:', error)
    }
  }

  const removeSlot = async (index: number) => {
    try {
      await effectApi.clearEffectSlot(index)
      setSlots(slots.filter(s => s.slotIndex !== index))
      if (selectedSlot === index) setSelectedSlot(null)
    } catch (error) {
      console.error('Failed to remove slot:', error)
    }
  }

  const addEffect = async (type: EffectType) => {
    // 找到下一个可用的槽位索引
    const usedIndices = new Set(slots.map(s => s.slotIndex))
    let newIndex = 0
    while (usedIndices.has(newIndex)) {
      newIndex++
    }

    try {
      await effectApi.setEffectSlot({
        slotIndex: newIndex,
        effectType: type,
        enabled: true,
        parameters: DEFAULT_PARAMS[type],
      })

      // 重新加载槽位
      const slotsData = await effectApi.getEffectSlots()
      setSlots(slotsData.map(s => ({
        ...s,
        parameters: typeof s.parameters === 'string' ? JSON.parse(s.parameters) : s.parameters
      })))
      setShowAddMenu(false)
    } catch (error) {
      console.error('Failed to add effect:', error)
      alert('添加效果器失败: ' + error)
    }
  }

  const updateSlotParameters = async (slotIndex: number, parameters: Record<string, any>) => {
    setSlots(slots.map(s =>
      s.slotIndex === slotIndex ? { ...s, parameters } : s
    ))

    try {
      await effectApi.updateEffectParameters(slotIndex, parameters)
    } catch (error) {
      console.error('Failed to update parameters:', error)
    }
  }

  const getEffectInfo = (type: string) => {
    return EFFECT_TYPES.find(e => e.type === type)
  }

  // 拖拽处理
  const handleDragStart = (e: React.DragEvent, slotIndex: number) => {
    setDraggedIndex(slotIndex)
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', slotIndex.toString())
  }

  const handleDragOver = (e: React.DragEvent, slotIndex: number) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
    setDragOverIndex(slotIndex)
  }

  const handleDragLeave = () => {
    setDragOverIndex(null)
  }

  const handleDrop = async (e: React.DragEvent, targetIndex: number) => {
    e.preventDefault()
    setDragOverIndex(null)

    if (draggedIndex === null || draggedIndex === targetIndex) {
      setDraggedIndex(null)
      return
    }

    try {
      await effectApi.moveEffectSlot(draggedIndex, targetIndex)
      // 重新加载槽位
      const slotsData = await effectApi.getEffectSlots()
      setSlots(slotsData.map(s => ({
        ...s,
        parameters: typeof s.parameters === 'string' ? JSON.parse(s.parameters) : s.parameters
      })))
    } catch (error) {
      console.error('Failed to move effect:', error)
    }

    setDraggedIndex(null)
  }

  const handleDragEnd = () => {
    setDraggedIndex(null)
    setDragOverIndex(null)
  }

  // 生成通道选项
  const generateChannelOptions = (channels: number) => {
    const options = []
    for (let i = 0; i < channels; i++) {
      options.push(<option key={i} value={i}>通道 {i + 1}</option>)
    }
    // 添加"混合所有通道"选项
    options.push(<option key="-1" value={-1}>混合所有通道</option>)
    return options
  }

  return (
    <div className="flex h-full">
      {/* 左侧：效果器链 */}
      <div className="w-64 bg-dark-900 border-r border-dark-700 flex flex-col">
        <div className="p-4 border-b border-dark-700">
          <div className="flex items-center justify-between mb-2">
            <h2 className="font-semibold">人声效果器链</h2>
            <button
              onClick={async () => {
                const newBypass = !bypassAll
                setBypassAll(newBypass)
                await effectApi.setEffectBypass(newBypass)
              }}
              className={`px-2 py-1 text-xs rounded transition-colors ${
                bypassAll ? 'bg-red-600 text-white' : 'bg-dark-700 text-dark-300'
              }`}
            >
              {bypassAll ? 'BYPASS' : 'ON'}
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {slots.sort((a, b) => a.slotIndex - b.slotIndex).map((slot) => {
            const info = getEffectInfo(slot.effectType)
            return (
              <div
                key={slot.id}
                draggable
                onDragStart={(e) => handleDragStart(e, slot.slotIndex)}
                onDragOver={(e) => handleDragOver(e, slot.slotIndex)}
                onDragLeave={handleDragLeave}
                onDrop={(e) => handleDrop(e, slot.slotIndex)}
                onDragEnd={handleDragEnd}
                onClick={() => setSelectedSlot(slot.slotIndex)}
                className={`p-2 rounded cursor-pointer transition-colors ${
                  selectedSlot === slot.slotIndex ? 'bg-dark-700' : 'hover:bg-dark-800'
                } ${!slot.isEnabled ? 'opacity-50' : ''} ${
                  draggedIndex === slot.slotIndex ? 'opacity-30' : ''
                } ${
                  dragOverIndex === slot.slotIndex && draggedIndex !== slot.slotIndex ? 'border-2 border-primary-500' : ''
                }`}
              >
                <div className="flex items-center gap-2">
                  <GripVertical className="w-4 h-4 text-dark-500 cursor-move flex-shrink-0" />
                  <div
                    className="w-1 h-8 rounded flex-shrink-0"
                    style={{ backgroundColor: EFFECT_COLORS[slot.effectType as EffectType] || '#6b7280' }}
                  />
                  <div className="flex-1 min-w-0">
                    <p className="font-medium text-sm truncate">{info?.name || slot.effectType}</p>
                    <p className="text-xs text-dark-400 truncate">
                      {slot.isEnabled ? '已启用' : '已禁用'}
                      {slot.midiNote !== null && (
                        <span className="ml-1 text-primary-400">[MIDI:{slot.midiNote}]</span>
                      )}
                    </p>
                  </div>
                  {/* 上下移动按钮组 */}
                  <div className="flex flex-col gap-0.5 flex-shrink-0">
                    <button
                      onClick={async (e) => {
                        e.stopPropagation()
                        try {
                          await effectApi.moveEffectUp(slot.slotIndex)
                          const slotsData = await effectApi.getEffectSlots()
                          setSlots(slotsData.map(s => ({
                            ...s,
                            parameters: typeof s.parameters === 'string' ? JSON.parse(s.parameters) : s.parameters
                          })))
                        } catch (err) {
                          console.error('Failed to move effect up:', err)
                        }
                      }}
                      className="p-0.5 text-dark-500 hover:text-primary-400 rounded transition-colors leading-none"
                      title="上移"
                    >
                      <ChevronUp className="w-3 h-3" />
                    </button>
                    <button
                      onClick={async (e) => {
                        e.stopPropagation()
                        try {
                          await effectApi.moveEffectDown(slot.slotIndex)
                          const slotsData = await effectApi.getEffectSlots()
                          setSlots(slotsData.map(s => ({
                            ...s,
                            parameters: typeof s.parameters === 'string' ? JSON.parse(s.parameters) : s.parameters
                          })))
                        } catch (err) {
                          console.error('Failed to move effect down:', err)
                        }
                      }}
                      className="p-0.5 text-dark-500 hover:text-primary-400 rounded transition-colors leading-none"
                      title="下移"
                    >
                      <ChevronDown className="w-3 h-3" />
                    </button>
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation()
                      if (midiLearning === slot.slotIndex) {
                        setMidiLearning(null)
                      } else {
                        setMidiLearning(slot.slotIndex)
                      }
                    }}
                    className={`p-1 rounded transition-colors flex-shrink-0 ${
                      midiLearning === slot.slotIndex
                        ? 'bg-primary-600 text-white'
                        : slot.midiNote !== null
                          ? 'text-primary-400'
                          : 'text-dark-500 hover:text-primary-400'
                    }`}
                    title={slot.midiNote !== null ? `MIDI 已绑定 (Note ${slot.midiNote})` : 'MIDI 学习'}
                  >
                    <Music className="w-3.5 h-3.5" />
                  </button>
                  {slot.midiNote !== null && (
                    <button
                      onClick={async (e) => {
                        e.stopPropagation()
                        try {
                          await effectApi.clearEffectMidi(slot.slotIndex)
                          const slotsData = await effectApi.getEffectSlots()
                          setSlots(slotsData.map(s => ({
                            ...s,
                            parameters: typeof s.parameters === 'string' ? JSON.parse(s.parameters) : s.parameters
                          })))
                        } catch (err) {
                          console.error('Failed to clear effect MIDI:', err)
                        }
                      }}
                      className="p-1 text-dark-500 hover:text-red-400 rounded transition-colors flex-shrink-0"
                      title="清除 MIDI 绑定"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  )}
                  <button
                    onClick={(e) => { e.stopPropagation(); toggleSlot(slot.slotIndex) }}
                    className={`p-1 rounded transition-colors flex-shrink-0 ${
                      slot.isEnabled ? 'text-primary-400' : 'text-dark-500'
                    }`}
                  >
                    <Power className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={(e) => { e.stopPropagation(); removeSlot(slot.slotIndex) }}
                    className="p-1 text-dark-500 hover:text-red-400 rounded transition-colors flex-shrink-0"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
                {midiLearning === slot.slotIndex && (
                  <div className="mt-1 text-xs text-primary-400 text-center">
                    按下 MIDI 键来绑定...
                  </div>
                )}
              </div>
            )
          })}

          {slots.length < 8 && (
            <div className="relative">
              <button
                onClick={() => setShowAddMenu(!showAddMenu)}
                className="w-full p-2 border border-dashed border-dark-600 rounded text-dark-400 hover:border-primary-500 hover:text-primary-400 transition-colors flex items-center justify-center gap-1"
              >
                <Plus className="w-4 h-4" />
                <span className="text-sm">添加效果器</span>
              </button>

              {showAddMenu && (
                <div className="absolute left-0 right-0 mt-1 bg-dark-800 border border-dark-600 rounded-lg shadow-xl z-10 py-1 max-h-64 overflow-y-auto">
                  {EFFECT_TYPES.filter(t => !slots.some(s => s.effectType === t.type)).map((effect) => (
                    <button
                      key={effect.type}
                      onClick={() => addEffect(effect.type as EffectType)}
                      className="w-full px-3 py-2 text-left hover:bg-dark-700 transition-colors flex items-center gap-2"
                    >
                      <div
                        className="w-2 h-2 rounded"
                        style={{ backgroundColor: EFFECT_COLORS[effect.type as EffectType] }}
                      />
                      <span className="text-sm">{effect.name}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        <div className="p-2 border-t border-dark-700 space-y-2">
          <button className="w-full px-3 py-2 bg-dark-700 hover:bg-dark-600 rounded text-sm transition-colors flex items-center justify-center gap-2">
            <Save className="w-4 h-4" />
            保存预设
          </button>
          <button className="w-full px-3 py-2 bg-dark-700 hover:bg-dark-600 rounded text-sm transition-colors flex items-center justify-center gap-2">
            <FolderOpen className="w-4 h-4" />
            加载预设
          </button>
        </div>
      </div>

      {/* 右侧：参数编辑 */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* 音频路由配置 */}
        <div className="p-4 border-b border-dark-700">
          <div className="flex items-center justify-between mb-3">
            <h3 className="font-medium flex items-center gap-2">
              <Settings className="w-4 h-4" />
              音频路由
            </h3>
            <div className="flex items-center gap-2">
              <button
                onClick={toggleLiveAudio}
                className={`px-3 py-1.5 rounded text-sm font-medium transition-colors flex items-center gap-2 ${
                  liveState?.isRunning
                    ? 'bg-green-600 hover:bg-green-700 text-white'
                    : 'bg-primary-600 hover:bg-primary-700 text-white'
                }`}
              >
                <Power className="w-4 h-4" />
                {liveState?.isRunning ? '停止' : '启动'}
              </button>

              <button
                onClick={toggleRecording}
                disabled={!liveState?.isRunning}
                className={`px-3 py-1.5 rounded text-sm font-medium transition-colors flex items-center gap-2 ${
                  isRecording
                    ? 'bg-red-600 hover:bg-red-700 text-white animate-pulse'
                    : 'bg-dark-700 hover:bg-dark-600 text-dark-200 disabled:opacity-50 disabled:cursor-not-allowed'
                }`}
              >
                {isRecording ? <Square className="w-4 h-4" /> : <Circle className="w-4 h-4" />}
                {isRecording ? '停止录音' : '录音'}
              </button>
            </div>
          </div>

          {/* 输入设备选择 */}
          <div className="grid grid-cols-2 gap-4 mb-3">
            <div>
              <label className="block text-sm text-dark-400 mb-1">
                <Mic className="w-3 h-3 inline mr-1" />
                人声输入设备
              </label>
              <select
                value={vocalInputDevice}
                onChange={(e) => setVocalInputDevice(e.target.value)}
                className="w-full bg-dark-700 rounded px-2 py-1.5 text-sm mb-1"
              >
                <option value="">选择人声输入设备</option>
                {inputDevices.map(d => (
                  <option key={d.name} value={d.name}>{d.name} {d.isDefault ? '(默认)' : ''} [{d.channels}ch]</option>
                ))}
              </select>
              {selectedVocalDevice && selectedVocalDevice.channels > 1 && (
                <select
                  value={vocalInputChannel}
                  onChange={(e) => handleVocalChannelChange(Number(e.target.value))}
                  className="w-full bg-dark-700 rounded px-2 py-1.5 text-sm"
                >
                  {generateChannelOptions(selectedVocalDevice.channels)}
                </select>
              )}
            </div>
            <div>
              <label className="block text-sm text-dark-400 mb-1">
                <Mic className="w-3 h-3 inline mr-1" />
                乐器输入设备
              </label>
              <select
                value={instrumentInputDevice}
                onChange={(e) => setInstrumentInputDevice(e.target.value)}
                className="w-full bg-dark-700 rounded px-2 py-1.5 text-sm mb-1"
              >
                <option value="">选择乐器输入设备（可选）</option>
                {inputDevices.map(d => (
                  <option key={d.name} value={d.name}>{d.name} {d.isDefault ? '(默认)' : ''} [{d.channels}ch]</option>
                ))}
              </select>
              {selectedInstrumentDevice && selectedInstrumentDevice.channels > 1 && (
                <select
                  value={instrumentInputChannel}
                  onChange={(e) => handleInstrumentChannelChange(Number(e.target.value))}
                  className="w-full bg-dark-700 rounded px-2 py-1.5 text-sm"
                >
                  {generateChannelOptions(selectedInstrumentDevice.channels)}
                </select>
              )}
            </div>
          </div>

          {/* 输出设备选择 */}
          <div className="grid grid-cols-2 gap-4 mb-3">
            <div>
              <label className="block text-sm text-dark-400 mb-1">
                <Headphones className="w-3 h-3 inline mr-1" />
                监听输出
              </label>
              <select
                value={monitorDevice}
                onChange={(e) => setMonitorDevice(e.target.value)}
                className="w-full bg-dark-700 rounded px-2 py-1.5 text-sm"
              >
                <option value="">选择监听输出设备</option>
                {outputDevices.map(d => (
                  <option key={d.name} value={d.name}>{d.name} {d.isDefault ? '(默认)' : ''} [{d.channels}ch]</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-dark-400 mb-1">
                <Radio className="w-3 h-3 inline mr-1" />
                直播输出（可选）
              </label>
              <select
                value={streamDevice}
                onChange={(e) => setStreamDevice(e.target.value)}
                className="w-full bg-dark-700 rounded px-2 py-1.5 text-sm"
              >
                <option value="">选择直播输出设备</option>
                {outputDevices.map(d => (
                  <option key={d.name} value={d.name}>{d.name}</option>
                ))}
              </select>
            </div>
          </div>

          {/* 效果器输入源选择 */}
          <div className="mb-3">
            <label className="block text-sm text-dark-400 mb-1">
              <Volume2 className="w-3 h-3 inline mr-1" />
              效果器输入源
            </label>
            <div className="flex gap-2">
              {[
                { value: 'vocal', label: '人声' },
                { value: 'instrument', label: '乐器' },
                { value: 'none', label: '不处理' },
              ].map(opt => (
                <button
                  key={opt.value}
                  onClick={() => handleEffectInputChange(opt.value as 'vocal' | 'instrument' | 'none')}
                  className={`px-3 py-1.5 rounded text-sm transition-colors ${
                    effectInput === opt.value
                      ? 'bg-primary-600 text-white'
                      : 'bg-dark-700 text-dark-300 hover:bg-dark-600'
                  }`}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          </div>

          {/* 音量控制 */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-dark-400 mb-1">人声音量</label>
              <div className="flex items-center gap-2">
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={vocalVolume}
                  onChange={(e) => handleVocalVolumeChange(Number(e.target.value))}
                  className="flex-1"
                />
                <span className="text-sm text-dark-400 w-8">{vocalVolume}%</span>
              </div>
            </div>
            <div>
              <label className="block text-sm text-dark-400 mb-1">乐器音量</label>
              <div className="flex items-center gap-2">
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={instrumentVolume}
                  onChange={(e) => handleInstrumentVolumeChange(Number(e.target.value))}
                  className="flex-1"
                />
                <span className="text-sm text-dark-400 w-8">{instrumentVolume}%</span>
              </div>
            </div>
          </div>

          <div className="mt-3 flex justify-end">
            <button
              onClick={saveConfig}
              className="px-4 py-1.5 bg-primary-600 hover:bg-primary-700 rounded text-sm font-medium transition-colors"
            >
              保存配置
            </button>
          </div>
        </div>

        {/* 效果器参数编辑 */}
        <div className="flex-1 overflow-y-auto p-4">
          {selectedSlot !== null ? (
            <EffectParameters
              slot={slots.find(s => s.slotIndex === selectedSlot)!}
              onUpdate={(params) => updateSlotParameters(selectedSlot, params)}
            />
          ) : (
            <div className="h-full flex items-center justify-center text-dark-400">
              <div className="text-center">
                <Settings className="w-12 h-12 mx-auto mb-2 opacity-50" />
                <p>选择一个效果器编辑参数</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

// 效果器参数编辑组件
function EffectParameters({ slot, onUpdate }: { slot: EffectSlot; onUpdate: (params: Record<string, any>) => void }) {
  const params = slot.parameters

  const updateParam = (key: string, value: any) => {
    onUpdate({ ...params, [key]: value })
  }

  const renderReverb = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">混响参数</h4>

      {/* 房间类型预设 */}
      <div className="mb-4">
        <label className="block text-sm text-dark-300 mb-2">房间类型</label>
        <div className="flex gap-2">
          {[
            { value: 0, label: '小房间' },
            { value: 1, label: '中等房间' },
            { value: 2, label: '大厅堂' },
            { value: 3, label: '大教堂' },
          ].map(opt => (
            <button
              key={opt.value}
              onClick={() => updateParam('roomType', opt.value)}
              className={`px-3 py-1.5 rounded text-sm transition-colors ${
                params.roomType === opt.value
                  ? 'bg-primary-600 text-white'
                  : 'bg-dark-700 text-dark-300 hover:bg-dark-600'
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>

      <ParameterSlider label="房间大小" value={params.roomSize} min={0} max={100} unit="%" onChange={(v) => updateParam('roomSize', v)} />
      <ParameterSlider label="阻尼" value={params.damping} min={0} max={100} unit="%" onChange={(v) => updateParam('damping', v)} />
      <ParameterSlider label="湿声比例" value={params.wetLevel} min={0} max={100} unit="%" onChange={(v) => updateParam('wetLevel', v)} />
      <ParameterSlider label="干声比例" value={params.dryLevel} min={0} max={100} unit="%" onChange={(v) => updateParam('dryLevel', v)} />
      <ParameterSlider label="预延迟" value={params.preDelay} min={0} max={100} unit="ms" onChange={(v) => updateParam('preDelay', v)} />
      <ParameterSlider label="立体声宽度" value={params.width ?? 100} min={0} max={100} unit="%" onChange={(v) => updateParam('width', v)} />
    </div>
  )

  const renderChorus = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">合唱参数</h4>
      <ParameterSlider label="调制速率" value={params.rate} min={0.1} max={10} step={0.1} unit="Hz" onChange={(v) => updateParam('rate', v)} />
      <ParameterSlider label="调制深度" value={params.depth} min={0} max={100} unit="%" onChange={(v) => updateParam('depth', v)} />
      <ParameterSlider label="混合比例" value={params.mix} min={0} max={100} unit="%" onChange={(v) => updateParam('mix', v)} />
      <ParameterSlider label="声音数量" value={params.voices} min={1} max={8} step={1} unit="" onChange={(v) => updateParam('voices', v)} />
      <ParameterSlider label="声像展开" value={params.spread} min={0} max={100} unit="%" onChange={(v) => updateParam('spread', v)} />
    </div>
  )

  const renderEQ = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">均衡器参数</h4>

      {/* 低切滤波器 */}
      <div className="bg-dark-800 p-3 rounded">
        <div className="flex items-center justify-between mb-2">
          <h5 className="text-sm font-medium">低切 (高通)</h5>
          <label className="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" checked={params.lowCut?.enabled || false} onChange={(e) => updateParam('lowCut', { ...params.lowCut, enabled: e.target.checked })} className="sr-only peer" />
            <div className="w-9 h-5 bg-dark-600 rounded-full peer peer-checked:bg-primary-600 after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
          </label>
        </div>
        {params.lowCut?.enabled && (
          <ParameterSlider label="频率" value={params.lowCut?.frequency || 80} min={20} max={500} unit="Hz" onChange={(v) => updateParam('lowCut', { ...params.lowCut, frequency: v })} />
        )}
      </div>

      {/* 高切滤波器 */}
      <div className="bg-dark-800 p-3 rounded">
        <div className="flex items-center justify-between mb-2">
          <h5 className="text-sm font-medium">高切 (低通)</h5>
          <label className="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" checked={params.highCut?.enabled || false} onChange={(e) => updateParam('highCut', { ...params.highCut, enabled: e.target.checked })} className="sr-only peer" />
            <div className="w-9 h-5 bg-dark-600 rounded-full peer peer-checked:bg-primary-600 after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
          </label>
        </div>
        {params.highCut?.enabled && (
          <ParameterSlider label="频率" value={params.highCut?.frequency || 12000} min={2000} max={20000} unit="Hz" onChange={(v) => updateParam('highCut', { ...params.highCut, frequency: v })} />
        )}
      </div>

      {/* 参数均衡频段 */}
      {['low', 'lowMid', 'highMid', 'high'].map((band) => (
        <div key={band} className="bg-dark-800 p-3 rounded">
          <h5 className="text-sm font-medium mb-2">
            {band === 'low' ? '低频' : band === 'lowMid' ? '低中频' : band === 'highMid' ? '高中频' : '高频'}
          </h5>
          <ParameterSlider label="增益" value={params[band]?.gain || 0} min={-12} max={12} unit="dB" onChange={(v) => updateParam(band, { ...params[band], gain: v })} />
          <ParameterSlider label="频率" value={params[band]?.frequency || 1000} min={20} max={20000} unit="Hz" onChange={(v) => updateParam(band, { ...params[band], frequency: v })} />
        </div>
      ))}
    </div>
  )

  const renderCompressor = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">压缩器参数</h4>
      <ParameterSlider label="阈值" value={params.threshold} min={-60} max={0} unit="dB" onChange={(v) => updateParam('threshold', v)} />
      <ParameterSlider label="压缩比" value={params.ratio} min={1} max={20} unit=":1" onChange={(v) => updateParam('ratio', v)} />
      <ParameterSlider label="启动时间" value={params.attack} min={0.1} max={100} unit="ms" onChange={(v) => updateParam('attack', v)} />
      <ParameterSlider label="释放时间" value={params.release} min={10} max={1000} unit="ms" onChange={(v) => updateParam('release', v)} />
      <ParameterSlider label="补偿增益" value={params.makeupGain} min={0} max={24} unit="dB" onChange={(v) => updateParam('makeupGain', v)} />
    </div>
  )

  const renderDelay = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">延迟参数</h4>
      <ParameterSlider label="延迟时间" value={params.time} min={1} max={1000} unit="ms" onChange={(v) => updateParam('time', v)} />
      <ParameterSlider label="反馈" value={params.feedback} min={0} max={90} unit="%" onChange={(v) => updateParam('feedback', v)} />
      <ParameterSlider label="混合比例" value={params.mix} min={0} max={100} unit="%" onChange={(v) => updateParam('mix', v)} />
      <div className="flex items-center justify-between">
        <span className="text-sm">乒乓延迟</span>
        <label className="relative inline-flex items-center cursor-pointer">
          <input type="checkbox" checked={params.pingPong} onChange={(e) => updateParam('pingPong', e.target.checked)} className="sr-only peer" />
          <div className="w-9 h-5 bg-dark-600 rounded-full peer peer-checked:bg-primary-600 after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
        </label>
      </div>
    </div>
  )

  const renderDeEsser = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">去齿音参数</h4>
      <ParameterSlider label="中心频率" value={params.frequency} min={2000} max={12000} unit="Hz" onChange={(v) => updateParam('frequency', v)} />
      <ParameterSlider label="阈值" value={params.threshold} min={-40} max={0} unit="dB" onChange={(v) => updateParam('threshold', v)} />
      <ParameterSlider label="衰减范围" value={params.range} min={0} max={24} unit="dB" onChange={(v) => updateParam('range', v)} />
    </div>
  )

  const renderExciter = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">激励器参数</h4>
      <ParameterSlider label="中心频率" value={params.frequency} min={2000} max={16000} unit="Hz" onChange={(v) => updateParam('frequency', v)} />
      <ParameterSlider label="谐波量" value={params.harmonics} min={0} max={100} unit="%" onChange={(v) => updateParam('harmonics', v)} />
      <ParameterSlider label="混合比例" value={params.mix} min={0} max={100} unit="%" onChange={(v) => updateParam('mix', v)} />
    </div>
  )

  const renderGate = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">噪声门参数</h4>
      <ParameterSlider label="阈值" value={params.threshold} min={-80} max={-20} unit="dB" onChange={(v) => updateParam('threshold', v)} />
      <ParameterSlider label="启动时间" value={params.attack} min={0.1} max={50} unit="ms" onChange={(v) => updateParam('attack', v)} />
      <ParameterSlider label="释放时间" value={params.release} min={10} max={500} unit="ms" onChange={(v) => updateParam('release', v)} />
      <ParameterSlider label="衰减范围" value={params.range} min={0} max={80} unit="dB" onChange={(v) => updateParam('range', v)} />
    </div>
  )

  const renderGain = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">增益参数</h4>
      <ParameterSlider label="增益" value={params.gainDb || 0} min={-24} max={24} step={0.5} unit="dB" onChange={(v) => updateParam('gainDb', v)} />
    </div>
  )

  const renderLevelMeter = () => (
    <div className="space-y-4">
      <h4 className="font-medium mb-2">电平表</h4>
      <p className="text-sm text-dark-400">
        电平表用于监测音频信号电平。将其放置在效果器链中需要监测的位置。
      </p>
    </div>
  )

  const renderers: Record<string, () => JSX.Element> = {
    gain: renderGain,
    reverb: renderReverb,
    chorus: renderChorus,
    eq: renderEQ,
    compressor: renderCompressor,
    delay: renderDelay,
    deesser: renderDeEsser,
    exciter: renderExciter,
    gate: renderGate,
    levelmeter: renderLevelMeter,
  }

  return (
    <div className="bg-dark-800 rounded-lg p-4">
      {renderers[slot.effectType]?.() || <p>未知效果器类型: {slot.effectType}</p>}
    </div>
  )
}

function ParameterSlider({
  label,
  value,
  min,
  max,
  step = 1,
  unit,
  onChange,
}: {
  label: string
  value: number
  min: number
  max: number
  step?: number
  unit: string
  onChange: (value: number) => void
}) {
  return (
    <div className="flex items-center gap-3">
      <label className="text-sm text-dark-300 w-20">{label}</label>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="flex-1 h-1 bg-dark-600 rounded-full appearance-none cursor-pointer"
      />
      <span className="text-sm text-dark-400 w-16 text-right">
        {value}{unit}
      </span>
    </div>
  )
}

export default EffectChain
