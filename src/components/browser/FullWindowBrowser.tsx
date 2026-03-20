import { useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { tauriCommands } from "@/lib/tauri";

export default function FullWindowBrowser({ 
  initialUrl = "https://7smile.dlithink.com/cheersai_desktop/apps/"
}: { initialUrl?: string }) {
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    const navigateWithButton = async () => {
      try {
        // 使用 Tauri 命令导航主窗口并注入返回按钮
        const returnUrl = window.location.origin;
        await tauriCommands.navigateMainWindowWithButton(initialUrl, returnUrl);
      } catch (err) {
        console.error("Failed to navigate with button:", err);
        setError(`导航失败: ${err}`);
      }
    };

    navigateWithButton();
  }, [initialUrl]);

  return (
    <div className="fixed inset-0 z-50 bg-white flex items-center justify-center">
      <div className="text-center">
        {error ? (
          <>
            <div className="text-red-600 mb-4">{error}</div>
            <p className="text-gray-600">请刷新页面重试</p>
          </>
        ) : (
          <>
            <Loader2 className="w-8 h-8 animate-spin mx-auto mb-4 text-blue-600" />
            <p className="text-gray-600">正在跳转到浏览器...</p>
          </>
        )}
      </div>
    </div>
  );
}