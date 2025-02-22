use crate::PackageParser;
use std::collections::HashMap;
use std::error::Error;
use regex::Regex;

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
                    let clean_version = version.split('-').next().unwrap_or(&version).to_string();

                    packages.insert(current_package.clone(), clean_version);
                }
            }
        }

        Ok(packages)
    }
}