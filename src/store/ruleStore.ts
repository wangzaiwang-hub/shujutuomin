import { create } from "zustand";
import type { MaskRule } from "@/types/rules";

const DEFAULT_RULES: MaskRule[] = [
  { id: "id_card", name: "身份证号", pattern: "", replacement: "***IDCARD***", enabled: true, builtin: true },
  { id: "phone", name: "手机号", pattern: "", replacement: "***PHONE***", enabled: true, builtin: true },
  { id: "email", name: "电子邮箱", pattern: "", replacement: "***EMAIL***", enabled: true, builtin: true },
  { id: "bank_card", name: "银行卡号", pattern: "", replacement: "***BANKCARD***", enabled: true, builtin: true },
  { id: "ipv4", name: "IPv4地址", pattern: "", replacement: "***IP***", enabled: true, builtin: true },
  { id: "chinese_name", name: "中文姓名", pattern: "", replacement: "***NAME***", enabled: false, builtin: true },
  { id: "passport", name: "护照号", pattern: "", replacement: "***PASSPORT***", enabled: true, builtin: true },
];

interface RuleStore {
  rules: MaskRule[];
  selectedRuleIds: string[];
  setRules: (rules: MaskRule[]) => void;
  toggleRule: (id: string) => void;
  addRule: (rule: MaskRule) => void;
  removeRule: (id: string) => void;
  setSelectedRuleIds: (ids: string[]) => void;
}

export const useRuleStore = create<RuleStore>((set) => ({
  rules: DEFAULT_RULES,
  selectedRuleIds: DEFAULT_RULES.filter((r) => r.enabled).map((r) => r.id),

  setRules: (rules) => set({ rules }),

  toggleRule: (id) =>
    set((state) => ({
      rules: state.rules.map((r) =>
        r.id === id ? { ...r, enabled: !r.enabled } : r
      ),
    })),

  addRule: (rule) =>
    set((state) => ({ rules: [...state.rules, rule] })),

  removeRule: (id) =>
    set((state) => ({ rules: state.rules.filter((r) => r.id !== id) })),

  setSelectedRuleIds: (ids) => set({ selectedRuleIds: ids }),
}));
