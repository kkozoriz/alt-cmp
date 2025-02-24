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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_valid_data() {
        let input = "Package: test-package\n\
                     Version: 1.2.3-4\n\
                     Package: another-package\n\
                     Version: 2.3.4+git123\n";

        let result = ProxmoxParser::parse(input).unwrap();
        let mut expected = HashMap::new();
        expected.insert("test-package".to_string(), "1.2.3".to_string());
        expected.insert("another-package".to_string(), "2.3.4".to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_empty_data() {
        let input = "";
        let result = ProxmoxParser::parse(input).unwrap();
        let expected: HashMap<String, String> = HashMap::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_missing_version() {
        let input = "Package: test-package\n\
                     Package: another-package\n";

        let result = ProxmoxParser::parse(input).unwrap();
        let expected: HashMap<String, String> = HashMap::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_invalid_regex() {
        let input = "Not a package format\nInvalid line\n";
        let result = ProxmoxParser::parse(input).unwrap();
        let expected: HashMap<String, String> = HashMap::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_clean_version_simple() {
        let version = "1.2.3";
        let result = clean_version_string(version);
        assert_eq!(result, "1.2.3");
    }

    #[test]
    fn test_clean_version_with_dash() {
        let version = "1.2.3-4";
        let result = clean_version_string(version);
        assert_eq!(result, "1.2.3");
    }

    #[test]
    fn test_clean_version_with_plus() {
        let version = "1.2.3+git123";
        let result = clean_version_string(version);
        assert_eq!(result, "1.2.3");
    }

    #[test]
    fn test_clean_version_with_both() {
        let version = "1.2.3-4+git123";
        let result = clean_version_string(version);
        assert_eq!(result, "1.2.3");
    }

    #[test]
    fn test_clean_version_empty() {
        let version = "";
        let result = clean_version_string(version);
        assert_eq!(result, "");
    }
}
