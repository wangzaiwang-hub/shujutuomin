import { BrowserRouter, Routes, Route } from "react-router-dom";
import { useEffect } from "react";
import { MainLayout } from "@/components/layout/MainLayout";
import FileProcess from "@/pages/FileProcess";
import RuleConfig from "@/pages/RuleConfig";
import SandboxManager from "@/pages/SandboxManager";
import OperationLog from "@/pages/OperationLog";
import { useLogStore } from "@/store/logStore";

function App() {
  const { initializeDatabase } = useLogStore();

  // 应用启动时初始化数据库
  useEffect(() => {
    const init = async () => {
      try {
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
          <Route path="/rules" element={<RuleConfig />} />
          <Route path="/sandbox" element={<SandboxManager />} />
          <Route path="/log" element={<OperationLog />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
