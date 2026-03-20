import { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Search, Replace, X } from 'lucide-react';
import { Badge } from '@/components/ui/badge';

interface FindReplaceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onReplace: (findText: string, replaceText: string) => void;
  detectedEntities?: Array<{ text: string; entity_type: string }>;
}

export function FindReplaceDialog({
  open,
  onOpenChange,
  onReplace,
  detectedEntities = [],
}: FindReplaceDialogProps) {
  const [findText, setFindText] = useState('');
  const [replaceText, setReplaceText] = useState('');

  // 去重实体列表
  const uniqueEntities = Array.from(
    new Map(detectedEntities.map(e => [e.text, e])).values()
  );

  const handleReplace = () => {
    if (!findText.trim()) {
      return;
    }
    onReplace(findText, replaceText);
    onOpenChange(false);
  };

  const handleQuickSelect = (text: string, type: string) => {
    setFindText(text);
    // 根据实体类型生成替换文本
    const typeMap: Record<string, string> = {
      '姓名': '姓名',
      '手机号': '***PHONE***',
      '邮箱': '***EMAIL***',
      '身份证号': '***IDCARD***',
      '银行卡号': '***BANKCARD***',
      'IP地址': '***IP***',
      '护照号': '***PASSPORT***',
      '地址': '***ADDRESS***',
      '日期': '***DATE***',
      '金额': '***AMOUNT***',
    };
    setReplaceText(typeMap[type] || '***MASKED***');
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Search className="w-5 h-5" />
            查找替换
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          {/* 查找输入 */}
          <div className="space-y-2">
            <Label htmlFor="find-text">查找内容</Label>
            <Input
              id="find-text"
              placeholder="输入要查找的文本..."
              value={findText}
              onChange={(e) => setFindText(e.target.value)}
              className="font-mono"
            />
          </div>

          {/* 替换输入 */}
          <div className="space-y-2">
            <Label htmlFor="replace-text">替换为</Label>
            <Input
              id="replace-text"
              placeholder="输入替换后的文本..."
              value={replaceText}
              onChange={(e) => setReplaceText(e.target.value)}
              className="font-mono"
            />
          </div>

          {/* NER 检测到的实体快捷选择 */}
          {uniqueEntities.length > 0 && (
            <div className="space-y-2">
              <Label>检测到的敏感信息（点击快速填充）</Label>
              <div className="flex flex-wrap gap-2 p-3 bg-gray-50 rounded-lg max-h-40 overflow-y-auto">
                {uniqueEntities.map((entity, idx) => (
                  <Badge
                    key={idx}
                    variant="outline"
                    className="cursor-pointer hover:bg-blue-100 transition-colors"
                    onClick={() => handleQuickSelect(entity.text, entity.entity_type)}
                  >
                    <span className="text-orange-700 font-medium mr-1">
                      {entity.entity_type}:
                    </span>
                    <span className="text-gray-700">{entity.text}</span>
                  </Badge>
                ))}
              </div>
              <p className="text-xs text-gray-500">
                💡 点击上方标签可快速填充查找和替换内容
              </p>
            </div>
          )}

          {/* 预览 */}
          {findText && (
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
              <p className="text-sm text-blue-900 mb-2">替换预览：</p>
              <div className="font-mono text-sm">
                <span className="bg-red-100 text-red-700 line-through px-1 rounded">
                  {findText}
                </span>
                {' → '}
                <span className="bg-green-100 text-green-700 font-medium px-1 rounded">
                  {replaceText || '(空)'}
                </span>
              </div>
            </div>
          )}
        </div>

        <DialogFooter className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            className="flex items-center gap-2"
          >
            <X className="w-4 h-4" />
            取消
          </Button>
          <Button
            onClick={handleReplace}
            disabled={!findText.trim()}
            className="bg-blue-600 hover:bg-blue-700 flex items-center gap-2"
          >
            <Replace className="w-4 h-4" />
            全部替换
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
