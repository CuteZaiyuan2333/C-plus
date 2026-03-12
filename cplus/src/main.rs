mod cli;
mod transpiler;
mod builder;
mod config;

use clap::Parser;
use cli::{Cli, Commands};
use transpiler::Transpiler;
use builder::Builder;
use config::Config;
use std::{env, fs};
use anyhow::Result;

use std::process::Command;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let current_dir = env::current_dir()?;

    match cli.command {
        Commands::Build { debug } => {
            build_project(&current_dir, debug)?;
        }
        Commands::Run { debug } => {
            let exe_path = build_project(&current_dir, debug)?;
            println!("Running project...");
            let mut run = Command::new(exe_path).spawn()?;
            run.wait()?;
        }
        Commands::Init { name } => {
            let project_name = name.unwrap_or_else(|| "new_project".to_string());
            let project_path = current_dir.join(&project_name);
            fs::create_dir_all(project_path.join("src"))?;
            
            // 生成默认 cplus.toml
            let toml_content = format!(
r#"[package]
name = "{}"
version = "0.1.0"
type = "bin"

[build]
flags = ["-Wall"]

[profile.debug]
flags = ["-g"]
opt_level = 0

[profile.release]
opt_level = 3
"#, project_name);
            fs::write(project_path.join("cplus.toml"), toml_content)?;

            fs::write(project_path.join("src").join("main.cp"), 
                "struct demo { int a = 1; }\nbind demo { demo(n){ host.a = n; } }\nint main() { let demo d.demo(10); printf(\"Hello C+\\n\"); return 0; }")?;
            println!("Initialized project: {}", project_name);
        }
    }
    Ok(())
}

fn build_project(root: &std::path::PathBuf, debug: bool) -> Result<std::path::PathBuf> {
    let config = Config::load(root)?;
    let transpiler = Transpiler::new(root.clone());
    transpiler.transpile()?;
    let builder = Builder::new(root.clone(), config);
    builder.build(debug)
}
