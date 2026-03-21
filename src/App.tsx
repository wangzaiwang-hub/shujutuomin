import { BrowserRouter, Routes, Route } from "react-router-dom";
import { useEffect } from "react";
import { MainLayout } from "@/components/layout/MainLayout";
import FileProcess from "@/pages/FileProcess";
import RuleConfig from "@/pages/RuleConfig";
import SandboxManager from "@/pages/SandboxManager";
import OperationLog from "@/pages/OperationLog";
import CheersAICloudBrowser from "@/pages/CheersAICloudBrowser";
import { FileManager } from "@/components/file/FileManager";
import { GiteaSettings } from "@/components/settings/GiteaSettings";
import { useLogStore } from "@/store/logStore";
import { tauriCommands } from "@/lib/tauri";

function App() {
  const { initializeDatabase } = useLogStore();

  // 应用启动时初始化数据库和迁移旧数据
  useEffect(() => {
    const init = async () => {
      try {
        // 先尝试迁移旧数据库
        try {
          const migrationResult = await tauriCommands.migrateOldDatabase();
          console.log("Migration result:", migrationResult);
        } catch (migrationError) {
          console.log("No migration needed or migration failed:", migrationError);
        }
        
        // 然后初始化数据库
        await initializeDatabase();
        console.log("Database initialized successfully");
      } catch (error) {
        console.error("Failed to initialize database:", error);
      }
    };
    init();
  }, [initializeDatabase]);

  return (
    <BrowserRouter>
      <Routes>
        <Route element={<MainLayout />}>
          <Route index element={<FileProcess />} />
          <Route path="/files" element={<FileManager />} />
          <Route path="/gitea" element={<GiteaSettings />} />
          <Route path="/rules" element={<RuleConfig />} />
          <Route path="/sandbox" element={<SandboxManager />} />
          <Route path="/log" element={<OperationLog />} />
          <Route path="/cloud" element={<CheersAICloudBrowser />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
