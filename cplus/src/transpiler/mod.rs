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
        let content = fs::read_to_string(path)?;
        let relative_path = path.strip_prefix(&self.project_root)?;
        let target_path = self.temp_dir.join(relative_path).with_extension("c");

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // --- 1. Parser ---
        println!("  Parsing...");
        let mut parser = Parser::new(&content);
        let ast = parser.parse();
        
        // --- 2. Checker (所有权检查) ---
        println!("  Checking...");
        let mut checker = crate::transpiler::checker::OwnershipChecker::new();
        if let Err(e) = checker.check(&ast) {
            return Err(anyhow::anyhow!("Ownership Error in {:?}: {}", path, e));
        }

        // --- 3. Generator ---
        println!("  Generating...");
        let mut generator = Generator::new();
        let translated = generator.generate(&ast);

        fs::write(target_path, translated)?;
        println!("  Done.");
        Ok(())
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
