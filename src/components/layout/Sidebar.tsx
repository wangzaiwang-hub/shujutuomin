import { NavLink } from "react-router-dom";
import {
  Shield,
  FileText,
  Settings2,
  Lock,
  ClipboardList,
  ChevronLeft,
  ChevronRight,
  Cloud,
  FolderOpen,
  Upload,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store/appStore";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

const navItems = [
  { to: "/", icon: FileText, label: "文件脱敏", description: "处理和脱敏文件" },
  { to: "/files", icon: FolderOpen, label: "文件管理", description: "管理脱敏后的文件" },
  { to: "/gitea", icon: Upload, label: "Gitea 设置", description: "配置 Gitea 上传" },
  { to: "/rules", icon: Settings2, label: "规则配置" },
  { to: "/sandbox", icon: Lock, label: "沙箱管理" },
  { to: "/log", icon: ClipboardList, label: "操作日志" },
  { to: "/cloud", icon: Cloud, label: "CheersAI 云端", description: "访问云端AI服务" },
];



export function Sidebar() {
  const { sidebarCollapsed, toggleSidebar } = useAppStore();

  return (
    <aside
      className={cn(
        "flex flex-col h-full bg-slate-900 text-white transition-all duration-200 shrink-0",
        sidebarCollapsed ? "w-16" : "w-64"
      )}
    >
      {/* Logo */}
      <div className="flex items-center gap-3 px-4 py-6 border-b border-slate-800/50">
        <div className="relative">
          <div className="flex items-center justify-center w-10 h-10 bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl shadow-lg shadow-blue-500/25 shrink-0 overflow-hidden">
            <img 
              src="/logo.jpg" 
              alt="CheersAI Vault Logo" 
              className="w-full h-full object-cover rounded-xl"
              onError={(e) => {
                // 如果logo加载失败，显示默认图标
                const target = e.target as HTMLImageElement;
                target.style.display = 'none';
                target.nextElementSibling?.classList.remove('hidden');
              }}
            />
            <Shield className="w-5 h-5 text-white hidden" />
          </div>
          {/* 小的状态指示器 */}
          <div className="absolute -top-1 -right-1 w-3 h-3 bg-green-400 rounded-full border-2 border-slate-900 animate-pulse" />
        </div>
        {!sidebarCollapsed && (
          <div className="flex flex-col">
            <span className="font-bold text-lg text-white tracking-wide">CheersAI Vault</span>
            <span className="text-xs text-slate-400 font-medium">数据脱敏工具</span>
          </div>
        )}
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 py-2 space-y-1 overflow-y-auto">
        {navItems.map(({ to, icon: Icon, label, description }) => (
          <Tooltip key={to} delayDuration={0}>
            <TooltipTrigger asChild>
              <NavLink
                to={to}
                end={to === "/"}
                className={({ isActive }) =>
                  cn(
                    "group flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-normal transition-all duration-200",
                    isActive
                      ? "bg-blue-600 text-white"
                      : "bg-slate-800/50 text-slate-300 hover:text-white hover:bg-slate-700/70"
                  )
                }
              >
                <>
                  {/* 图标 */}
                  <Icon className="w-5 h-5 shrink-0" />
                  
                  {/* 文字标签 */}
                  {!sidebarCollapsed && (
                    <span>{label}</span>
                  )}
                </>
              </NavLink>
            </TooltipTrigger>
            {sidebarCollapsed && (
              <TooltipContent 
                side="right" 
                className="bg-slate-800 text-white border-slate-700 shadow-xl"
                sideOffset={10}
              >
                <div className="flex flex-col">
                  <span className="font-medium">{label}</span>
                  {description && (
                    <span className="text-xs text-slate-400">{description}</span>
                  )}
                </div>
              </TooltipContent>
            )}
          </Tooltip>
        ))}
      </nav>

      {/* Footer */}
      <div className="px-3 py-4 border-t border-slate-800/50">
        {!sidebarCollapsed && (
          <div className="px-4 py-3 mb-4 bg-gradient-to-r from-slate-800 to-slate-700 rounded-xl border border-slate-700/50">
            <div className="flex items-center gap-2 mb-1">
              <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
              <div className="text-xs text-slate-400">系统状态</div>
            </div>
            <div className="text-sm text-white font-medium">运行正常</div>
            <div className="text-xs text-slate-400 mt-1">版本 v0.1.0</div>
          </div>
        )}
        
        {/* Collapse toggle */}
        <button
          onClick={toggleSidebar}
          className={cn(
            "flex items-center justify-center w-full h-12 rounded-xl transition-all duration-200",
            "bg-slate-800/50 hover:bg-slate-700 text-slate-400 hover:text-white",
            "border border-slate-700/50 hover:border-slate-600",
            "hover:shadow-lg hover:shadow-slate-900/20",
            "active:scale-95"
          )}
        >
          <div className="flex items-center gap-2">
            {sidebarCollapsed ? (
              <ChevronRight className="w-4 h-4" />
            ) : (
              <>
                <ChevronLeft className="w-4 h-4" />
                <span className="text-xs font-medium">收起</span>
              </>
            )}
          </div>
        </button>
      </div>
    </aside>
  );
}
