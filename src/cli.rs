use clap::Parser;
use std::{env, path::PathBuf};

/// Steam shortcuts icon recovery tool for linux
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Optional file to process, overrides `dir`
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Directory to parse files from, defaults to $HOME/.local/share/applications
    #[arg(short, long, value_name = "DIR")]
    pub dir: Option<PathBuf>,
}

impl Cli {
    pub fn get_dir(self) -> PathBuf {
        self.dir.unwrap_or_else(|| {
            let user_home = env::var("HOME").expect("HOME not set");
            PathBuf::from(user_home).join(".local/share/applications/")
        })
    }
}
