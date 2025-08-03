use std::fs::File;
use std::io::{Seek, SeekFrom};

use anyhow::Context;
use ico::{IconDir, IconDirEntry};
use reqwest::blocking::get;

pub fn download_icon(url: &String, icon_name: &String) -> anyhow::Result<()> {
    let ico_path = format!("{icon_name}.ico");

    let mut dest = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&ico_path)
        .with_context(|| format!("Failed to create file at {ico_path}"))?;

    let mut response = get(url).with_context(|| format!("Failed to send GET request to {url}"))?;

    response
        .copy_to(&mut dest)
        .with_context(|| format!("Failed to write response to file at {:?}", dest))?;

    dest.seek(SeekFrom::Start(0))?;

    let icon_dir =
        IconDir::read(&dest).with_context(|| format!("Read icon failed for {:?}", dest))?;

    for entry in icon_dir.entries().iter() {
        process_icon_entry(entry, &dest, icon_name)?;
    }

    Ok(())
}

fn process_icon_entry(
    icon_entry: &IconDirEntry,
    icon_file_instance: &File,
    icon_name: &String,
) -> anyhow::Result<()> {
    let image = icon_entry.decode().with_context(|| {
        format!(
            "{:?} in {:?} might be malformed",
            icon_entry, icon_file_instance
        )
    })?;

    let png_icon_name = format!(
        "{icon_name}_{}x{}.png",
        icon_entry.width(),
        icon_entry.height()
    );

    let png_file = File::create(&png_icon_name)
        .with_context(|| format!("Failed to create file {}", png_icon_name))?;

    image
        .write_png(png_file)
        .with_context(|| format!("Failed to write png file {}", png_icon_name))?;

    Ok(())
}
