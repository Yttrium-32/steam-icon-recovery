use std::process::Command;
use regex::Regex;
use anyhow::{Context, bail};

#[inline]
pub fn extract_game_id(exec_field: &str) -> Option<String> {
    let game_id_regex: Regex = Regex::new(r"steam steam://rungameid/([0-9]+)").unwrap();

    if let Some(capture) = game_id_regex.captures(exec_field) {
        return Some(capture[1].to_owned());
    }
    None
}

#[inline]
pub fn extract_icon_id(game_id: &String, is_dummy: bool) -> anyhow::Result<String> {
    let game_id_regex: Regex = Regex::new(r#""clienticon"\s+"([^"]+)""#).unwrap();

    if is_dummy {
        // Return a dummy string for testing
        return Ok("9102f4v4h3491h8hf4c1u2394n184n1th4".to_string());
    }

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
