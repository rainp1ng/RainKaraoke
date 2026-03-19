function AtmospherePanel() {
  const sounds = [
    { id: 1, name: '掌声', color: '#22c55e' },
    { id: 2, name: '欢呼', color: '#eab308' },
    { id: 3, name: '倒计时', color: '#3b82f6' },
    { id: 4, name: '笑声', color: '#ec4899' },
  ]

  return (
    <div className="p-3">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-medium">气氛组</h3>
        <span className="text-xs text-dark-400">MIDI控制</span>
      </div>

      <div className="grid grid-cols-4 gap-2">
        {sounds.map((sound) => (
          <button
            key={sound.id}
            className="p-2 bg-dark-700 hover:bg-dark-600 rounded text-xs font-medium transition-colors"
            style={{ borderLeft: `3px solid ${sound.color}` }}
          >
            {sound.name}
          </button>
        ))}
        <button className="p-2 bg-dark-700 hover:bg-dark-600 rounded text-xs transition-colors text-dark-400">
          + 添加
        </button>
      </div>
    </div>
  )
}

export default AtmospherePanel
