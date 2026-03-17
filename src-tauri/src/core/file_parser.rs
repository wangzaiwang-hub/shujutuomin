use anyhow::{Result, Context};
use std::fs;
use std::io::Read;

#[derive(Debug)]
pub enum FileFormat {
    Csv,
    Excel,
    Json,
    Text,
    Word,
    PowerPoint,
    Markdown,
}

pub fn detect_format(path: &str) -> FileFormat {
    let lower = path.to_lowercase();
    if lower.ends_with(".csv") {
        FileFormat::Csv
    } else if lower.ends_with(".xlsx") || lower.ends_with(".xls") {
        FileFormat::Excel
    } else if lower.ends_with(".json") {
        FileFormat::Json
    } else if lower.ends_with(".docx") || lower.ends_with(".doc") {
        FileFormat::Word
    } else if lower.ends_with(".pptx") || lower.ends_with(".ppt") {
        FileFormat::PowerPoint
    } else if lower.ends_with(".md") || lower.ends_with(".markdown") {
        FileFormat::Markdown
    } else {
        FileFormat::Text
    }
}

pub fn parse_csv(path: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    // 首先尝试直接读取
    match try_parse_csv_direct(path) {
        Ok(result) => return Ok(result),
        Err(_) => {
            // 如果失败，尝试不同编码
            if let Ok(content) = parse_text_file(path) {
                return try_parse_csv_from_string(&content);
            }
        }
    }
    
    Err(anyhow::anyhow!("Failed to parse CSV file: {}", path))
}

fn try_parse_csv_direct(path: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();
    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result?;
        rows.push(record.iter().map(|s| s.to_string()).collect());
    }
    Ok((headers, rows))
}

fn try_parse_csv_from_string(content: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();
    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result?;
        rows.push(record.iter().map(|s| s.to_string()).collect());
    }
    Ok((headers, rows))
}

pub fn write_csv(path: &str, headers: &[String], rows: &[Vec<String>]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(headers)?;
    for row in rows {
        writer.write_record(row)?;
    }
    writer.flush()?;
    Ok(())
}

// Word document parsing (DOCX) - simplified version
// Extracts text content from DOCX using zip + XML parsing
pub fn parse_word(path: &str) -> Result<String> {
    use zip::ZipArchive;
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let file = fs::File::open(path).context("Failed to open Word file")?;
    let mut archive = ZipArchive::new(file).context("Failed to read DOCX archive")?;

    let mut text = String::new();

    // Try to read document.xml which contains the main content
    if let Ok(mut doc_file) = archive.by_name("word/document.xml") {
        let mut content = String::new();
        doc_file.read_to_string(&mut content)?;

        // Parse XML and extract text
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    if let Ok(txt) = e.unescape() {
                        text.push_str(&txt);
                        text.push(' ');
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("XML parse error: {}", e)),
                _ => {}
            }
            buf.clear();
        }
    }

    Ok(text)
}

pub fn write_word(path: &str, content: &str) -> Result<()> {
    // For simplicity, write as plain text with .docx extension
    // Users can open in Word and it will be imported as text
    fs::write(path, content).context("Failed to write Word file")?;
    Ok(())
}

// PowerPoint parsing (PPTX)
pub fn parse_powerpoint(path: &str) -> Result<String> {
    use zip::ZipArchive;
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let file = fs::File::open(path).context("Failed to open PPTX file")?;
    let mut archive = ZipArchive::new(file).context("Failed to read PPTX archive")?;

    let mut text = String::new();

    // Iterate through all slides
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Only process slide XML files
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            // Parse XML and extract text
            let mut reader = Reader::from_str(&content);
            reader.trim_text(true);

            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Text(e)) => {
                        if let Ok(txt) = e.unescape() {
                            text.push_str(&txt);
                            text.push(' ');
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(anyhow::anyhow!("XML parse error: {}", e)),
                    _ => {}
                }
                buf.clear();
            }
            text.push('\n');
        }
    }

    Ok(text)
}

pub fn write_powerpoint(path: &str, content: &str) -> Result<()> {
    // For now, write as text file with .pptx extension
    // Full PPTX generation would require complex XML structure
    fs::write(path, content).context("Failed to write PowerPoint file")?;
    Ok(())
}

// Markdown parsing
pub fn parse_markdown(path: &str) -> Result<String> {
    parse_text_file(path).context("Failed to read Markdown file")
}

pub fn write_markdown(path: &str, content: &str) -> Result<()> {
    fs::write(path, content).context("Failed to write Markdown file")
}

// 通用文本文件解析函数，支持多种编码
fn parse_text_file(path: &str) -> Result<String> {
    use std::path::Path;
    
    // 打印当前工作目录和文件路径信息
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    println!("Current working directory: {}", current_dir.display());
    println!("Attempting to read file: {}", path);
    
    // 标准化路径处理
    let file_path = Path::new(path);
    
    // 如果是绝对路径，直接使用
    let final_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        // 相对路径，尝试多种解析方式
        let possible_paths = vec![
            current_dir.join(path),                                    // 相对于当前目录
            current_dir.parent().unwrap_or(&current_dir).join(path),   // 相对于父目录（项目根目录）
        ];
        
        let mut found_path = None;
        for candidate in &possible_paths {
            println!("Trying path: {}", candidate.display());
            if candidate.exists() && candidate.is_file() {
                println!("Found valid file at: {}", candidate.display());
                found_path = Some(candidate.clone());
                break;
            }
        }
        
        match found_path {
            Some(path) => path,
            None => {
                let error_msg = format!(
                    "File not found: '{}'. Tried paths:\n{}",
                    path,
                    possible_paths.iter()
                        .map(|p| format!("  - {}", p.display()))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                return Err(anyhow::anyhow!("{}", error_msg));
            }
        }
    };

    println!("Using final path: {}", final_path.display());

    // 尝试读取文件，提供更详细的错误信息
    match std::fs::read_to_string(&final_path) {
        Ok(content) => {
            println!("Successfully read file: {} ({} bytes)", final_path.display(), content.len());
            Ok(content)
        },
        Err(e) => {
            println!("Failed to read file as UTF-8: {}, trying alternative encodings...", e);
            // 尝试以字节方式读取，然后转换为字符串
            match std::fs::read(&final_path) {
                Ok(bytes) => {
                    println!("Read {} bytes from file: {}", bytes.len(), final_path.display());
                    // 尝试不同的编码
                    if let Ok(content) = String::from_utf8(bytes.clone()) {
                        println!("Successfully decoded as UTF-8");
                        Ok(content)
                    } else {
                        println!("UTF-8 decode failed, trying GBK...");
                        // 尝试 GBK 编码（中文 Windows 常用）
                        match encoding_rs::GBK.decode(&bytes) {
                            (content, _, false) => {
                                println!("Successfully decoded as GBK");
                                Ok(content.into_owned())
                            },
                            _ => {
                                println!("GBK decode failed, using lossy UTF-8...");
                                // 如果都失败，使用 UTF-8 lossy 转换
                                Ok(String::from_utf8_lossy(&bytes).into_owned())
                            }
                        }
                    }
                }
                Err(read_err) => {
                    Err(anyhow::anyhow!(
                        "Failed to read file '{}': {} (original UTF-8 error: {})", 
                        final_path.display(), read_err, e
                    ))
                }
            }
        }
    }
}
