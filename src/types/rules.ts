export interface RuleGroup {
  id: string;
  name: string;
  rules: string[];
}

export interface MaskRule {
  id: string;
  name: string;
  pattern: string;
  replacement: string;
  enabled: boolean;
  builtin: boolean;
  description?: string;
}
