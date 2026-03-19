import { Play, Pause, Volume2 } from 'lucide-react'

function InterludePanel() {
  return (
    <div className="p-3">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-medium">过场音乐</h3>
        <div className="flex items-center gap-2">
          <span className="text-xs text-dark-400">Ducking: ON</span>
        </div>
      </div>

      <div className="flex items-center gap-3">
        <button className="w-8 h-8 rounded-full bg-dark-700 hover:bg-dark-600 flex items-center justify-center transition-colors">
          <Play className="w-4 h-4" />
        </button>

        <div className="flex-1">
          <p className="text-sm text-dark-300 truncate">未播放</p>
        </div>

        <div className="flex items-center gap-2">
          <Volume2 className="w-4 h-4 text-dark-400" />
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            defaultValue="0.3"
            className="w-20 h-1 bg-dark-700 rounded-full appearance-none cursor-pointer"
          />
        </div>
      </div>
    </div>
  )
}

export default InterludePanel
