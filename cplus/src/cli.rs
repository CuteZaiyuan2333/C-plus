use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cplus")]
#[command(about = "C+ Language Transpiler", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build the C+ project
    Build {
        /// Build in debug mode
        #[arg(long)]
        debug: bool,
    },
    /// Build and run the C+ project
    Run {
        /// Build in debug mode
        #[arg(long)]
        debug: bool,
    },
    /// Initialize a new C+ project
    Init {
        /// The name of the project
        name: Option<String>,
    },
}
