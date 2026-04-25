import { Button } from '@/components/ui/button'

export function Sidebar() {
  return (
    <aside className="w-[260px] flex h-full flex-col bg-black border-r border-white/10">
      <div className="border-b border-white/10 px-3 py-4">
        <h1 className="text-sm font-semibold tracking-tight text-white/80">
          Claude UI
        </h1>
      </div>

      <div className="border-b border-white/10 px-3 py-2">
        <Button
          className="w-full border-white/20 text-sm py-2 px-3 hover:bg-white/5 hover:border-white/30 active:bg-white/10"
          variant="outline"
        >
          New chat
        </Button>
      </div>

      <div className="flex-1 px-2 py-3 overflow-y-auto">
        <div className="px-3 py-4">
          <p className="text-xs text-white/30 px-2 text-center">
            No conversations yet
          </p>
        </div>
      </div>
    </aside>
  );
}