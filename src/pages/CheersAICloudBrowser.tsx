import { useEffect, useRef } from "react";
import { useLocation } from "react-router-dom";
import { tauriCommands } from "@/lib/tauri";
import { CLOUD_APP_URL } from "@/lib/cloud";

export default function CheersAICloudBrowser() {
  const cloudUrl = CLOUD_APP_URL;
  const hasNavigated = useRef(false);
  const location = useLocation();

  useEffect(() => {
    // 检查是否是从返回按钮导航过来的
    const isReturning = location.state?.returning === true;
    
    // 检查当前 URL，如果已经在云端页面则不再导航
    const currentUrl = window.location.href;
    if (currentUrl.includes('uat-desktop.cheersai.cloud') || 
        currentUrl.includes('desktop.cheersai.cloud')) {
      console.log('Already on cloud page, skipping navigation');
      return;
    }

    // 如果是返回状态，不导航
    if (isReturning) {
      console.log('Returning from cloud, skipping navigation');
      return;
    }

    if (hasNavigated.current) return;
    hasNavigated.current = true;

    tauriCommands.navigateMainWindowWithButton(cloudUrl, window.location.origin)
      .catch((err: unknown) => console.error("Navigation failed:", err));
  }, [location]);

  return (
    <div className="flex items-center justify-center h-full">
      <div className="text-center">
        <img src="/safer.png" alt="CheersAI" className="w-14 h-14 animate-spin mx-auto mb-4" />
        <p className="text-base font-medium text-gray-700">CheersAI，让数据留在本地，让 AI 能力走在前沿</p>
      </div>
    </div>
  );
}
