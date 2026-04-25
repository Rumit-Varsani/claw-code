import { Sidebar } from '@/components/layout/sidebar'
import { ChatArea } from '@/components/layout/chat-area'
import { StatusBar } from '@/components/layout/status-bar'

export function Layout() {
  return (
    <div className="flex h-screen flex-row bg-black">
      <Sidebar />
      <ChatArea />
      <StatusBar />
    </div>
  )
}