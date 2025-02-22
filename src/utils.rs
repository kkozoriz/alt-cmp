use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use xz2::read::XzDecoder;
use colored::Colorize;

pub async fn fetch_package_list(client: &Client, url: &str, label: &str) -> Result<String, Box<dyn Error>> {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} Fetching {msg} packages [{elapsed_precise}]")?
    );
    pb.set_message(label.to_string());

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    let mut decompressed = String::new();

    if url.ends_with(".xz") {
        let mut decoder = XzDecoder::new(bytes.as_ref());
        decoder.read_to_string(&mut decompressed)?;
    } else {
        decompressed = String::from_utf8(bytes.to_vec())?;
    }

    pb.finish_with_message(format!("{} packages fetched", label.green()));
    Ok(decompressed)
}

pub fn read_package_mapping(file_path: &PathBuf) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut mapping = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 2 {
            mapping.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    Ok(mapping)
}

pub fn save_to_file(data: &str, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;

    file.write_all(data.as_bytes())?;
    Ok(())
}