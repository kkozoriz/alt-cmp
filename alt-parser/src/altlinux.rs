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
                let raw_version = parts[1];

                let version = raw_version
                    .split(':')
                    .last()
                    .unwrap_or(raw_version)
                    .split('-')
                    .filter(|part| !part.starts_with("alt"))
                    .collect::<Vec<&str>>()
                    .join("-");

                packages.insert(name, version);
            }
        }
        Ok(packages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_valid_data() {
        let input = "package1 1.2.3-alt1\n\
                     package2 2:4.5.6-alt2\n\
                     package3 0.1-alt3\n";

        let result = AltLinuxParser::parse(input).unwrap();
        let mut expected = HashMap::new();
        expected.insert("package1".to_string(), "1.2.3".to_string());
        expected.insert("package2".to_string(), "4.5.6".to_string());
        expected.insert("package3".to_string(), "0.1".to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_empty_data() {
        let input = "";
        let result = AltLinuxParser::parse(input).unwrap();
        let expected: HashMap<String, String> = HashMap::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_invalid_line() {
        let input = "package1 1.2.3-alt1\n\
                     invalid_line\n\
                     package2 2:4.5.6-alt2\n";

        let result = AltLinuxParser::parse(input).unwrap();
        let mut expected = HashMap::new();
        expected.insert("package1".to_string(), "1.2.3".to_string());
        expected.insert("package2".to_string(), "4.5.6".to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_version_with_colon() {
        let input = "package1 1:2:3.4.5-alt1";
        let result = AltLinuxParser::parse(input).unwrap();
        let mut expected = HashMap::new();
        expected.insert("package1".to_string(), "3.4.5".to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_single_part_line() {
        let input = "package1\n\
                     package2 2.3.4-alt1\n";

        let result = AltLinuxParser::parse(input).unwrap();
        let mut expected = HashMap::new();
        expected.insert("package2".to_string(), "2.3.4".to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_no_alt_suffix() {
        let input = "package1 1.2.3";
        let result = AltLinuxParser::parse(input).unwrap();
        let mut expected = HashMap::new();
        expected.insert("package1".to_string(), "1.2.3".to_string());

        assert_eq!(result, expected);
    }
}