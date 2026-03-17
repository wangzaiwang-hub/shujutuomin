use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub replacement_template: String,
    pub enabled: bool,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingEntry {
    pub original: String,
    pub masked: String,
    pub rule_id: String,
}

static BUILTIN_RULES: Lazy<Vec<MaskingRule>> = Lazy::new(|| {
    vec![
        MaskingRule {
            id: "id_card".to_string(),
            name: "身份证号".to_string(),
            pattern: r"\b[1-9]\d{5}(18|19|20)\d{2}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])\d{3}[\dXx]\b".to_string(),
            replacement_template: "***IDCARD***".to_string(),
            enabled: true,
            builtin: true,
        },
        MaskingRule {
            id: "phone".to_string(),
            name: "手机号".to_string(),
            pattern: r"\b1[3-9]\d{9}\b".to_string(),
            replacement_template: "***PHONE***".to_string(),
            enabled: true,
            builtin: true,
        },
        MaskingRule {
            id: "email".to_string(),
            name: "电子邮箱".to_string(),
            pattern: r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b".to_string(),
            replacement_template: "***EMAIL***".to_string(),
            enabled: true,
            builtin: true,
        },
        MaskingRule {
            id: "bank_card".to_string(),
            name: "银行卡号".to_string(),
            pattern: r"\b[1-9]\d{15,18}\b".to_string(),
            replacement_template: "***BANKCARD***".to_string(),
            enabled: true,
            builtin: true,
        },
        MaskingRule {
            id: "ipv4".to_string(),
            name: "IPv4地址".to_string(),
            pattern: r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b".to_string(),
            replacement_template: "***IP***".to_string(),
            enabled: true,
            builtin: true,
        },
        MaskingRule {
            id: "chinese_name".to_string(),
            name: "中文姓名".to_string(),
            pattern: r"(?:[\u4e00-\u9fa5]{1})[·•]?[\u4e00-\u9fa5]{1,3}".to_string(),
            replacement_template: "***NAME***".to_string(),
            enabled: false,
            builtin: true,
        },
        MaskingRule {
            id: "passport".to_string(),
            name: "护照号".to_string(),
            pattern: r"\b[A-Za-z][0-9]{8}\b".to_string(),
            replacement_template: "***PASSPORT***".to_string(),
            enabled: true,
            builtin: true,
        },
    ]
});

pub fn get_builtin_rules() -> &'static Vec<MaskingRule> {
    &BUILTIN_RULES
}

pub fn mask_value(
    value: &str,
    rules: &[MaskingRule],
    mapping: &mut HashMap<String, MappingEntry>,
    counter: &mut usize,
) -> String {
    let mut result = value.to_string();

    for rule in rules {
        if !rule.enabled {
            continue;
        }
        let re = match Regex::new(&rule.pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let original = caps[0].to_string();
                if let Some(entry) = mapping.values().find(|e| e.original == original) {
                    return entry.masked.clone();
                }
                *counter += 1;
                let masked = format!("{}{}", rule.replacement_template, counter);
                mapping.insert(
                    format!("{}-{}", rule.id, counter),
                    MappingEntry {
                        original: original.clone(),
                        masked: masked.clone(),
                        rule_id: rule.id.clone(),
                    },
                );
                masked
            })
            .to_string();
    }

    result
}
