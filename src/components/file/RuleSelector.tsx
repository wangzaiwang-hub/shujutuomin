import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { tauriCommands } from "@/lib/tauri";
import type { MaskRule } from "@/types/commands";

interface RuleSelectorProps {
  selectedRules: string[];
  onRulesChange: (ruleIds: string[]) => void;
}

// 从 localStorage 获取保存的规则选择
const getSavedRules = (): string[] => {
  try {
    const saved = localStorage.getItem("selected-rules");
    return saved ? JSON.parse(saved) : [];
  } catch {
    return [];
  }
};

// 保存规则选择到 localStorage
const saveRules = (ruleIds: string[]) => {
  try {
    localStorage.setItem("selected-rules", JSON.stringify(ruleIds));
  } catch (error) {
    console.error("Failed to save rules:", error);
  }
};

export function RuleSelector({ selectedRules, onRulesChange }: RuleSelectorProps) {
  const [rules, setRules] = useState<MaskRule[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadRules = async () => {
      try {
        const allRules = await tauriCommands.getRules();
        setRules(allRules);
        
        // 首先尝试使用保存的规则选择
        const savedRules = getSavedRules();
        if (savedRules.length > 0) {
          // 验证保存的规则是否仍然有效
          const validSavedRules = savedRules.filter(ruleId => 
            allRules.some(rule => rule.id === ruleId)
          );
          if (validSavedRules.length > 0) {
            onRulesChange(validSavedRules);
            return;
          }
        }
        
        // 如果没有保存的规则或保存的规则无效，使用默认规则
        const defaultRules = allRules
          .filter(rule => rule.builtin && rule.enabled)
          .map(rule => rule.id);
        onRulesChange(defaultRules);
      } catch (error) {
        console.error("Failed to load rules:", error);
      } finally {
        setLoading(false);
      }
    };

    loadRules();
  }, [onRulesChange]);

  const handleRuleToggle = (ruleId: string, enabled: boolean) => {
    let newRules: string[];
    if (enabled) {
      newRules = [...selectedRules, ruleId];
    } else {
      newRules = selectedRules.filter(id => id !== ruleId);
    }
    
    onRulesChange(newRules);
    saveRules(newRules); // 保存到 localStorage
  };

  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-sm">脱敏规则</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-gray-500">加载中...</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">脱敏规则</CardTitle>
        <p className="text-xs text-gray-500">选择的规则会自动保存</p>
      </CardHeader>
      <CardContent className="space-y-3">
        {rules.map((rule) => (
          <div key={rule.id} className="flex items-center justify-between">
            <Label htmlFor={rule.id} className="text-sm font-normal">
              {rule.name}
            </Label>
            <Switch
              id={rule.id}
              checked={selectedRules.includes(rule.id)}
              onCheckedChange={(checked) => handleRuleToggle(rule.id, checked)}
            />
          </div>
        ))}
      </CardContent>
    </Card>
  );
}