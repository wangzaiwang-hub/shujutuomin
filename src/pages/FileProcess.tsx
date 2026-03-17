import { PageHeader } from "@/components/layout/PageHeader";
import { DropZone } from "@/components/file/DropZone";
import { FileQueueItem } from "@/components/file/FileQueueItem";
import { BatchProgress } from "@/components/file/BatchProgress";
import { RuleSelector } from "@/components/file/RuleSelector";
import { PassphraseBox } from "@/components/common/PassphraseBox";
import { Button } from "@/components/ui/button";
import { useFileStore } from "@/store/fileStore";
import { useLogStore } from "@/store/logStore";
import { Play, Trash2, FolderOpen } from "lucide-react";
import { v4 as uuidv4 } from "uuid";
import { stat, exists } from "@tauri-apps/plugin-fs";
import { open } from "@tauri-apps/plugin-dialog";
import { tauriCommands } from "@/lib/tauri";
import { useEffect, useState } from "react";
import { getDisplayPath, validatePath, getDefaultDocumentsPath } from "@/lib/path";
import type { BatchStatus } from "@/types/commands";

export default function FileProcess() {
  const { 
    files, 
    passphrase, 
    outputDir,
    activeJobId,
    addFiles, 
    removeFile, 
    clearCompleted, 
    setPassphrase,
    setOutputDir,
    setActiveJob,
    updateFile
  } = useFileStore();

  const { addLog } = useLogStore();

  const [batchStatus, setBatchStatus] = useState<BatchStatus | null>(null);
  const [selectedRules, setSelectedRules] = useState<string[]>([]);

  // 轮询批处理状态
  useEffect(() => {
    if (!activeJobId) {
      setBatchStatus(null);
      return;
    }

    const interval = setInterval(async () => {
      try {
        const status = await tauriCommands.getBatchStatus(activeJobId);
        setBatchStatus(status);
        
        // 更新文件状态
        files.forEach((file, index) => {
          if (index < status.completed) {
            updateFile(file.id, { status: "completed" });
          } else if (index === status.completed && status.current_file) {
            updateFile(file.id, { status: "processing" });
          } else if (index < status.completed + status.failed) {
            updateFile(file.id, { 
              status: "failed", 
              error: status.error || "处理失败" 
            });
          }
        });

        // 如果任务完成，停止轮询
        if (status.status === "Completed" || status.status === "Failed" || status.status === "Cancelled") {
          setActiveJob(null);
          setTimeout(() => setBatchStatus(null), 3000); // 3秒后清除状态显示
        }
      } catch (error) {
        console.error("Failed to get batch status:", error);
        setActiveJob(null);
        setBatchStatus(null);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [activeJobId, files, updateFile, setActiveJob]);

  const handleDrop = async (paths: string[]) => {
    const queued = await Promise.all(
      paths.map(async (p) => {
        let size = 0;
        try {
          console.log(`Checking file: ${p}`);
          const fileExists = await exists(p);
          console.log(`File exists: ${fileExists}`);
          
          if (fileExists) {
            const info = await stat(p);
            size = info.size ?? 0;
            console.log(`File size: ${size} bytes`);
          } else {
            console.warn(`File does not exist: ${p}`);
            size = -1;
          }
        } catch (error) {
          console.error(`Failed to get file info for ${p}:`, error);
          size = -1;
        }
        
        const fileName = p.split(/[\\/]/).pop() ?? p;
        console.log(`Creating queued file: ${fileName} (${p})`);
        
        return {
          id: uuidv4(),
          name: fileName,
          path: p,
          size,
          status: "pending" as const,
          addedAt: Date.now(),
        };
      })
    );
    addFiles(queued);
  };

  const handleSelectOutputDir = async () => {
    try {
      const selected = await open({
        directory: true,
        title: "选择输出目录",
        defaultPath: outputDir || getDefaultDocumentsPath(),
      });
      if (selected) {
        const selectedPath = selected as string;
        const validation = validatePath(selectedPath);
        
        if (validation.valid) {
          setOutputDir(selectedPath);
        } else {
          alert(`路径无效: ${validation.error}`);
        }
      }
    } catch (error) {
      console.error("Failed to select output directory:", error);
    }
  };

  const handleStart = async () => {
    if (pendingCount === 0) return;
    if (!outputDir) {
      alert("请先选择输出目录");
      return;
    }
    if (selectedRules.length === 0) {
      alert("请至少选择一个脱敏规则");
      return;
    }

    // 记录开始处理日志
    await addLog("info", `开始批处理 ${pendingCount} 个文件`, `输出目录: ${outputDir}`, undefined, "batch_start");

    try {
      const jobId = await tauriCommands.startBatchJob({
        file_paths: files.filter(f => f.status === "pending").map(f => f.path),
        output_dir: outputDir,
        rule_ids: selectedRules,
        passphrase: passphrase || undefined,
      });

      setActiveJob(jobId);
      
      // 将所有待处理文件状态设为处理中
      files.forEach(file => {
        if (file.status === "pending") {
          updateFile(file.id, { status: "processing" });
        }
      });
    } catch (error) {
      console.error("Failed to start batch job:", error);
      await addLog("error", "启动批处理失败", `错误: ${error}`, undefined, "batch_error");
      alert(`启动批处理失败: ${error}`);
    }
  };

  const pendingCount = files.filter((f) => f.status === "pending").length;

  return (
    <div className="flex flex-col h-full">
      <PageHeader
        title="文件处理"
        description="拖放文件进行数据脱敏"
        actions={
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={clearCompleted}>
              <Trash2 className="w-4 h-4 mr-1" />
              清除已完成
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleSelectOutputDir}
            >
              <FolderOpen className="w-4 h-4 mr-1" />
              选择输出目录
            </Button>
            <Button
              size="sm"
              onClick={handleStart}
              disabled={pendingCount === 0 || !outputDir || !!activeJobId || selectedRules.length === 0}
              className="bg-indigo-500 hover:bg-indigo-600"
            >
              <Play className="w-4 h-4 mr-1" />
              {activeJobId ? "处理中..." : `开始处理 ${pendingCount > 0 ? `(${pendingCount})` : ""}`}
            </Button>
          </div>
        }
      />
      <div className="flex-1 overflow-auto p-6">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2 space-y-6">
            <DropZone onFilesDropped={handleDrop} />

            <div className="space-y-4">
              <PassphraseBox
                value={passphrase}
                onChange={setPassphrase}
                label="映射加密口令（可选）"
              />
              
              {outputDir && (
                <div className="p-3 bg-green-50 border border-green-200 rounded-lg">
                  <p className="text-sm text-green-700">
                    <strong>输出目录:</strong> {getDisplayPath(outputDir, 60)}
                  </p>
                </div>
              )}

              {batchStatus && (
                <BatchProgress
                  total={batchStatus.total}
                  completed={batchStatus.completed}
                  failed={batchStatus.failed}
                  currentFile={batchStatus.current_file}
                  status={batchStatus.status}
                />
              )}
            </div>

            {files.length > 0 && (
              <div className="space-y-2">
                <p className="text-sm font-medium text-gray-700">
                  文件队列 ({files.length})
                </p>
                {files.map((f) => (
                  <FileQueueItem key={f.id} file={f} onRemove={removeFile} />
                ))}
              </div>
            )}
          </div>

          <div className="space-y-6">
            <RuleSelector
              selectedRules={selectedRules}
              onRulesChange={setSelectedRules}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
