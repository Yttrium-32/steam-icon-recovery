use anyhow::{Context, bail};
use std::path::PathBuf;
use std::io::{BufRead, BufReader};
use std::fs::{read_dir, File};

use crate::extractors::{extract_game_id, extract_icon_id};
use crate::icon_downloader::download_icon;

pub fn parse_dir(dir_path: PathBuf) -> anyhow::Result<()> {
    let entries = read_dir(&dir_path)
        .with_context(|| format!("{} doesn't exist or lacks read permissions", dir_path.display()))?;

    for entry in entries {
        match entry {
            Ok(entry) => {
                if let Err(e) = recover_icon_for_file(&entry.path()) {
                    eprintln!("Error processing {}: {e}", entry.path().display());
                }
            }
            Err(e) => {
                eprintln!("Failed to read path: {e}");
                continue;
            }
        };
    }
    Ok(())
}

pub fn recover_icon_for_file(file_entry: &PathBuf) -> anyhow::Result<()> {
    if !file_entry.is_file() {
        bail!("Not a file, skipping...\n");
    }

    let file_handle = File::open(file_entry)?;
    let mut reader = BufReader::new(file_handle).lines();

    println!("Processing `{}` :", file_entry.display());

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
            if val.trim() != "[Desktop Entry]" {
                eprintln!("No `[Desktop Entry]` section found, skipping file...\n");
                return Ok(());
            }
        }
        Err(e) => {
            eprintln!("Failed to read first line: {e}\nSkipping file...\n");
            return Ok(());
        }
    }

    let mut icon_exists = false;
    let mut game_id: Option<String> = None;

    'line_iter: for (i, line) in reader.enumerate() {
        let line = match line {
            Ok(val) => {
                val
            }
            Err(e) => {
                eprintln!("Failed to read line {i}: {e}");
                continue 'line_iter;
            }
        };

        if let Some((key, value)) = line.split_once("=") {
            let key = key.trim();
            let value = value.trim();

            if key == "Exec" {
                game_id = match extract_game_id(value) {
                    Some(val) => Some(val),
                    None => {
                        eprintln!("No game id found!");
                        break 'line_iter;
                    }
                };
            }

            if key == "Icon" {
                if value != "steam" {
                    println!("Icon already exists, skipping...");
                    icon_exists = true;
                }
                break 'line_iter;
            }
        } else {
            eprintln!("Line number {i} might be malformed, failed to parse");
            continue 'line_iter;
        }
    }


    if !icon_exists {
        if let Some(game_id) = game_id {
            let icon_id = extract_icon_id(&game_id, true)?;
            println!("Found icon id: {}", &icon_id);

            let icon_name = format!("steam_icon_{game_id}");

            let url = format!("https://cdn.steamstatic.com/steamcommunity/public/images/apps/{game_id}/{icon_id}.ico");
            println!("Icon url: {url}");

            download_icon(&url, &icon_name)?;
        }
    }

    println!();
    Ok(())
}
