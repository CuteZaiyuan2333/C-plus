use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, anyhow};
use std::fs;
use crate::config::{Config, Dependency};

pub struct Builder {
    project_root: PathBuf,
    temp_dir: PathBuf,
    config: Config,
}

impl Builder {
    pub fn new(project_root: PathBuf, config: Config) -> Self {
        Self {
            project_root: project_root.clone(),
            temp_dir: project_root.join(".temp"),
            config,
        }
    }

    pub fn build(&self, debug: bool) -> Result<PathBuf> {
        let profile_name = if debug { "debug" } else { "release" };
        println!("Building project '{}' in {} mode...", self.config.package.name, profile_name);

        let output_dir = self.project_root.join("build").join(profile_name);
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir)?;
        }

        let mut sources = Vec::new();
        self.collect_c_files(&self.temp_dir, &mut sources)?;

        if sources.is_empty() {
            return Err(anyhow!("No source files found in .temp directory. Check your src/ folder."));
        }

        let exe_name = if cfg!(windows) {
            format!("{}.exe", self.config.package.name)
        } else {
            self.config.package.name.clone()
        };
        let output_exe = output_dir.join(exe_name);

        let mut command = Command::new("gcc");
        command.args(&sources)
            .arg("-o")
            .arg(&output_exe);

        // --- 应用配置中的编译标志 ---
        // 1. 全局标志
        command.args(&self.config.build.flags);
        
        // 2. 包含路径
        command.arg(format!("-I{}", self.temp_dir.display()));
        for inc in &self.config.build.includes {
            command.arg(format!("-I{}", self.project_root.join(inc).display()));
        }

        // 3. 库路径
        for dir in &self.config.build.lib_dirs {
            command.arg(format!("-L{}", self.project_root.join(dir).display()));
        }

        // 4. 链接库
        for lib in &self.config.build.libs {
            command.arg(format!("-l{}", lib));
        }

        // 5. Profile 标志
        if let Some(profile) = self.config.profile.get(profile_name) {
            command.args(&profile.flags);
            if let Some(opt) = profile.opt_level {
                command.arg(format!("-O{}", opt));
            }
        }

        // 6. 依赖项 (pkg-config)
        for (name, dep) in &self.config.dependencies {
            if let Dependency::System { system: true, .. } = dep {
                if let Ok(output) = Command::new("pkg-config").args(&["--cflags", "--libs", name]).output() {
                    if output.status.success() {
                        let extra_args = String::from_utf8_lossy(&output.stdout);
                        for arg in extra_args.split_whitespace() {
                            command.arg(arg);
                        }
                    }
                }
            }
        }

        // 默认调试/发布参数 (Fallback)
        if debug && !command.get_args().any(|a| a == "-g") {
            command.arg("-g");
        }

        let status = command.status()
            .map_err(|e| anyhow!("Failed to execute gcc: {}. Check if gcc is in PATH.", e))?;

        if status.success() {
            println!("Build successful at {:?}", output_exe);
            Ok(output_exe)
        } else {
            Err(anyhow!("Compilation failed."))
        }
    }

    fn collect_c_files(&self, dir: &Path, sources: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.collect_c_files(&path, sources)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("c") {
                sources.push(path);
            }
        }
        Ok(())
    }
}
