import { useState, useEffect } from "react";
import { PageHeader } from "@/components/layout/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { 
  Lock, 
  Unlock, 
  FileText, 
  FolderOpen, 
  Shield, 
  Eye,
  EyeOff,
  Save,
  RefreshCw,
  Info
} from "lucide-react";
import { useSandboxStore } from "@/store/sandboxStore";
import { useFileStore } from "@/store/fileStore";
import { tauriCommands } from "@/lib/tauri";
import { formatBytes } from "@/lib/utils";
import { getDisplayPath, validatePath, getPlatform, getDefaultDocumentsPath } from "@/lib/path";
import { open } from "@tauri-apps/plugin-dialog";

export default function SandboxManager() {
  const { locked, files, setLocked, setFiles } = useSandboxStore();
  const { passphrase, outputDir, setPassphrase, setOutputDir } = useFileStore();
  
  // 本地状态
  const [pin, setPin] = useState("");
  const [newPin, setNewPin] = useState("");
  const [confirmPin, setConfirmPin] = useState("");
  const [showPin, setShowPin] = useState(false);
  const [showNewPin, setShowNewPin] = useState(false);
  const [loading, setLoading] = useState(false);
  const [pathError, setPathError] = useState<string>("");

  // 获取当前平台信息
  const platform = getPlatform();
  const platformName = platform === 'windows' ? 'Windows' : platform === 'macos' ? 'macOS' : 'Linux';

  // 加载沙箱文件列表（基于输出目录）
  const loadFiles = async () => {
    if (!locked && outputDir) {
      try {
        const fileList = await tauriCommands.listFilesInDirectory(outputDir);
        setFiles(fileList);
      } catch (error) {
        console.error("Failed to load sandbox files:", error);
        setFiles([]); // 如果失败，设置为空数组
      }
    } else {
      setFiles([]);
    }
  };

  useEffect(() => {
    loadFiles();
  }, [locked, outputDir]);

  // 验证PIN
  const handleUnlock = async () => {
    if (!pin) return;
    
    setLoading(true);
    try {
      const ok = await tauriCommands.verifyPin(pin);
      if (ok) {
        setLocked(false);
        setPin("");
        await loadFiles();
      } else {
        alert("PIN 错误");
      }
    } catch (error) {
      alert("验证失败");
    } finally {
      setLoading(false);
    }
  };

  // 设置新PIN
  const handleSetPin = async () => {
    if (!newPin || newPin !== confirmPin) {
      alert("PIN 不匹配");
      return;
    }
    
    if (newPin.length < 4) {
      alert("PIN 至少需要4位");
      return;
    }

    setLoading(true);
    try {
      await tauriCommands.setPin(newPin);
      setNewPin("");
      setConfirmPin("");
      alert("PIN 设置成功");
    } catch (error) {
      alert("PIN 设置失败");
    } finally {
      setLoading(false);
    }
  };

  // 选择输出路径（跨平台）
  const handleSelectPath = async () => {
    try {
      const selected = await open({
        directory: true,
        title: "选择文件输出/沙箱存储路径",
        defaultPath: outputDir || getDefaultDocumentsPath(),
      });
      if (selected) {
        const selectedPath = selected as string;
        const validation = validatePath(selectedPath);
        
        if (!validation.valid) {
          setPathError(validation.error || "路径无效");
          return;
        }
        
        setPathError("");
        setOutputDir(selectedPath);
      }
    } catch (error) {
      console.error("Failed to select output directory:", error);
      setPathError("选择路径失败");
    }
  };

  // 手动输入路径时验证
  const handlePathChange = (path: string) => {
    setOutputDir(path);
    
    if (path) {
      const validation = validatePath(path);
      setPathError(validation.valid ? "" : validation.error || "路径无效");
    } else {
      setPathError("");
    }
  };

  // 使用默认路径
  const handleUseDefaultPath = () => {
    const defaultPath = getDefaultDocumentsPath();
    setOutputDir(defaultPath);
    setPathError("");
  };

  // 生成随机口令
  const generatePassphrase = async () => {
    try {
      const newPassphrase = await tauriCommands.generatePassphrase();
      setPassphrase(newPassphrase);
    } catch (error) {
      console.error("Failed to generate passphrase:", error);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <PageHeader
        title="沙箱管理"
        description="安全设置和文件输出配置"
        actions={
          <div className="flex gap-2">
            {!locked && (
              <Button size="sm" variant="outline" onClick={() => setLocked(true)}>
                <Lock className="w-4 h-4 mr-1" />
                锁定沙箱
              </Button>
            )}
          </div>
        }
      />

      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-4xl mx-auto space-y-6">
          
          {/* 沙箱状态卡片 */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                {locked ? (
                  <Lock className="w-5 h-5 text-red-500" />
                ) : (
                  <Unlock className="w-5 h-5 text-green-500" />
                )}
                沙箱状态
              </CardTitle>
            </CardHeader>
            <CardContent>
              {locked ? (
                <div className="space-y-4">
                  <p className="text-sm text-gray-600">沙箱已锁定，请输入 PIN 解锁</p>
                  <div className="flex gap-2 max-w-md">
                    <div className="relative flex-1">
                      <Input
                        type={showPin ? "text" : "password"}
                        placeholder="输入 PIN"
                        value={pin}
                        onChange={(e) => setPin(e.target.value)}
                        onKeyDown={(e) => e.key === "Enter" && handleUnlock()}
                      />
                      <button
                        type="button"
                        onClick={() => setShowPin(!showPin)}
                        className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
                      >
                        {showPin ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                      </button>
                    </div>
                    <Button onClick={handleUnlock} disabled={!pin || loading}>
                      <Unlock className="w-4 h-4 mr-1" />
                      解锁
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="space-y-4">
                  <p className="text-sm text-green-600">沙箱已解锁，可以访问安全文件</p>
                  <div className="text-sm text-gray-600">
                    文件数量: {files.length} 个
                  </div>
                </div>
              )}
            </CardContent>
          </Card>

          {/* 安全设置 */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Shield className="w-5 h-5 text-blue-500" />
                安全设置
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              
              {/* PIN 设置 */}
              <div className="space-y-4">
                <Label className="text-sm font-medium">设置新 PIN</Label>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="relative">
                    <Input
                      type={showNewPin ? "text" : "password"}
                      placeholder="新 PIN (至少4位)"
                      value={newPin}
                      onChange={(e) => setNewPin(e.target.value)}
                    />
                    <button
                      type="button"
                      onClick={() => setShowNewPin(!showNewPin)}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
                    >
                      {showNewPin ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </button>
                  </div>
                  <Input
                    type="password"
                    placeholder="确认 PIN"
                    value={confirmPin}
                    onChange={(e) => setConfirmPin(e.target.value)}
                  />
                </div>
                <Button 
                  onClick={handleSetPin} 
                  disabled={!newPin || !confirmPin || loading}
                  size="sm"
                >
                  <Save className="w-4 h-4 mr-1" />
                  设置 PIN
                </Button>
              </div>

              <Separator />

              {/* 默认加密口令 */}
              <div className="space-y-4">
                <Label className="text-sm font-medium">默认加密口令</Label>
                <p className="text-xs text-gray-500">
                  用于脱敏映射文件的加密，设置后会自动应用到文件处理
                </p>
                <div className="flex gap-2 max-w-md">
                  <Input
                    type="password"
                    placeholder="加密口令"
                    value={passphrase}
                    onChange={(e) => setPassphrase(e.target.value)}
                  />
                  <Button variant="outline" size="icon" onClick={generatePassphrase}>
                    <RefreshCw className="w-4 h-4" />
                  </Button>
                </div>
              </div>

              <Separator />

              {/* 输出/沙箱路径 */}
              <div className="space-y-4">
                <div className="flex items-center gap-2">
                  <Label className="text-sm font-medium">文件输出路径</Label>
                  <div className="px-2 py-1 bg-blue-100 text-blue-700 text-xs rounded">
                    {platformName}
                  </div>
                </div>
                <p className="text-xs text-gray-500">
                  脱敏文件的输出目录，同时也是沙箱的存储位置
                </p>
                
                <div className="space-y-2">
                  <div className="flex gap-2">
                    <Input
                      placeholder={`选择文件输出路径 (${platform === 'windows' ? 'C:\\Users\\用户名\\Documents\\CheersAI Vault' : '~/Documents/CheersAI Vault'})`}
                      value={outputDir}
                      onChange={(e) => handlePathChange(e.target.value)}
                      className={`flex-1 ${pathError ? 'border-red-300' : ''}`}
                    />
                    <Button variant="outline" onClick={handleSelectPath}>
                      <FolderOpen className="w-4 h-4 mr-1" />
                      浏览
                    </Button>
                  </div>
                  
                  {pathError && (
                    <p className="text-sm text-red-600 flex items-center gap-1">
                      <Info className="w-4 h-4" />
                      {pathError}
                    </p>
                  )}
                  
                  <Button 
                    variant="ghost" 
                    size="sm" 
                    onClick={handleUseDefaultPath}
                    className="text-xs"
                  >
                    使用默认路径 ({getDisplayPath(getDefaultDocumentsPath(), 40)})
                  </Button>
                </div>
                
                {outputDir && !pathError && (
                  <div className="p-3 bg-green-50 border border-green-200 rounded-lg">
                    <p className="text-sm text-green-700">
                      <strong>当前路径:</strong> {getDisplayPath(outputDir, 60)}
                    </p>
                    <p className="text-xs text-green-600 mt-1">
                      脱敏文件和映射文件都将保存到此目录
                    </p>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

          {/* 沙箱文件列表 */}
          {!locked && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <FileText className="w-5 h-5 text-gray-500" />
                  输出目录文件
                </CardTitle>
                {outputDir && (
                  <p className="text-xs text-gray-500">
                    位置: {getDisplayPath(outputDir, 50)}
                  </p>
                )}
              </CardHeader>
              <CardContent>
                {!outputDir ? (
                  <div className="text-center py-8 text-gray-400">
                    <FolderOpen className="w-12 h-12 mx-auto mb-2 opacity-50" />
                    <p className="text-sm">请先设置文件输出路径</p>
                  </div>
                ) : files.length === 0 ? (
                  <div className="text-center py-8 text-gray-400">
                    <FileText className="w-12 h-12 mx-auto mb-2 opacity-50" />
                    <p className="text-sm">输出目录为空</p>
                    <p className="text-xs mt-1">处理文件后，脱敏文件和映射文件将显示在这里</p>
                  </div>
                ) : (
                  <div className="space-y-2">
                    {files.map((file) => (
                      <div
                        key={file.name}
                        className="flex items-center gap-3 p-3 rounded-lg border hover:bg-gray-50"
                      >
                        <FileText className="w-5 h-5 text-gray-400" />
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium text-gray-800 truncate">
                            {file.name}
                          </p>
                          <p className="text-xs text-gray-500">
                            {formatBytes(file.size)} · {file.modified}
                          </p>
                        </div>
                        {file.name.endsWith('.cmap') && (
                          <div className="px-2 py-1 bg-blue-100 text-blue-700 text-xs rounded">
                            映射文件
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
