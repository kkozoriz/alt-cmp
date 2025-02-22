use crate::PackageParser;
use std::collections::HashMap;
use std::error::Error;

pub struct AltLinuxParser;

impl PackageParser for AltLinuxParser {
    fn parse(data: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut packages = HashMap::new();

        for line in data.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let version = parts[1].split('-')
                    .filter(|part| !part.starts_with("alt"))
                    .collect::<Vec<&str>>()
                    .join("-");

                packages.insert(name, version);
            }
        }
        Ok(packages)
    }
}