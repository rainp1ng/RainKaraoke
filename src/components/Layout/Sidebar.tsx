import { NavLink } from 'react-router-dom'
import { Music, ListMusic, Settings, Mic2 } from 'lucide-react'

function Sidebar() {
  const navItems = [
    { to: '/', icon: Music, label: '媒体库' },
    { to: '/queue', icon: ListMusic, label: '播放队列' },
    { to: '/effects', icon: Mic2, label: '效果器链' },
    { to: '/settings', icon: Settings, label: '设置' },
  ]

  return (
    <nav className="w-16 bg-dark-900 border-r border-dark-700 flex flex-col items-center py-4">
      {/* Logo */}
      <div className="w-10 h-10 bg-primary-600 rounded-lg flex items-center justify-center mb-6">
        <Music className="w-6 h-6" />
      </div>

      {/* 导航项 */}
      <div className="flex-1 flex flex-col gap-2">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              `w-10 h-10 rounded-lg flex items-center justify-center transition-colors ${
                isActive
                  ? 'bg-primary-600 text-white'
                  : 'text-dark-400 hover:bg-dark-700 hover:text-white'
              }`
            }
            title={item.label}
          >
            <item.icon className="w-5 h-5" />
          </NavLink>
        ))}
      </div>
    </nav>
  )
}

export default Sidebar
