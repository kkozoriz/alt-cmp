use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use xz2::read::XzDecoder;

pub type PackageMapping = HashMap<String, String>;

struct FetchProgress {
    pb: ProgressBar,
}

impl FetchProgress {
    fn new(label: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.cyan} Fetching {msg} packages [{elapsed_precise}]")
                .expect("Failed to set progress style"),
        );
        pb.set_message(label.to_string());
        Self { pb }
    }

    fn finish(&self, label: &str) {
        self.pb
            .finish_with_message(format!("{} packages fetched", label.green()));
    }
}

pub fn fetch_package_list(
    client: &Client,
    url: &str,
    label: &str,
) -> Result<String, Box<dyn Error>> {
    let progress = FetchProgress::new(label);
    let response = client.get(url).send()?;
    let bytes = response.bytes()?;
    let mut decompressed = String::new();

    if url.ends_with(".xz") {
        XzDecoder::new(bytes.as_ref()).read_to_string(&mut decompressed)?;
    } else {
        decompressed = String::from_utf8(bytes.to_vec())?;
    }

    progress.finish(label);
    Ok(decompressed)
}

pub fn read_package_mapping(file_path: &PathBuf) -> Result<PackageMapping, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mapping = reader
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() >= 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    Ok(mapping)
}

pub fn save_to_file(data: &str, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    File::create(path)?.write_all(data.as_bytes())?;

    Ok(())
}
