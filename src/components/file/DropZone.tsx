import { useCallback } from "react";
import { useDropzone } from "react-dropzone";
import { UploadCloud } from "lucide-react";
import { cn } from "@/lib/utils";
import { open } from "@tauri-apps/plugin-dialog";

interface DropZoneProps {
  onFilesDropped: (paths: string[]) => void;
}

export function DropZone({ onFilesDropped }: DropZoneProps) {
  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "支持的文件",
            extensions: ["csv", "xlsx", "xls", "json", "txt", "docx", "doc", "pptx", "ppt", "md", "markdown"]
          }
        ]
      });
      
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        console.log("Selected files via dialog:", paths);
        onFilesDropped(paths);
      }
    } catch (error) {
      console.error("Failed to open file dialog:", error);
    }
  };

  const onDrop = useCallback(
    (accepted: File[]) => {
      console.log("Files dropped:", accepted);
      
      const paths = accepted.map((f) => {
        // 在 Tauri 中，文件对象应该有完整的路径信息
        console.log("File object keys:", Object.keys(f));
        console.log("File object:", f);
        
        // 尝试多种方式获取文件路径
        const possiblePaths = [
          (f as any).path,
          (f as any).webkitRelativePath,
          (f as any).mozFullPath,
          f.name
        ];
        
        console.log("Possible paths:", possiblePaths);
        
        // 选择第一个非空的路径
        const filePath = possiblePaths.find(p => p && p.length > 0) || f.name;
        console.log("Selected path:", filePath);
        
        return filePath;
      });
      
      console.log("Final paths to process:", paths);
      onFilesDropped(paths);
    },
    [onFilesDropped]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      "text/csv": [".csv"],
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet": [".xlsx"],
      "application/vnd.ms-excel": [".xls"],
      "application/json": [".json"],
      "text/plain": [".txt"],
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document": [".docx"],
      "application/msword": [".doc"],
      "application/vnd.openxmlformats-officedocument.presentationml.presentation": [".pptx"],
      "application/vnd.ms-powerpoint": [".ppt"],
      "text/markdown": [".md", ".markdown"],
    },
    multiple: true,
    noClick: true, // 禁用点击，我们使用自定义的点击处理
  });

  return (
    <div
      {...getRootProps()}
      className={cn(
        "flex flex-col items-center justify-center w-full h-48 border-2 border-dashed rounded-xl cursor-pointer transition-colors",
        isDragActive
          ? "border-indigo-400 bg-indigo-50"
          : "border-gray-200 bg-gray-50 hover:border-indigo-300 hover:bg-indigo-50/50"
      )}
      onClick={handleFileSelect}
    >
      <input {...getInputProps()} />
      <UploadCloud
        className={cn(
          "w-10 h-10 mb-3 transition-colors",
          isDragActive ? "text-indigo-500" : "text-gray-400"
        )}
      />
      <p className="text-sm font-medium text-gray-600">
        {isDragActive ? "释放文件以添加" : "拖放文件到此处，或点击选择"}
      </p>
      <p className="mt-1 text-xs text-gray-400">
        支持 CSV、Excel、JSON、TXT、Word、PowerPoint、Markdown
      </p>
      <p className="mt-0.5 text-xs text-gray-400">
        注：Word/PPT 将输出为 TXT 格式
      </p>
    </div>
  );
}
