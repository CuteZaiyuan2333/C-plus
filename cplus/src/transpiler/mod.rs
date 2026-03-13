pub mod lexer;
pub mod ast;
pub mod parser;
pub mod codegen;
pub mod checker;

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use crate::transpiler::parser::Parser;
use crate::transpiler::codegen::Generator;

pub struct Transpiler {
    project_root: PathBuf,
    temp_dir: PathBuf,
}

impl Transpiler {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root: project_root.clone(),
            temp_dir: project_root.join(".temp"),
        }
    }

    pub fn transpile(&self) -> Result<()> {
        println!("Transpiling C+ project...");
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir)?;
        }
        fs::create_dir_all(&self.temp_dir)?;
        self.process_directory(&self.project_root)?;
        Ok(())
    }

    fn process_directory(&self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default();
                if dir_name == ".temp" || dir_name == "build" || dir_name == ".git" {
                    continue;
                }
                self.process_directory(&path)?;
            } else {
                let extension = path.extension().and_then(|s| s.to_str());
                match extension {
                    Some("cp") | Some("cph") => self.transpile_file(&path)?,
                    Some("c") | Some("h") => self.copy_to_temp(&path)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn transpile_file(&self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).map_err(|e| anyhow::anyhow!("Failed to read source file: {}", e))?;
        let relative_path = path.strip_prefix(&self.project_root).unwrap_or(path);
        let target_path = self.temp_dir.join(relative_path).with_extension("c");

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| anyhow::anyhow!("Failed to create build directory: {}", e))?;
        }

        // --- 1. Parser ---
        println!("  Parsing...");
        let mut parser = Parser::new(&content);
        let ast = parser.parse();
        
        // --- 2. Checker (所有权检查) ---
        println!("  Checking...");
        let mut checker = crate::transpiler::checker::OwnershipChecker::new();
        if let Err(e) = checker.check(&ast) {
            self.report_error(path, &content, &e);
            return Err(anyhow::anyhow!("Stopping due to ownership error."));
        }

        // --- 3. Generator ---
        println!("  Generating...");
        let mut generator = Generator::new();
        let translated = generator.generate(&ast);

        fs::write(target_path, translated).map_err(|e| anyhow::anyhow!("Failed to write translated file: {}", e))?;
        println!("  Done.");
        Ok(())
    }

    fn report_error(&self, path: &Path, content: &str, err_msg: &str) {
        // 尝试解析错误信息中的行号和列号，例如 "Ownership Error in ...: [15:5] Use of moved value: 'd1'"
        // 我们寻找最后的 [，但要确保它后面跟着数字和冒号
        let mut best_pos = None;
        for (idx, c) in err_msg.char_indices().rev() {
            if c == '[' {
                let remaining = &err_msg[idx + 1..];
                if let Some(end_bracket) = remaining.find(']') {
                    let pos_str = &remaining[..end_bracket];
                    let parts: Vec<&str> = pos_str.split(':').collect();
                    if parts.len() == 2 {
                        if parts[0].chars().all(|c| c.is_ascii_digit()) && parts[1].chars().all(|c| c.is_ascii_digit()) {
                            if let (Ok(line_idx), Ok(col_idx)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                                best_pos = Some((idx, line_idx, col_idx, end_bracket));
                                break;
                            }
                        }
                    }
                }
            }
        }

        if let Some((idx, line_idx, col_idx, end_bracket)) = best_pos {
            let remaining = &err_msg[idx + 1..];
            let lines: Vec<&str> = content.lines().collect();
            if line_idx > 0 && line_idx <= lines.len() {
                let source_line = lines[line_idx - 1];
                let msg = &remaining[end_bracket + 1..].trim();
                
                println!("\n\x1b[1;31mError\x1b[0m in {}:{}:{}", path.display(), line_idx, col_idx);
                println!(" {:5} | {}", line_idx, source_line);
                
                let padding = " ".repeat(7 + col_idx); 
                println!("{} \x1b[1;31m^\x1b[0m", padding);
                println!("{} \x1b[1;31m{}\x1b[0m\n", padding, msg);
                use std::io::Write;
                std::io::stdout().flush().unwrap();
                return;
            }
        }
        // 回退到原始错误打印
        println!("\x1b[1;31mError\x1b[0m in {}: {}", path.display(), err_msg);
    }

    fn copy_to_temp(&self, path: &Path) -> Result<()> {
        let relative_path = path.strip_prefix(&self.project_root)?;
        let target_path = self.temp_dir.join(relative_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(path, target_path)?;
        Ok(())
    }
}
