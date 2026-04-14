import { useEffect, useState, useRef } from "react";
import { Loader2, ExternalLink } from "lucide-react";
import { tauriCommands } from "@/lib/tauri";
import { useNavigate } from "react-router-dom";
import { CLOUD_APP_URL } from "@/lib/cloud";

export default function FullWindowBrowser({ 
  initialUrl = CLOUD_APP_URL
}: { initialUrl?: string }) {
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();
  const hasOpened = useRef(false);
  
  useEffect(() => {
    if (hasOpened.current) return;
    hasOpened.current = true;

    const openWebview = async () => {
      try {
        const label = await tauriCommands.openWebviewWindow({
          url: initialUrl,
          title: "CheersAI 云端服务",
          width: 1400,
          height: 900
        });
        
        console.log("Webview window opened:", label);
        // 返回上一页而不是跳转到 /process
        navigate(-1);
      } catch (err) {
        console.error("Failed to open webview:", err);
        setError(`打开窗口失败: ${err}`);
      }
    };

    openWebview();
  }, [initialUrl, navigate]);

  return (
    <div className="fixed inset-0 z-50 bg-white flex items-center justify-center">
      <div className="text-center max-w-md">
        {error ? (
          <>
            <div className="text-red-600 mb-4">{error}</div>
            <p className="text-gray-600 mb-4">请刷新页面重试</p>
            <button
              onClick={() => navigate('/process')}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
            >
              返回应用
            </button>
          </>
        ) : (
          <>
            <ExternalLink className="w-16 h-16 mx-auto mb-6 text-blue-600" />
            <p className="text-lg font-semibold text-gray-800 mb-2">正在打开 CheersAI 云端服务</p>
            <p className="text-sm text-gray-500 mb-4">窗口将最大化显示</p>
            <Loader2 className="w-8 h-8 animate-spin mx-auto text-blue-600" />
          </>
        )}
      </div>
    </div>
  );
}
