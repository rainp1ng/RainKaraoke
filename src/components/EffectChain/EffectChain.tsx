import { useState } from 'react'
import { Plus, Power, Trash2, GripVertical, Save, FolderOpen, Settings, Mic, Headphones, Radio } from 'lucide-react'
import { EFFECT_TYPES, EffectType, EffectSlot } from '@/types'

// 效果器默认参数
const DEFAULT_PARAMS: Record<EffectType, Record<string, any>> = {
  reverb: { roomSize: 50, damping: 30, wetLevel: 30, dryLevel: 70, preDelay: 10 },
  chorus: { rate: 1.5, depth: 50, mix: 30, voices: 4, spread: 50 },
  eq: { low: { gain: 0, frequency: 100 }, lowMid: { gain: 0, frequency: 500 }, highMid: { gain: 0, frequency: 4000 }, high: { gain: 0, frequency: 12000 } },
  compressor: { threshold: -24, ratio: 4, attack: 10, release: 100, makeupGain: 0 },
  delay: { time: 250, feedback: 30, mix: 20, pingPong: false },
  deesser: { frequency: 6000, threshold: -20, range: 6 },
  exciter: { frequency: 8000, harmonics: 30, mix: 20 },
  gate: { threshold: -50, attack: 1, release: 50, range: 40 },
}

const EFFECT_COLORS: Record<EffectType, string> = {
  reverb: '#3b82f6',
  chorus: '#8b5cf6',
  eq: '#10b981',
  compressor: '#f59e0b',
  delay: '#ec4899',
  deesser: '#ef4444',
  exciter: '#06b6d4',
  gate: '#6b7280',
}

function EffectChain() {
  const [slots, setSlots] = useState<EffectSlot[]>([
    { id: 1, slotIndex: 0, effectType: 'reverb', isEnabled: true, parameters: DEFAULT_PARAMS.reverb },
    { id: 2, slotIndex: 1, effectType: 'eq', isEnabled: true, parameters: DEFAULT_PARAMS.eq },
  ])
  const [selectedSlot, setSelectedSlot] = useState<number | null>(0)
  const [showAddMenu, setShowAddMenu] = useState(false)
  const [bypassAll, setBypassAll] = useState(false)

  // 输入/输出配置
  const [inputDevice, setInputDevice] = useState('default')
  const [monitorDevice, setMonitorDevice] = useState('default')
  const [streamDevice, setStreamDevice] = useState('vb-cable')
  const [inputVolume, setInputVolume] = useState(80)
  const [monitorVolume, setMonitorVolume] = useState(80)

  const toggleSlot = (index: number) => {
    setSlots(slots.map(s =>
      s.slotIndex === index ? { ...s, isEnabled: !s.isEnabled } : s
    ))
  }

  const removeSlot = (index: number) => {
    setSlots(slots.filter(s => s.slotIndex !== index))
    if (selectedSlot === index) setSelectedSlot(null)
  }

  const addEffect = (type: EffectType) => {
    const newSlot: EffectSlot = {
      id: Date.now(),
      slotIndex: slots.length,
      effectType: type,
      isEnabled: true,
      parameters: DEFAULT_PARAMS[type],
    }
    setSlots([...slots, newSlot])
    setShowAddMenu(false)
  }

  const getEffectInfo = (type: EffectType) => {
    return EFFECT_TYPES.find(e => e.type === type)
  }

  return (
    <div className="flex h-full">
      {/* 左侧：效果器链 */}
      <div className="w-64 bg-dark-900 border-r border-dark-700 flex flex-col">
        {/* 标题 */}
        <div className="p-4 border-b border-dark-700">
          <div className="flex items-center justify-between mb-2">
            <h2 className="font-semibold">人声效果器链</h2>
            <button
              onClick={() => setBypassAll(!bypassAll)}
              className={`px-2 py-1 text-xs rounded transition-colors ${
                bypassAll ? 'bg-red-600 text-white' : 'bg-dark-700 text-dark-300'
              }`}
            >
              {bypassAll ? 'BYPASS' : 'ON'}
            </button>
          </div>
        </div>

        {/* 效果器槽位 */}
        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {slots.map((slot) => {
            const info = getEffectInfo(slot.effectType)
            return (
              <div
                key={slot.id}
                onClick={() => setSelectedSlot(slot.slotIndex)}
                className={`p-2 rounded cursor-pointer transition-colors ${
                  selectedSlot === slot.slotIndex ? 'bg-dark-700' : 'hover:bg-dark-800'
                } ${!slot.isEnabled ? 'opacity-50' : ''}`}
              >
                <div className="flex items-center gap-2">
                  <GripVertical className="w-4 h-4 text-dark-500 cursor-move" />
                  <div
                    className="w-1 h-8 rounded"
                    style={{ backgroundColor: EFFECT_COLORS[slot.effectType] }}
                  />
                  <div className="flex-1 min-w-0">
                    <p className="font-medium text-sm truncate">{info?.name}</p>
                    <p className="text-xs text-dark-400">
                      {slot.isEnabled ? '已启用' : '已禁用'}
                    </p>
                  </div>
                  <button
                    onClick={(e) => { e.stopPropagation(); toggleSlot(slot.slotIndex) }}
                    className={`p-1 rounded transition-colors ${
                      slot.isEnabled ? 'text-primary-400' : 'text-dark-500'
                    }`}
                  >
                    <Power className="w-4 h-4" />
                  </button>
                  <button
                    onClick={(e) => { e.stopPropagation(); removeSlot(slot.slotIndex) }}
                    className="p-1 text-dark-500 hover:text-red-400 rounded transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            )
          })}

          {/* 添加效果器按钮 */}
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
                <div className="absolute left-0 right-0 mt-1 bg-dark-800 border border-dark-600 rounded-lg shadow-xl z-10 py-1">
                  {EFFECT_TYPES.filter(t => !slots.some(s => s.effectType === t.type)).map((effect) => (
                    <button
                      key={effect.type}
                      onClick={() => addEffect(effect.type)}
                      className="w-full px-3 py-2 text-left hover:bg-dark-700 transition-colors flex items-center gap-2"
                    >
                      <div
                        className="w-2 h-2 rounded"
                        style={{ backgroundColor: EFFECT_COLORS[effect.type] }}
                      />
                      <span className="text-sm">{effect.name}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {/* 预设管理 */}
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
          <h3 className="font-medium mb-3 flex items-center gap-2">
            <Settings className="w-4 h-4" />
            音频路由
          </h3>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="block text-sm text-dark-400 mb-1">
                <Mic className="w-3 h-3 inline mr-1" />
                输入设备
              </label>
              <select
                value={inputDevice}
                onChange={(e) => setInputDevice(e.target.value)}
                className="w-full bg-dark-700 rounded px-2 py-1.5 text-sm"
              >
                <option value="default">系统默认麦克风</option>
                <option value="usb-mic">USB 麦克风</option>
                <option value="audio-interface">声卡输入</option>
              </select>
            </div>
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
                <option value="default">系统默认扬声器</option>
                <option value="headphones">耳机</option>
              </select>
            </div>
            <div>
              <label className="block text-sm text-dark-400 mb-1">
                <Radio className="w-3 h-3 inline mr-1" />
                直播输出
              </label>
              <select
                value={streamDevice}
                onChange={(e) => setStreamDevice(e.target.value)}
                className="w-full bg-dark-700 rounded px-2 py-1.5 text-sm"
              >
                <option value="vb-cable">VB-Cable</option>
                <option value="blackhole">BlackHole</option>
                <option value="loopback">Loopback</option>
              </select>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4 mt-3">
            <div>
              <label className="block text-sm text-dark-400 mb-1">输入增益</label>
              <div className="flex items-center gap-2">
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={inputVolume}
                  onChange={(e) => setInputVolume(Number(e.target.value))}
                  className="flex-1"
                />
                <span className="text-sm text-dark-400 w-8">{inputVolume}%</span>
              </div>
            </div>
            <div>
              <label className="block text-sm text-dark-400 mb-1">监听音量</label>
              <div className="flex items-center gap-2">
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={monitorVolume}
                  onChange={(e) => setMonitorVolume(Number(e.target.value))}
                  className="flex-1"
                />
                <span className="text-sm text-dark-400 w-8">{monitorVolume}%</span>
              </div>
            </div>
          </div>
        </div>

        {/* 效果器参数编辑 */}
        <div className="flex-1 overflow-y-auto p-4">
          {selectedSlot !== null ? (
            <EffectParameters
              slot={slots.find(s => s.slotIndex === selectedSlot)!}
              onUpdate={(params) => {
                setSlots(slots.map(s =>
                  s.slotIndex === selectedSlot ? { ...s, parameters: params } : s
                ))
              }}
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
      <ParameterSlider label="房间大小" value={params.roomSize} min={0} max={100} unit="%" onChange={(v) => updateParam('roomSize', v)} />
      <ParameterSlider label="阻尼" value={params.damping} min={0} max={100} unit="%" onChange={(v) => updateParam('damping', v)} />
      <ParameterSlider label="湿声比例" value={params.wetLevel} min={0} max={100} unit="%" onChange={(v) => updateParam('wetLevel', v)} />
      <ParameterSlider label="干声比例" value={params.dryLevel} min={0} max={100} unit="%" onChange={(v) => updateParam('dryLevel', v)} />
      <ParameterSlider label="预延迟" value={params.preDelay} min={0} max={100} unit="ms" onChange={(v) => updateParam('preDelay', v)} />
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

  const renderers: Record<EffectType, () => JSX.Element> = {
    reverb: renderReverb,
    chorus: renderChorus,
    eq: renderEQ,
    compressor: renderCompressor,
    delay: renderDelay,
    deesser: renderDeEsser,
    exciter: renderExciter,
    gate: renderGate,
  }

  return (
    <div className="bg-dark-800 rounded-lg p-4">
      {renderers[slot.effectType]?.() || <p>未知效果器类型</p>}
    </div>
  )
}

// 参数滑块组件
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
