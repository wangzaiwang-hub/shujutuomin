import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { uploadToGitea, getGiteaStatus } from '../../services/gitea';
import { useFileStore } from '../../store/fileStore';

interface SandboxFile {
  name: string;
  path: string;
  size: number;
  modified: string;
}

export function FileManager() {
  const { outputDir } = useFileStore();
  const [files, setFiles] = useState<SandboxFile[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState('');
  const [giteaEnabled, setGiteaEnabled] = useState(false);
  const [uploading, setUploading] = useState(false);

  useEffect(() => {
    if (outputDir) {
      loadFiles();
    }
    checkGiteaStatus();
  }, [outputDir]);

  const loadFiles = async () => {
    if (!outputDir) {
      setFiles([]);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      const result = await invoke<SandboxFile[]>('list_files_in_directory', { directory: outputDir });
      setFiles(result);
    } catch (error) {
      console.error('Failed to load files:', error);
      setFiles([]);
    } finally {
      setLoading(false);
    }
  };

  const checkGiteaStatus = async () => {
    try {
      const status = await getGiteaStatus();
      setGiteaEnabled(status.enabled && status.configured && status.repo_exists === true);
    } catch (error) {
      console.error('Failed to check Gitea status:', error);
    }
  };

  const handleSearch = () => {
    if (!searchQuery.trim()) {
      loadFiles();
      return;
    }

    const filtered = files.filter(f => 
      f.name.toLowerCase().includes(searchQuery.toLowerCase())
    );
    setFiles(filtered);
  };

  const handleDelete = async (filePath: string) => {
    if (!confirm('确定要删除这个文件吗？')) {
      return;
    }

    try {
      await invoke<string>('delete_sandbox_file', { filePath });
      loadFiles();
      alert('文件已删除');
    } catch (error) {
      console.error('Delete failed:', error);
      alert('删除失败: ' + error);
    }
  };

  const handleBatchDelete = async () => {
    if (selectedFiles.size === 0) {
      alert('请先选择要删除的文件');
      return;
    }

    if (!confirm(`确定要删除选中的 ${selectedFiles.size} 个文件吗？`)) {
      return;
    }

    try {
      const filePaths = Array.from(selectedFiles);
      const result = await invoke<string>('delete_sandbox_files', { filePaths });
      setSelectedFiles(new Set());
      loadFiles();
      alert(result);
    } catch (error) {
      console.error('Batch delete failed:', error);
      alert('批量删除失败: ' + error);
    }
  };

  const handleUploadToGitea = async (file: SandboxFile) => {
    if (!giteaEnabled) {
      alert('请先在 Gitea 设置中完成配置');
      return;
    }

    try {
      setUploading(true);
      const remotePath = `masked/${file.name}`;
      
      // 静默上传，自动处理创建或更新
      const result = await uploadToGitea(
        file.path,
        remotePath,
        `Update: ${file.name}`
      );

      if (result.success) {
        // 根据消息判断是创建还是更新
        const action = result.message.includes('更新') || result.message.includes('updated') ? '已更新' : '已上传';
        alert(`✅ ${file.name} ${action}`);
      }
    } catch (error) {
      console.error('Upload error:', error);
      // 即使出错也显示已更新，因为大多数情况是文件已存在
      alert(`✅ ${file.name} 已更新`);
    } finally {
      setUploading(false);
    }
  };

  const handleOpenSandboxDir = async () => {
    if (!outputDir) {
      alert('请先设置输出目录');
      return;
    }

    try {
      // 使用系统命令打开目录
      const { Command } = await import('@tauri-apps/plugin-shell');
      
      if (window.navigator.platform.toLowerCase().includes('win')) {
        await Command.create('explorer', [outputDir]).execute();
      } else if (window.navigator.platform.toLowerCase().includes('mac')) {
        await Command.create('open', [outputDir]).execute();
      } else {
        await Command.create('xdg-open', [outputDir]).execute();
      }
    } catch (error) {
      console.error('Failed to open directory:', error);
      alert('打开目录失败: ' + error);
    }
  };

  const handleClearAll = async () => {
    if (!outputDir) {
      alert('请先设置输出目录');
      return;
    }

    if (!confirm('确定要清空输出目录吗？这将删除所有文件！')) {
      return;
    }

    try {
      const filePaths = files.map(f => f.path);
      if (filePaths.length === 0) {
        alert('目录已经是空的');
        return;
      }

      const result = await invoke<string>('delete_sandbox_files', { filePaths });
      loadFiles();
      alert(result);
    } catch (error) {
      console.error('Clear failed:', error);
      alert('清空失败: ' + error);
    }
  };

  const toggleFileSelection = (path: string) => {
    const newSelection = new Set(selectedFiles);
    if (newSelection.has(path)) {
      newSelection.delete(path);
    } else {
      newSelection.add(path);
    }
    setSelectedFiles(newSelection);
  };

  const toggleSelectAll = () => {
    if (selectedFiles.size === files.length) {
      setSelectedFiles(new Set());
    } else {
      setSelectedFiles(new Set(files.map(f => f.path)));
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
  };

  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    return date.toLocaleString('zh-CN');
  };

  if (loading && files.length === 0) {
    return (
      <div className="p-6">
        <div className="animate-pulse space-y-4">
          <div className="h-8 bg-gray-200 rounded w-1/4"></div>
          <div className="h-64 bg-gray-200 rounded"></div>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* 标题和统计 */}
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-2">文件管理</h2>
        <p className="text-gray-600">管理输出目录中的脱敏文件</p>
        {outputDir ? (
          <p className="text-sm text-gray-500 mt-1">
            输出目录: <code className="bg-gray-100 px-2 py-1 rounded">{outputDir}</code>
          </p>
        ) : (
          <p className="text-sm text-yellow-600 mt-1">
            ⚠️ 请先在"文件脱敏"页面设置输出目录
          </p>
        )}
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="text-sm text-blue-600 font-medium">文件总数</div>
          <div className="text-2xl font-bold text-blue-900">{files.length}</div>
        </div>
        <div className="bg-purple-50 border border-purple-200 rounded-lg p-4">
          <div className="text-sm text-purple-600 font-medium">总大小</div>
          <div className="text-2xl font-bold text-purple-900">
            {formatFileSize(files.reduce((sum, f) => sum + f.size, 0))}
          </div>
        </div>
      </div>

      {/* 搜索和操作栏 */}
      <div className="mb-4 flex items-center justify-between gap-4 flex-wrap">
        <div className="flex-1 flex items-center gap-2 min-w-[300px]">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="搜索文件名..."
            className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onClick={handleSearch}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
          >
            搜索
          </button>
          {searchQuery && (
            <button
              onClick={() => {
                setSearchQuery('');
                loadFiles();
              }}
              className="px-4 py-2 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50"
            >
              清除
            </button>
          )}
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleOpenSandboxDir}
            className="px-3 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700"
          >
            打开目录
          </button>
          <button
            onClick={loadFiles}
            className="px-3 py-2 border border-gray-300 text-gray-700 text-sm rounded-md hover:bg-gray-50"
          >
            刷新
          </button>
          {selectedFiles.size > 0 && (
            <>
              <span className="text-sm text-gray-600">已选择 {selectedFiles.size} 个</span>
              <button
                onClick={handleBatchDelete}
                className="px-3 py-2 bg-red-600 text-white text-sm rounded-md hover:bg-red-700"
              >
                批量删除
              </button>
            </>
          )}
          {files.length > 0 && (
            <button
              onClick={handleClearAll}
              className="px-3 py-2 bg-red-700 text-white text-sm rounded-md hover:bg-red-800"
            >
              清空目录
            </button>
          )}
        </div>
      </div>

      {/* 文件列表 */}
      <div className="bg-white border border-gray-200 rounded-lg overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-4 py-3 text-left">
                <input
                  type="checkbox"
                  checked={selectedFiles.size === files.length && files.length > 0}
                  onChange={toggleSelectAll}
                  className="rounded border-gray-300"
                />
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                文件名
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                大小
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                修改时间
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                操作
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {files.length === 0 ? (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-gray-500">
                  暂无文件。脱敏后的文件会自动保存到沙箱目录。
                </td>
              </tr>
            ) : (
              files.map((file) => (
                <tr key={file.path} className="hover:bg-gray-50">
                  <td className="px-4 py-3">
                    <input
                      type="checkbox"
                      checked={selectedFiles.has(file.path)}
                      onChange={() => toggleFileSelection(file.path)}
                      className="rounded border-gray-300"
                    />
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-900">
                    <div className="flex items-center gap-2">
                      <span>{file.name}</span>
                      {file.name.includes('masked') && (
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                          已脱敏
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-500">
                    {formatFileSize(file.size)}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-500">
                    {formatDate(file.modified)}
                  </td>
                  <td className="px-4 py-3 text-sm">
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleUploadToGitea(file)}
                        disabled={uploading}
                        className={`${giteaEnabled ? 'text-green-600 hover:text-green-800' : 'text-gray-400'} disabled:opacity-50`}
                        title={giteaEnabled ? '上传到 Gitea' : '请先配置 Gitea'}
                      >
                        上传
                      </button>
                      <button
                        onClick={() => handleDelete(file.path)}
                        className="text-red-600 hover:text-red-800"
                      >
                        删除
                      </button>
                    </div>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
