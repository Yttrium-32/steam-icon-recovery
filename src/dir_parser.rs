use anyhow::{Context, bail};
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::extractors::{extract_game_id, extract_icon_id};
use crate::icon_downloader::download_icon;

pub fn parse_dir(dir_path: PathBuf) -> anyhow::Result<()> {
    let entries = read_dir(&dir_path).with_context(|| {
        format!(
            "{} doesn't exist or lacks read permissions",
            dir_path.display()
        )
    })?;

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
    let mut lines = Vec::new();

    let mut game_id: Option<String> = None;

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
            } else {
                // Keep [Desktop Entry] header
                lines.push(val.clone());
            }
        }
        Err(e) => {
            eprintln!("Failed to read first line: {e}\nSkipping file...\n");
            return Ok(());
        }
    }

    for (i, line) in reader.enumerate() {
        let line = match line {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to read line {i}: {e}");
                continue;
            }
        };

        // Keep track of all lines for writing later
        lines.push(line.clone());

        if let Some((key, value)) = line.split_once("=") {
            let key = key.trim();
            let value = value.trim();

            if key == "Exec" && game_id.is_none() {
                game_id = extract_game_id(value).map(|v| v.to_string());
                if game_id.is_none() {
                    eprintln!("No game ID found, skipping file...\n");
                    return Ok(());
                }
            }

            if key == "Icon" && value != "steam" {
                eprintln!("Icon already exists, skipping...\n");
                return Ok(());
            }
        } else {
            eprintln!("Line number {i} might be malformed, failed to parse");
        }
    }

    // Game ID is guaranteed to be valid if an Exec field exists
    let game_id = game_id.context("Game ID not found, is there a valid `Exec` field?")?;

    let icon_id = extract_icon_id(&game_id, false)?;
    println!("Found icon id: {}", &icon_id);

    let icon_name = format!("steam_icon_{game_id}");

    let url = format!(
        "https://cdn.steamstatic.com/steamcommunity/public/images/apps/{game_id}/{icon_id}.ico"
    );
    println!("Icon url: {url}");

    download_icon(&url, &icon_name)?;

    println!();
    Ok(())
}
