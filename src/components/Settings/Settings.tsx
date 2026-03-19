import { useEffect, useState } from 'react'
import { audioApi, midiApi } from '@/lib/api'
import type { AudioDevice } from '@/types'

function Settings() {
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([])
  const [midiDevices, setMidiDevices] = useState<{ id: string; name: string }[]>([])
  const [midiStatus, setMidiStatus] = useState<{ connected: boolean; deviceName: string | null }>({ connected: false, deviceName: null })
  const [audioConfig, setAudioConfig] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      console.log('[Settings] Loading data...')
      const [devices, config, midiDev, midiSt] = await Promise.all([
        audioApi.getAudioDevices(),
        audioApi.getAudioConfig(),
        midiApi.getMidiDevices(),
        midiApi.getMidiStatus(),
      ])
      console.log('[Settings] Audio devices:', devices)
      console.log('[Settings] MIDI devices:', midiDev)
      console.log('[Settings] MIDI status:', midiSt)
      setAudioDevices(devices)
      setAudioConfig(config)
      setMidiDevices(midiDev)
      setMidiStatus({ connected: midiSt.connected, deviceName: midiSt.deviceName })
    } catch (err) {
      console.error('[Settings] Failed to load:', err)
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  const handleConnectMidi = async (deviceName: string) => {
    console.log('[Settings] Connecting to MIDI device:', deviceName)
    try {
      await midiApi.connectMidiDevice(deviceName)
      const status = await midiApi.getMidiStatus()
      console.log('[Settings] MIDI status after connect:', status)
      setMidiStatus({ connected: status.connected, deviceName: status.deviceName })
    } catch (err) {
      console.error('[Settings] MIDI connect failed:', err)
      setError(`连接 MIDI 失败: ${err}`)
    }
  }

  const handleDisconnectMidi = async () => {
    try {
      await midiApi.disconnectMidiDevice()
      setMidiStatus({ connected: false, deviceName: null })
    } catch (err) {
      setError(`断开 MIDI 失败: ${err}`)
    }
  }

  const handleSaveConfig = async (key: string, value: any) => {
    try {
      await audioApi.saveAudioConfig({ [key]: value })
    } catch (err) {
      setError(`保存配置失败: ${err}`)
    }
  }

  const outputDevices = audioDevices.filter(d => d.type === 'output')
  const inputDevices = audioDevices.filter(d => d.type === 'input')

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-dark-400">加载中...</div>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-y-auto p-6">
      <h1 className="text-2xl font-bold mb-6">设置</h1>

      {error && (
        <div className="mb-4 p-3 bg-red-900/50 text-red-300 rounded-lg text-sm">
          {error}
          <button onClick={() => setError(null)} className="ml-2 text-red-400 hover:text-red-300">×</button>
        </div>
      )}

      <div className="max-w-2xl space-y-6">
        {/* 音频设置 */}
        <section className="bg-dark-800 rounded-lg p-4">
          <h2 className="text-lg font-semibold mb-4">音频设置</h2>

          <div className="space-y-4">
            <div>
              <label className="block text-sm text-dark-300 mb-1">默认输出设备</label>
              <select
                className="w-full bg-dark-700 rounded px-3 py-2 text-sm"
                value={audioConfig?.defaultOutputDevice || ''}
                onChange={(e) => {
                  setAudioConfig({ ...audioConfig, defaultOutputDevice: e.target.value })
                  handleSaveConfig('defaultOutputDevice', e.target.value)
                }}
              >
                <option value="">系统默认</option>
                {outputDevices.map(d => (
                  <option key={d.id} value={d.id}>{d.name} {d.isDefault ? '(默认)' : ''}</option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm text-dark-300 mb-1">主音量: {Math.round((audioConfig?.masterVolume || 0.8) * 100)}%</label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={audioConfig?.masterVolume || 0.8}
                onChange={(e) => {
                  setAudioConfig({ ...audioConfig, masterVolume: parseFloat(e.target.value) })
                }}
                onMouseUp={(e) => handleSaveConfig('masterVolume', parseFloat((e.target as HTMLInputElement).value))}
                className="w-full"
              />
            </div>
          </div>
        </section>

        {/* 过场音乐设置 */}
        <section className="bg-dark-800 rounded-lg p-4">
          <h2 className="text-lg font-semibold mb-4">过场音乐设置</h2>

          <div className="space-y-4">
            <div>
              <label className="block text-sm text-dark-300 mb-1">过场音乐输出设备</label>
              <select
                className="w-full bg-dark-700 rounded px-3 py-2 text-sm"
                value={audioConfig?.interludeOutputDevice || ''}
                onChange={(e) => handleSaveConfig('interludeOutputDevice', e.target.value)}
              >
                <option value="">使用默认设备</option>
                {outputDevices.map(d => (
                  <option key={d.id} value={d.id}>{d.name}</option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm text-dark-300 mb-1">过场音乐音量: {Math.round((audioConfig?.interludeVolume || 0.3) * 100)}%</label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={audioConfig?.interludeVolume || 0.3}
                onChange={(e) => setAudioConfig({ ...audioConfig, interludeVolume: parseFloat(e.target.value) })}
                onMouseUp={(e) => handleSaveConfig('interludeVolume', parseFloat((e.target as HTMLInputElement).value))}
                className="w-full"
              />
            </div>
          </div>
        </section>

        {/* Ducking 设置 */}
        <section className="bg-dark-800 rounded-lg p-4">
          <h2 className="text-lg font-semibold mb-4">过场音乐 Ducking</h2>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm">启用语音检测降低音量</span>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={audioConfig?.duckingEnabled ?? true}
                  onChange={(e) => {
                    setAudioConfig({ ...audioConfig, duckingEnabled: e.target.checked })
                    handleSaveConfig('duckingEnabled', e.target.checked)
                  }}
                  className="sr-only peer"
                />
                <div className="w-9 h-5 bg-dark-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary-600"></div>
              </label>
            </div>

            <div>
              <label className="block text-sm text-dark-300 mb-1">触发阈值: {Math.round((audioConfig?.duckingThreshold || 0.1) * 100)}%</label>
              <input
                type="range"
                min="0"
                max="0.5"
                step="0.01"
                value={audioConfig?.duckingThreshold || 0.1}
                onChange={(e) => setAudioConfig({ ...audioConfig, duckingThreshold: parseFloat(e.target.value) })}
                onMouseUp={(e) => handleSaveConfig('duckingThreshold', parseFloat((e.target as HTMLInputElement).value))}
                className="w-full"
              />
            </div>

            <div>
              <label className="block text-sm text-dark-300 mb-1">降低比例: {Math.round((audioConfig?.duckingRatio || 0.2) * 100)}%</label>
              <input
                type="range"
                min="0"
                max="0.5"
                step="0.01"
                value={audioConfig?.duckingRatio || 0.2}
                onChange={(e) => setAudioConfig({ ...audioConfig, duckingRatio: parseFloat(e.target.value) })}
                onMouseUp={(e) => handleSaveConfig('duckingRatio', parseFloat((e.target as HTMLInputElement).value))}
                className="w-full"
              />
            </div>
          </div>
        </section>

        {/* MIDI 设置 */}
        <section className="bg-dark-800 rounded-lg p-4">
          <h2 className="text-lg font-semibold mb-4">MIDI 设备</h2>

          <div className="space-y-4">
            <div>
              <label className="block text-sm text-dark-300 mb-1">MIDI 输入设备</label>
              <div className="flex gap-2">
                <select
                  className="flex-1 bg-dark-700 rounded px-3 py-2 text-sm"
                  value={midiStatus.connected ? 'connected' : ''}
                  onChange={(e) => {
                    const selectedDevice = e.target.value
                    if (selectedDevice && selectedDevice !== 'connected') {
                      handleConnectMidi(selectedDevice)
                    }
                  }}
                >
                  <option value="">{midiStatus.connected ? `已连接: ${midiStatus.deviceName}` : '选择 MIDI 设备'}</option>
                  {midiDevices.filter(d => d.name !== midiStatus.deviceName).map(d => (
                    <option key={d.id} value={d.name}>{d.name}</option>
                  ))}
                </select>
                {midiStatus.connected && (
                  <button
                    onClick={handleDisconnectMidi}
                    className="px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm transition-colors"
                  >
                    断开
                  </button>
                )}
              </div>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm">MIDI 状态</span>
              <span className={`text-sm ${midiStatus.connected ? 'text-green-400' : 'text-dark-400'}`}>
                {midiStatus.connected ? `已连接: ${midiStatus.deviceName}` : '未连接'}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm">启用 MIDI 控制</span>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={audioConfig?.midiEnabled ?? true}
                  onChange={(e) => {
                    setAudioConfig({ ...audioConfig, midiEnabled: e.target.checked })
                    handleSaveConfig('midiEnabled', e.target.checked)
                  }}
                  className="sr-only peer"
                />
                <div className="w-9 h-5 bg-dark-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary-600"></div>
              </label>
            </div>

            {midiDevices.length === 0 && (
              <p className="text-sm text-dark-400">未检测到 MIDI 设备，请确保设备已连接</p>
            )}
          </div>
        </section>

        {/* 音频设备列表 */}
        <section className="bg-dark-800 rounded-lg p-4">
          <h2 className="text-lg font-semibold mb-4">音频设备列表</h2>

          <div className="space-y-3">
            <div>
              <h3 className="text-sm font-medium text-dark-300 mb-2">输出设备</h3>
              <div className="space-y-1">
                {outputDevices.map(d => (
                  <div key={d.id} className="flex items-center justify-between text-sm py-1 px-2 bg-dark-700/50 rounded">
                    <span>{d.name}</span>
                    <span className="text-dark-400">{d.channels} 声道 {d.isDefault && <span className="text-primary-400">(默认)</span>}</span>
                  </div>
                ))}
              </div>
            </div>

            <div>
              <h3 className="text-sm font-medium text-dark-300 mb-2">输入设备</h3>
              <div className="space-y-1">
                {inputDevices.map(d => (
                  <div key={d.id} className="flex items-center justify-between text-sm py-1 px-2 bg-dark-700/50 rounded">
                    <span>{d.name}</span>
                    <span className="text-dark-400">{d.channels} 声道 {d.isDefault && <span className="text-primary-400">(默认)</span>}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}

export default Settings
