import { Outlet } from 'react-router';
import { Sidebar } from './Sidebar';
import { useSidebarStore } from '@/stores/sidebarStore';

export function AppLayout() {
  const isCollapsed = useSidebarStore((state) => state.isCollapsed);

  return (
    <div className="min-h-screen bg-background">
      <Sidebar />
      <main
        className={`transition-all duration-300 ease-out ${
          isCollapsed ? 'ml-16' : 'ml-60'
        }`}
      >
        <div className="min-h-screen">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
