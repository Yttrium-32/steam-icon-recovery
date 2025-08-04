use anyhow::{Context, bail};
use regex::Regex;
use std::process::Command;

#[inline]
pub fn extract_game_id(exec_field: &str) -> Option<&str> {
    let game_id_regex: Regex = Regex::new(r"steam steam://rungameid/([0-9]+)").unwrap();

    game_id_regex
        .captures(exec_field)
        .and_then(|caps| caps.get(1).map(|m| m.as_str()))
}

#[inline]
pub fn extract_icon_id(game_id: &str, is_dummy: bool) -> anyhow::Result<String> {
    let icon_id_regex: Regex = Regex::new(r#""clienticon"\s+"([^"]+)""#).unwrap();

    if is_dummy {
        // Return a dummy string for testing
        return Ok("9102f4v4h3491h8hf4c1u2394n184n1th4".into());
    }

    let cmd = Command::new("steamcmd")
        .arg("+login")
        .arg("anonymous")
        .arg("+app_info_print")
        .arg(game_id)
        .arg("+quit")
        .output()
        .with_context(|| "Failed to execute `steamcmd`")?;

    if !cmd.status.success() {
        bail!("steamcmd failed with status: {}", cmd.status);
    }

    let cmd_output = std::str::from_utf8(&cmd.stdout)
        .context("`steamcmd` output is not valid UTF-8")?;

    if let Some(capture) = icon_id_regex.captures(&cmd_output) {
        Ok(capture[1].to_owned())
    } else {
        bail!("No icon id found! Something has gone wrong...");
    }
}
