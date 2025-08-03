use std::fs::File;
use std::io::{Seek, SeekFrom};

use ico::IconDir;
use reqwest::blocking::get;
use anyhow::Context;

pub fn download_icon(url: &String, icon_name: &String) -> anyhow::Result<()>
{
    let ico_path = format!("{icon_name}.ico");

    let mut dest = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&ico_path)
        .with_context(|| format!("Failed to create file at {ico_path}"))?;

    let mut response = get(url)
        .with_context(|| format!("Failed to send GET request to {url}"))?;

    response.copy_to(&mut dest)
        .with_context(|| format!("Failed to write response to file at {:?}", dest))?;

    dest.seek(SeekFrom::Start(0))?;

    let icon_dir = IconDir::read(&dest)
        .with_context(|| format!("Read icon failed for {:?}", dest))?;

    for entry in icon_dir.entries().iter() {
        let image = entry.decode()
            .with_context(||
                format!("{:?} in {:?} might be malformed", entry, dest)
            )?;

        let png_icon_name = format!("{icon_name}_{}x{}.png", entry.width(), entry.height());

        let png_file = File::create(&png_icon_name)
            .with_context(|| format!("Failed to create file {}", png_icon_name))?;

        image.write_png(png_file)
            .with_context(|| format!("Failed to write png file {}", png_icon_name))?;
    }

    Ok(())
}
