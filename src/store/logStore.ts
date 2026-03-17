import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { LogEntry, LogLevel } from "@/types/log";
import { tauriCommands } from "@/lib/tauri";

interface LogStore {
  entries: LogEntry[];
  loading: boolean;
  initialized: boolean;
  currentPage: number;
  totalPages: number;
  pageSize: number;
  totalCount: number;
  addLog: (level: LogLevel, message: string, details?: string, filePath?: string, operationType?: string) => Promise<void>;
  loadLogs: (page?: number, levelFilter?: string) => Promise<void>;
  clearLogs: () => Promise<void>;
  initializeDatabase: () => Promise<void>;
  setPage: (page: number) => void;
  getTotalCount: (levelFilter?: string) => Promise<number>;
}

export const useLogStore = create<LogStore>()(
  persist(
    (set, get) => ({
      entries: [],
      loading: false,
      initialized: false,
      currentPage: 1,
      totalPages: 0,
      pageSize: 10,
      totalCount: 0,

      initializeDatabase: async () => {
        try {
          await tauriCommands.initializeDatabase();
          set({ initialized: true });
          // 初始化后加载第一页日志
          await get().loadLogs(1);
        } catch (error) {
          console.error("Failed to initialize database:", error);
        }
      },

      addLog: async (level, message, details, filePath, operationType) => {
        try {
          // 添加到数据库
          await tauriCommands.addLogEntry(level, message, details, filePath, operationType);
          
          // 如果当前在第一页，重新加载以显示新日志
          const { currentPage } = get();
          if (currentPage === 1) {
            await get().loadLogs(1);
          }
        } catch (error) {
          console.error("Failed to add log:", error);
          
          // 如果数据库失败，至少添加到内存中
          const entry: LogEntry = {
            id: Date.now().toString(),
            timestamp: Date.now(),
            level,
            message,
            details,
            filePath,
            operationType,
          };
          
          set((state) => ({
            entries: [entry, ...state.entries].slice(0, state.pageSize),
          }));
        }
      },

      getTotalCount: async (levelFilter?: string) => {
        try {
          return await tauriCommands.getLogsCount(levelFilter);
        } catch (error) {
          console.error("Failed to get total count:", error);
          return 0;
        }
      },

      loadLogs: async (page = 1, levelFilter) => {
        if (!get().initialized) {
          await get().initializeDatabase();
        }
        
        set({ loading: true });
        try {
          const { pageSize } = get();
          const offset = (page - 1) * pageSize;
          
          // 获取总数
          const totalCount = await get().getTotalCount(levelFilter);
          const totalPages = Math.ceil(totalCount / pageSize);
          
          // 获取当前页数据
          const logs = await tauriCommands.getLogs(pageSize, offset, levelFilter);
          
          // 转换时间戳格式
          const processedLogs = logs.map(log => ({
            ...log,
            timestamp: typeof log.timestamp === 'string' 
              ? new Date(log.timestamp).getTime() 
              : log.timestamp,
            filePath: log.filePath || undefined,
            operationType: log.operationType || undefined,
          }));
          
          set({ 
            entries: processedLogs, 
            loading: false,
            currentPage: page,
            totalPages,
            totalCount
          });
        } catch (error) {
          console.error("Failed to load logs:", error);
          set({ loading: false });
        }
      },

      setPage: (page: number) => {
        set({ currentPage: page });
      },

      clearLogs: async () => {
        try {
          await tauriCommands.clearAllLogs();
          set({ 
            entries: [], 
            currentPage: 1, 
            totalPages: 0, 
            totalCount: 0 
          });
        } catch (error) {
          console.error("Failed to clear logs:", error);
          // 如果数据库清除失败，至少清除内存中的
          set({ 
            entries: [], 
            currentPage: 1, 
            totalPages: 0, 
            totalCount: 0 
          });
        }
      },
    }),
    {
      name: "log-store",
      // 只持久化初始化状态和当前页码
      partialize: (state) => ({ 
        initialized: state.initialized,
        currentPage: state.currentPage 
      }),
    }
  )
);
