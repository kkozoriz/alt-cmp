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

/// Progress indicator for package fetching operations
struct ProgressIndicator {
    progress_bar: ProgressBar,
}

impl ProgressIndicator {
    /// Creates a new spinner-style progress indicator with the given label
    fn new(label: &str) -> Self {
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.enable_steady_tick(Duration::from_millis(100));

        let spinner_style = ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} Fetching {msg} packages [{elapsed_precise}]")
            .expect("Failed to set progress style");

        progress_bar.set_style(spinner_style);
        progress_bar.set_message(label.to_string());

        Self { progress_bar }
    }

    /// Marks the progress as complete with a success message
    fn complete(&self, label: &str) {
        self.progress_bar
            .finish_with_message(format!("{} packages fetched", label.green()));
    }
}

/// NetworkClient handles remote resource fetching with progress indication
struct NetworkClient {
    client: Client,
}

impl NetworkClient {
    /// Creates a new network client
    fn new(client: &Client) -> Self {
        Self {
            client: client.clone(),
        }
    }

    /// Fetches data from a URL, showing progress and handling decompression if needed
    fn fetch_data(&self, url: &str, label: &str) -> Result<String, Box<dyn Error>> {
        let progress = ProgressIndicator::new(label);
        let response = self.client.get(url).send()?;
        let bytes = response.bytes()?;

        let result = if url.ends_with(".xz") {
            self.decompress_xz_data(&bytes)?
        } else {
            String::from_utf8(bytes.to_vec())?
        };

        progress.complete(label);
        Ok(result)
    }

    /// Decompresses XZ compressed data
    fn decompress_xz_data(&self, compressed_data: &[u8]) -> Result<String, Box<dyn Error>> {
        let mut decompressed = String::new();
        XzDecoder::new(compressed_data).read_to_string(&mut decompressed)?;
        Ok(decompressed)
    }
}

/// FileManager handles file operations for package mappings and data storage
struct FileManager;

impl FileManager {
    /// Reads a package mapping file and returns a HashMap of package mappings
    fn read_mapping(file_path: &PathBuf) -> Result<PackageMapping, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let mapping = reader
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| Self::parse_mapping_line(&line))
            .collect();

        Ok(mapping)
    }

    /// Parses a single line from a mapping file
    fn parse_mapping_line(line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    /// Saves string data to a file
    fn save_data(data: &str, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }
}

// Public API

/// Fetches a package list from a remote URL with progress indication
pub fn fetch_package_list(
    client: &Client,
    url: &str,
    label: &str,
) -> Result<String, Box<dyn Error>> {
    let network_client = NetworkClient::new(client);
    network_client.fetch_data(url, label)
}

/// Reads a package mapping file and returns a HashMap of package mappings
pub fn read_package_mapping(file_path: &PathBuf) -> Result<PackageMapping, Box<dyn Error>> {
    FileManager::read_mapping(file_path)
}

/// Saves string data to a file
pub fn save_to_file(data: &str, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    FileManager::save_data(data, path)
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
    fn test_progress_indicator_initialization() {
        let progress = ProgressIndicator::new("test");
        assert_eq!(progress.progress_bar.message(), "test");

        progress.complete("test");
    }

    #[test]
    fn test_file_manager_parse_mapping_line() {
        // Valid line with two parts
        assert_eq!(
            FileManager::parse_mapping_line("pkg1 pkg2"),
            Some(("pkg1".to_string(), "pkg2".to_string()))
        );

        // Valid line with more than two parts (uses first two)
        assert_eq!(
            FileManager::parse_mapping_line("pkg1 pkg2 extra"),
            Some(("pkg1".to_string(), "pkg2".to_string()))
        );

        // Invalid line with only one part
        assert_eq!(FileManager::parse_mapping_line("invalid"), None);

        // Empty line
        assert_eq!(FileManager::parse_mapping_line(""), None);
    }
}
