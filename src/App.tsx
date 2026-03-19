import { BrowserRouter, Routes, Route } from 'react-router-dom'
import MainLayout from './components/Layout/MainLayout'
import Library from './components/Library/Library'
import Queue from './components/Queue/Queue'
import Settings from './components/Settings/Settings'
import EffectChain from './components/EffectChain/EffectChain'

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<MainLayout />}>
          <Route index element={<Library />} />
          <Route path="queue" element={<Queue />} />
          <Route path="effects" element={<EffectChain />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}

export default App
