use std::env;
use std::path::PathBuf;
use std::fs::{read_dir, File, DirEntry};
use std::io::{BufReader, BufRead};
use anyhow::{Context, bail};

fn main() -> anyhow::Result<()> {
    let user_home = env::var("HOME").context("HOME is not set!")?;
    let app_dir: PathBuf = [user_home + "/.local/share/applications/"].iter().collect();

    let paths = read_dir(&app_dir)
        .with_context(|| format!("{} doesn't exist or don't have read permissions", app_dir.display()))?;

    for path in paths {
        let entry = match path {
            Ok(entry) => {
                println!("Processing {:?}...", entry);
                entry
            }
            Err(e) => {
                eprintln!("Failed to read path: {e}");
                continue;
            }
        };
        recover_icon(&entry)?;
    }

    Ok(())
}

fn recover_icon(desktop_file: &DirEntry) -> anyhow::Result<()> {
    if desktop_file.path().is_file() {
        let file_handle = File::open(desktop_file.path())?;
        let reader = BufReader::new(file_handle);

        println!("{}:", desktop_file.path().display());
        for (i, line) in reader.lines().enumerate() {
            if i == 0 && line? != "[Desktop Entry]" {
                eprintln!("No `[Desktop Entry]` section found, skipping...");
                continue;
            }
        }
    } else {
        bail!("{} is not a file, skipping...", desktop_file.path().display());
    }

    Ok(())
}
