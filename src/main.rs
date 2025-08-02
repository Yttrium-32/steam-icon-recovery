use std::env;
use std::path::PathBuf;
use std::fs::{read_dir, File};
use std::io::{BufReader, BufRead};
use std::process::Command;

use anyhow::{Context, bail};
use regex::Regex;
use clap::Parser;

/// Steam shortcuts icon recovery tool for linux
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional file to process, overrides `dir`
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// Directory to parse files from, defaults to $HOME/.local/share/applications
    #[arg(short, long, value_name = "DIR")]
    dir: Option<PathBuf>
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(file_path) = cli.file {
        recover_icon_for_file(&file_path)?;
    } else {
        let dir = cli.dir.unwrap_or_else(|| {
            let user_home = env::var("HOME").expect("HOME not set");
            PathBuf::from(user_home).join(".local/share/applications/")
        });
        parse_dir(dir)?;
    }

    Ok(())
}

fn parse_dir(dir_path: PathBuf) -> anyhow::Result<()> {
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

fn recover_icon_for_file(file_entry: &PathBuf) -> anyhow::Result<()> {
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
            let icon_hash = extract_icon_id(&game_id, false)?;
            println!("Found icon id: {}", &icon_hash);

            let url = format!("https://cdn.steamstatic.com/steamcommunity/public/images/apps/{game_id}/{icon_hash}.ico");
            println!("Icon url: {url}");
        }
    }

    println!();
    Ok(())
}

#[inline]
fn extract_game_id(exec_field: &str) -> Option<String> {
    let game_id_regex: Regex = Regex::new(r"steam steam://rungameid/([0-9]+)").unwrap();

    if let Some(capture) = game_id_regex.captures(exec_field) {
        return Some(capture[1].to_owned());
    }
    None
}

#[inline]
fn extract_icon_id(game_id: String) -> Option<String> {
    let game_id_regex: Regex = Regex::new(r#""clienticon"\s+"([^"]+)""#).unwrap();

    let cmd = Command::new("steamcmd")
        .arg("+login")
        .arg("anonymous")
        .arg("+app_info_print")
        .arg(game_id)
        .arg("+quit")
        .output()
        .with_context(|| "Failed to execute `steamcmd`")?;

    let cmd_output = String::from_utf8_lossy(&cmd.stdout);
    if let Some(capture) = game_id_regex.captures(&cmd_output) {
        Ok(capture[1].to_owned())
    } else {
        bail!("No icon id found! Something has gone wrong...");
    }
}
