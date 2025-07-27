use std::env;
use std::path::PathBuf;
use std::fs::{read_dir, File, DirEntry};
use std::io::{BufReader, BufRead};
use anyhow::{Context, bail};

fn main() -> anyhow::Result<()> {
    let user_home = env::var("HOME").context("HOME is not set!")?;
    let app_dir = PathBuf::from(user_home).join(".local/share/applications/");

    let paths = read_dir(&app_dir)
        .with_context(|| format!("{} doesn't exist or don't have read permissions", app_dir.display()))?;

    for path in paths {
        match path {
            Ok(entry) => {
                recover_icon(&entry)?;
            }
            Err(e) => {
                eprintln!("Failed to read path: {e}");
                continue;
            }
        };
    }

    Ok(())
}

fn recover_icon(desktop_file: &DirEntry) -> anyhow::Result<()> {
    if desktop_file.path().is_file() {
        let file_handle = File::open(desktop_file.path())?;
        let mut reader = BufReader::new(file_handle).lines();

        println!("Processing `{}` :", desktop_file.path().display());

        let first_line = match reader.next() {
            Some(val) => val,
            None => {
                eprintln!("File is empty, skipping...");
                return Ok(());
            }
        };

        // Before processing check if `[Desktop Entry]` header is at the
        // top of the file
        match first_line {
            Ok(val) => {
                if val != "[Desktop Entry]" {
                    eprintln!("No `[Desktop Entry]` section found, skipping file...\n");
                    return Ok(());
                }
            }
            Err(e) => {
                eprintln!("Failed to read first line: {e}\nSkipping file...");
                return Ok(());
            }
        }


        for (i, line) in reader.enumerate() {
            let line = match line {
                Ok(val) => {
                    val
                }
                Err(e) => {
                    eprintln!("Failed to read line {i}: {e}");
                    continue;
                }
            };

            if let Some((key, value)) = line.split_once("=") {
                let _key = key.trim();
                let _value = value.trim();
            } else {
                eprintln!("Failed to split at `=` in `{line}`, checking next line...");
                continue;
            }

            println!("{line}");
        }
    } else {
        bail!("{} is not a file, skipping file...", desktop_file.path().display());
    }
    println!();

    Ok(())
}
