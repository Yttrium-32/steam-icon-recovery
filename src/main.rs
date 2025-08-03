use clap::Parser;

mod cli;
use cli::Cli;

mod dir_parser;
use dir_parser::{recover_icon_for_file, parse_dir};

mod extractors;
mod icon_downloader;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(file_path) = cli.file {
        recover_icon_for_file(&file_path)?;
    } else {
        let dir = cli.get_dir();
        parse_dir(dir)?;
    }

    Ok(())
}
