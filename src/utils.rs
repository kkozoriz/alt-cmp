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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[test]
    fn test_read_package_mapping_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("mapping.txt");
        fs::write(&file_path, "pkg1 pkg2\npkg3 pkg4\ninvalid\n").unwrap();

        let result = read_package_mapping(&file_path).unwrap();
        let mut expected = HashMap::new();

        expected.insert("pkg1".to_string(), "pkg2".to_string());
        expected.insert("pkg3".to_string(), "pkg4".to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn test_read_package_mapping_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");
        File::create(&file_path).unwrap();

        let result = read_package_mapping(&file_path).unwrap();
        let expected: PackageMapping = HashMap::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_read_package_mapping_nonexistent_file() {
        let file_path = PathBuf::from("nonexistent.txt");
        let result = read_package_mapping(&file_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_save_to_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");
        let data = "test content";

        save_to_file(data, &file_path).unwrap();
        let mut file = File::open(&file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert_eq!(contents, "test content");
    }

    #[test]
    fn test_save_to_file_invalid_path() {
        let file_path = PathBuf::from("/invalid/path/output.txt");
        let result = save_to_file("test", &file_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_progress_initialization() {
        let progress = FetchProgress::new("test");
        assert_eq!(progress.pb.message(), "test");

        progress.finish("test");
    }
}
