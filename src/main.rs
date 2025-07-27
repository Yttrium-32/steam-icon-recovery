use std::env;
use std::path::PathBuf;
use std::fs::{read_dir, File, DirEntry};
use std::io::{BufReader, BufRead};
use std::process::Command;
use anyhow::{Context, bail};
use regex::Regex;

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
    if !desktop_file.path().is_file() {
        bail!("{} is not a file, skipping...", desktop_file.path().display());
    }

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
            let key = key.trim();
            let value = value.trim();
            if key == "Exec" {
                let game_id = match extract_game_id(value) {
                    Some(val) => val,
                    None => {
                        eprintln!("No game id found!");
                        break;
                    }
                };
                println!("Found game id: {game_id}");
            }
        } else {
            eprintln!("Failed to split at `=` in `{line}`, checking next line...");
            continue;
        }
    }
    println!();

    Ok(())
}

pub fn extract_game_id(exec_field: &str) -> Option<String> {
    let game_id_regex: Regex = Regex::new(r"steam steam://rungameid/([0-9]+)").unwrap();

    if let Some(capture) = game_id_regex.captures(exec_field) {
        return Some(capture[1].to_owned());
    }
    None
}

#[inline]
pub fn extract_icon_id(game_id: String) -> Option<String> {
    let game_id_regex: Regex = Regex::new(r#""clienticon"\s+"([^"]+)""#).unwrap();

    let cmd = Command::new("steamcmd")
        .arg("+login")
        .arg("anonymous")
        .arg("+app_info_print")
        .arg(game_id)
        .arg("+quit")
        .output()
        .expect("Failed to execute `steamcmd`");

    let cmd_output = String::from_utf8_lossy(&cmd.stdout);
    if let Some(capture) = game_id_regex.captures(&cmd_output) {
        return Some(capture[1].to_owned());
    }
    None
}
