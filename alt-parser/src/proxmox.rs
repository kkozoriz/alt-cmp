use crate::PackageParser;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;

pub struct ProxmoxParser;

impl PackageParser for ProxmoxParser {
    fn parse(data: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut packages = HashMap::new();
        let version_re = Regex::new(r"Version: (.+)")?;
        let package_re = Regex::new(r"Package: (.+)")?;
        let mut current_package = String::new();

        for line in data.lines() {
            if let Some(captures) = package_re.captures(line) {
                current_package = captures[1].to_string();
            } else if let Some(captures) = version_re.captures(line) {
                if !current_package.is_empty() {
                    let version = captures[1].to_string();
                    let clean_version = clean_version_string(&version);

                    packages.insert(current_package.clone(), clean_version);
                }
            }
        }

        Ok(packages)
    }
}

fn clean_version_string(version: &str) -> String {
    let base_version = version.split('-').next().unwrap_or(version);
    let parts: Vec<&str> = base_version.split('+').collect();

    if parts.len() > 1 {
        parts[0].to_string()
    } else {
        base_version.to_string()
    }
}
