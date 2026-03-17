import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";

export function MainLayout() {
  return (
    <TooltipProvider>
      <div className="flex h-screen w-screen overflow-hidden bg-gray-50">
        <Sidebar />
        <main className="flex-1 overflow-auto bg-white">
          <Outlet />
        </main>
      </div>
    </TooltipProvider>
  );
}
