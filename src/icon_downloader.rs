use std::collections::HashMap;
use std::env;
use std::fs::{File, read_dir};
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

use anyhow::{Context, bail};
use ico::{IconDir, IconDirEntry};
use reqwest::blocking::get;

pub fn download_icon(url: &String, icon_name: &String) -> anyhow::Result<()> {
    let user_home = env::var("HOME").context("HOME not set")?;
    let icon_dir_path = PathBuf::from(&user_home).join(".local/share/icons/hicolor/");
    let cache_dir = PathBuf::from(user_home).join(".cache/");

    let ico_path = cache_dir.join(format!("{icon_name}.ico"));

    // Avoid downloading icon file if it already exists
    let mut dest: File;
    if !ico_path.is_file() {
        dest = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&ico_path)
            .with_context(|| format!("Failed to create file at {}", ico_path.display()))?;

        let mut response =
            get(url).with_context(|| format!("Failed to send GET request to {url}"))?;

        response
            .copy_to(&mut dest)
            .with_context(|| format!("Failed to write response to file at {:?}", dest))?;

        dest.seek(SeekFrom::Start(0))?;
    } else {
        dest = File::open(&ico_path)
            .with_context(|| format!("Failed to open icon {}", ico_path.display()))?;
    }

    let icon_dir =
        IconDir::read(&dest).with_context(|| format!("Read icon failed for {:?}", dest))?;

    if let Ok(resolution_paths) = get_resolutions(&icon_dir_path) {
        for entry in icon_dir.entries().iter() {
            process_icon_entry(entry, &dest, icon_name, &resolution_paths)?;
        }
    } else {
        bail!(format!(
            "Failed to get resolution paths at {}!",
            icon_dir_path.display()
        ));
    }

    Ok(())
}

fn process_icon_entry(
    icon_entry: &IconDirEntry,
    icon_file_instance: &File,
    icon_name: &String,
    resolution_paths: &HashMap<u32, PathBuf>,
) -> anyhow::Result<()> {
    let image = icon_entry.decode().with_context(|| {
        format!(
            "{:?} in {:?} might be malformed",
            icon_entry, icon_file_instance
        )
    })?;

    let cur_resolution = &resolution_paths[&icon_entry.width()];
    let png_icon_path = cur_resolution.join(icon_name);

    println!("Icon Path: {}", png_icon_path.display());

    // Don't overwrite icon if it already exists
    if png_icon_path.is_file() {
        println!(
            "Icon {} already exists, skipping...",
            png_icon_path.display()
        );
        return Ok(());
    }

    let png_file = File::create(&png_icon_path)
        .with_context(|| format!("Failed to create file {}", png_icon_path.display()))?;

    image
        .write_png(png_file)
        .with_context(|| format!("Failed to write png file {}", png_icon_path.display()))?;

    Ok(())
}

fn get_resolutions(icon_dir_path: &PathBuf) -> anyhow::Result<HashMap<u32, PathBuf>> {
    let mut resolution_vec = HashMap::new();
    let paths = read_dir(icon_dir_path).with_context(|| {
        format!(
            "{} doesn't exist or lacks read permissions",
            icon_dir_path.display()
        )
    })?;

    for entry in paths {
        match entry {
            Ok(entry) => {
                for component in entry.path().components() {
                    if let Some(segment) = component.as_os_str().to_str() {
                        if let Some((resolution, _)) = segment.split_once("x") {
                            resolution_vec
                                .insert(resolution.parse()?, entry.path().to_owned().clone());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read path: {e}");
                continue;
            }
        };
    }
    Ok(resolution_vec)
}
