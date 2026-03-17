import { PageHeader } from "@/components/layout/PageHeader";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import { useRuleStore } from "@/store/ruleStore";

export default function RuleConfig() {
  const { rules, toggleRule } = useRuleStore();

  const builtin = rules.filter((r) => r.builtin);
  const custom = rules.filter((r) => !r.builtin);

  return (
    <div className="flex flex-col h-full">
      <PageHeader
        title="规则配置"
        description="管理内置与自定义脱敏规则"
      />
      <div className="flex-1 overflow-auto p-6 space-y-6">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">内置规则</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {builtin.map((rule) => (
              <div key={rule.id} className="flex items-center justify-between py-2 border-b border-gray-100 last:border-0">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-gray-800">{rule.name}</span>
                    <Badge variant="outline" className="text-xs">内置</Badge>
                  </div>
                  {rule.description && (
                    <p className="text-xs text-gray-400 mt-0.5">{rule.description}</p>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <Label htmlFor={`rule-${rule.id}`} className="text-xs text-gray-500">
                    {rule.enabled ? "已启用" : "已禁用"}
                  </Label>
                  <Switch
                    id={`rule-${rule.id}`}
                    checked={rule.enabled}
                    onCheckedChange={() => toggleRule(rule.id)}
                  />
                </div>
              </div>
            ))}
          </CardContent>
        </Card>

        {custom.length > 0 && (
          <Card>
            <CardHeader>
              <CardTitle className="text-base">自定义规则</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {custom.map((rule) => (
                <div key={rule.id} className="flex items-center justify-between py-2 border-b border-gray-100 last:border-0">
                  <span className="text-sm font-medium text-gray-800">{rule.name}</span>
                  <Switch
                    checked={rule.enabled}
                    onCheckedChange={() => toggleRule(rule.id)}
                  />
                </div>
              ))}
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
