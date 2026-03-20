use serde::{Deserialize, Serialize};
use crate::core::{masking_engine, file_parser, ner, crypto};

#[derive(Debug, Serialize, Deserialize)]
pub struct MaskFileOptions {
    pub file_path: String,
    pub output_path: String,
    pub rule_ids: Vec<String>,
    pub passphrase: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaskResult {
    pub output_path: String,
    pub masked_count: usize,
    pub mapping_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewOptions {
    pub file_path: String,
    pub rule_ids: Vec<String>,
    pub max_rows: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewResult {
    pub original_rows: Vec<Vec<String>>,
    pub masked_rows: Vec<Vec<String>>,
    pub headers: Vec<String>,
    pub detected_entities: Option<Vec<ner::RowEntities>>,
}

#[tauri::command]
pub async fn mask_file(options: MaskFileOptions) -> Result<MaskResult, String> {
    println!("=== MASK_FILE START ===");
    println!("File path: {}", options.file_path);
    println!("Output path: {}", options.output_path);
    println!("Rule IDs: {:?}", options.rule_ids);
    
    let format = file_parser::detect_format(&options.file_path);
    println!("Detected format: {:?}", format);
    
    let mut mapping = std::collections::HashMap::new();
    let mut counter = 0usize;
    let mapping_path = options.passphrase.as_ref().map(|_| {
        format!("{}.cmap", options.output_path)
    });

    // Get rules by IDs
    let all_rules = masking_engine::get_builtin_rules();
    let active_rules: Vec<_> = all_rules
        .iter()
        .filter(|r| options.rule_ids.contains(&r.id))
        .cloned()
        .collect();
    
    println!("Active rules count: {}", active_rules.len());

    match format {
        file_parser::FileFormat::Csv => {
            println!("Processing CSV file...");
            let (headers, rows) = file_parser::parse_csv(&options.file_path)
                .map_err(|e| {
                    println!("CSV parse error: {}", e);
                    format!("Failed to parse CSV: {}", e)
                })?;

            println!("CSV parsed successfully. Headers: {:?}, Rows: {}", headers, rows.len());

            let mut masked_rows = Vec::new();

            for (row_idx, row) in rows.iter().enumerate() {
                println!("Processing row {}/{}", row_idx + 1, rows.len());
                let mut masked_row = Vec::new();
                for cell in row {
                    let masked = masking_engine::mask_value(&cell, &active_rules, &mut mapping, &mut counter);
                    masked_row.push(masked);
                }
                masked_rows.push(masked_row);
            }

            println!("Writing CSV output...");
            file_parser::write_csv(&options.output_path, &headers, &masked_rows)
                .map_err(|e| {
                    println!("CSV write error: {}", e);
                    format!("Failed to write CSV: {}", e)
                })?;

            if let Some(passphrase) = &options.passphrase {
                if let Some(ref map_path) = mapping_path {
                    println!("Saving encrypted mapping...");
                    let mappings: Vec<_> = mapping.values().cloned().collect();
                    crypto::save_encrypted_mapping(map_path, &mappings, passphrase)
                        .map_err(|e| {
                            println!("Mapping save error: {}", e);
                            format!("Failed to save mapping: {}", e)
                        })?;
                }
            }
        }
        file_parser::FileFormat::Word => {
            println!("Processing Word file...");
            let content = file_parser::parse_word(&options.file_path)
                .map_err(|e| {
                    println!("Word parse error: {}", e);
                    format!("Failed to parse Word: {}", e)
                })?;

            let masked_content = masking_engine::mask_value(&content, &active_rules, &mut mapping, &mut counter);

            // Output as .txt file instead of .docx for simplicity
            let txt_output = options.output_path.replace(".docx", ".txt").replace(".doc", ".txt");
            println!("Writing Word output to: {}", txt_output);
            file_parser::write_markdown(&txt_output, &masked_content)
                .map_err(|e| {
                    println!("Word write error: {}", e);
                    format!("Failed to write Word: {}", e)
                })?;

            if let Some(passphrase) = &options.passphrase {
                if let Some(ref map_path) = mapping_path {
                    let mappings: Vec<_> = mapping.values().cloned().collect();
                    crypto::save_encrypted_mapping(map_path, &mappings, passphrase)
                        .map_err(|e| format!("Failed to save mapping: {}", e))?;
                }
            }
        }
        file_parser::FileFormat::PowerPoint => {
            println!("Processing PowerPoint file...");
            let content = file_parser::parse_powerpoint(&options.file_path)
                .map_err(|e| {
                    println!("PowerPoint parse error: {}", e);
                    format!("Failed to parse PowerPoint: {}", e)
                })?;

            let masked_content = masking_engine::mask_value(&content, &active_rules, &mut mapping, &mut counter);

            // Output as .txt file instead of .pptx for simplicity
            let txt_output = options.output_path.replace(".pptx", ".txt").replace(".ppt", ".txt");
            println!("Writing PowerPoint output to: {}", txt_output);
            file_parser::write_markdown(&txt_output, &masked_content)
                .map_err(|e| {
                    println!("PowerPoint write error: {}", e);
                    format!("Failed to write PowerPoint: {}", e)
                })?;

            if let Some(passphrase) = &options.passphrase {
                if let Some(ref map_path) = mapping_path {
                    let mappings: Vec<_> = mapping.values().cloned().collect();
                    crypto::save_encrypted_mapping(map_path, &mappings, passphrase)
                        .map_err(|e| format!("Failed to save mapping: {}", e))?;
                }
            }
        }
        file_parser::FileFormat::Markdown | file_parser::FileFormat::Text => {
            println!("Processing Markdown/Text file...");
            let content = file_parser::parse_markdown(&options.file_path)
                .map_err(|e| {
                    println!("Markdown/Text parse error: {}", e);
                    format!("Failed to read file: {}", e)
                })?;

            let masked_content = masking_engine::mask_value(&content, &active_rules, &mut mapping, &mut counter);

            println!("Writing Markdown/Text output...");
            file_parser::write_markdown(&options.output_path, &masked_content)
                .map_err(|e| {
                    println!("Markdown/Text write error: {}", e);
                    format!("Failed to write file: {}", e)
                })?;

            if let Some(passphrase) = &options.passphrase {
                if let Some(ref map_path) = mapping_path {
                    let mappings: Vec<_> = mapping.values().cloned().collect();
                    crypto::save_encrypted_mapping(map_path, &mappings, passphrase)
                        .map_err(|e| format!("Failed to save mapping: {}", e))?;
                }
            }
        }
        _ => {
            println!("Unsupported file format: {:?}", format);
            return Err("Unsupported file format".to_string());
        }
    }

    println!("=== MASK_FILE SUCCESS ===");
    println!("Masked count: {}", counter);
    println!("Output path: {}", options.output_path);

    Ok(MaskResult {
        output_path: options.output_path,
        masked_count: counter,
        mapping_path,
    })
}

#[tauri::command]
pub async fn preview_masking(options: PreviewOptions) -> Result<PreviewResult, String> {
    let format = file_parser::detect_format(&options.file_path);
    let max_rows = options.max_rows.unwrap_or(10);
    
    // Get rules by IDs
    let all_rules = masking_engine::get_builtin_rules();
    let active_rules: Vec<_> = all_rules
        .iter()
        .filter(|r| options.rule_ids.contains(&r.id))
        .cloned()
        .collect();
    
    // Create NER detector
    let ner_detector = ner::NERDetector::new();

    match format {
        file_parser::FileFormat::Csv => {
            let (headers, rows) = file_parser::parse_csv(&options.file_path)
                .map_err(|e| format!("Failed to parse CSV: {}", e))?;

            let preview_rows: Vec<_> = rows.into_iter().take(max_rows).collect();
            let mut mapping = std::collections::HashMap::new();
            let mut counter = 0usize;

            let masked_rows: Vec<Vec<String>> = preview_rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| masking_engine::mask_value(cell, &active_rules, &mut mapping, &mut counter))
                        .collect()
                })
                .collect();

            // Detect entities in original rows
            let detected_entities = Some(ner_detector.detect_in_rows(&preview_rows));

            Ok(PreviewResult {
                original_rows: preview_rows,
                masked_rows,
                headers,
                detected_entities,
            })
        }
        file_parser::FileFormat::Excel => {
            let (headers, rows) = file_parser::parse_excel(&options.file_path)
                .map_err(|e| format!("Failed to parse Excel: {}", e))?;

            let preview_rows: Vec<_> = rows.into_iter().take(max_rows).collect();
            let mut mapping = std::collections::HashMap::new();
            let mut counter = 0usize;

            let masked_rows: Vec<Vec<String>> = preview_rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| masking_engine::mask_value(cell, &active_rules, &mut mapping, &mut counter))
                        .collect()
                })
                .collect();

            // Detect entities in original rows
            let detected_entities = Some(ner_detector.detect_in_rows(&preview_rows));

            Ok(PreviewResult {
                original_rows: preview_rows,
                masked_rows,
                headers,
                detected_entities,
            })
        }
        file_parser::FileFormat::Word => {
            // 读取 Word 文档内容
            let content = file_parser::parse_word(&options.file_path)
                .map_err(|e| format!("Failed to parse Word: {}", e))?;
            
            // 按行分割
            let lines: Vec<String> = content.lines()
                .map(|s| s.to_string())
                .collect();
            
            let mut mapping = std::collections::HashMap::new();
            let mut counter = 0usize;
            
            // 对每行进行脱敏
            let masked_lines: Vec<String> = lines
                .iter()
                .map(|line| masking_engine::mask_value(line, &active_rules, &mut mapping, &mut counter))
                .collect();
            
            // 将文本行转换为表格格式（单列）
            let original_rows: Vec<Vec<String>> = lines.iter()
                .map(|line| vec![line.clone()])
                .collect();
            
            let masked_rows: Vec<Vec<String>> = masked_lines.iter()
                .map(|line| vec![line.clone()])
                .collect();
            
            // Detect entities in original rows
            let detected_entities = Some(ner_detector.detect_in_rows(&original_rows));
            
            Ok(PreviewResult {
                original_rows,
                masked_rows,
                headers: vec!["内容".to_string()],
                detected_entities,
            })
        }
        file_parser::FileFormat::PowerPoint => {
            // 读取 PowerPoint 内容
            let content = file_parser::parse_powerpoint(&options.file_path)
                .map_err(|e| format!("Failed to parse PowerPoint: {}", e))?;
            
            // 按行分割
            let lines: Vec<String> = content.lines()
                .map(|s| s.to_string())
                .collect();
            
            let mut mapping = std::collections::HashMap::new();
            let mut counter = 0usize;
            
            // 对每行进行脱敏
            let masked_lines: Vec<String> = lines
                .iter()
                .map(|line| masking_engine::mask_value(line, &active_rules, &mut mapping, &mut counter))
                .collect();
            
            // 将文本行转换为表格格式（单列）
            let original_rows: Vec<Vec<String>> = lines.iter()
                .map(|line| vec![line.clone()])
                .collect();
            
            let masked_rows: Vec<Vec<String>> = masked_lines.iter()
                .map(|line| vec![line.clone()])
                .collect();
            
            // Detect entities in original rows
            let detected_entities = Some(ner_detector.detect_in_rows(&original_rows));
            
            Ok(PreviewResult {
                original_rows,
                masked_rows,
                headers: vec!["内容".to_string()],
                detected_entities,
            })
        }
        file_parser::FileFormat::Pdf => {
            // 读取 PDF 内容
            let content = file_parser::parse_pdf(&options.file_path)
                .map_err(|e| format!("Failed to parse PDF: {}", e))?;
            
            // 按行分割
            let lines: Vec<String> = content.lines()
                .map(|s| s.to_string())
                .collect();
            
            let mut mapping = std::collections::HashMap::new();
            let mut counter = 0usize;
            
            // 对每行进行脱敏
            let masked_lines: Vec<String> = lines
                .iter()
                .map(|line| masking_engine::mask_value(line, &active_rules, &mut mapping, &mut counter))
                .collect();
            
            // 将文本行转换为表格格式（单列）
            let original_rows: Vec<Vec<String>> = lines.iter()
                .map(|line| vec![line.clone()])
                .collect();
            
            let masked_rows: Vec<Vec<String>> = masked_lines.iter()
                .map(|line| vec![line.clone()])
                .collect();
            
            // Detect entities in original rows
            let detected_entities = Some(ner_detector.detect_in_rows(&original_rows));
            
            Ok(PreviewResult {
                original_rows,
                masked_rows,
                headers: vec!["内容".to_string()],
                detected_entities,
            })
        }
        file_parser::FileFormat::Markdown | file_parser::FileFormat::Text => {
            // 读取文本内容
            let content = std::fs::read_to_string(&options.file_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            
            // 按行分割（读取全部内容，不限制行数）
            let lines: Vec<String> = content.lines()
                .map(|s| s.to_string())
                .collect();
            
            let mut mapping = std::collections::HashMap::new();
            let mut counter = 0usize;
            
            // 对每行进行脱敏
            let masked_lines: Vec<String> = lines
                .iter()
                .map(|line| masking_engine::mask_value(line, &active_rules, &mut mapping, &mut counter))
                .collect();
            
            // 将文本行转换为表格格式（单列）
            let original_rows: Vec<Vec<String>> = lines.iter()
                .map(|line| vec![line.clone()])
                .collect();
            
            let masked_rows: Vec<Vec<String>> = masked_lines.iter()
                .map(|line| vec![line.clone()])
                .collect();
            
            // Detect entities in original rows
            let detected_entities = Some(ner_detector.detect_in_rows(&original_rows));
            
            Ok(PreviewResult {
                original_rows,
                masked_rows,
                headers: vec!["内容".to_string()],
                detected_entities,
            })
        }
        _ => Err("预览功能目前支持 CSV、Excel、Word、PowerPoint、PDF、Markdown 和 TXT 文件".to_string()),
    }
}
