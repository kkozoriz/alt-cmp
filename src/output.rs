use crate::cli::Args;
use colored::Colorize;
use prettytable::{Cell, Row, Table};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use version_compare::Version;

#[derive(Clone)]
struct PackageInfo {
    name: String,
    alt_version: String,
    second_version: String,
}

struct PackageComparator {
    alt_packages: HashMap<String, String>,
    second_packages: HashMap<String, String>,
    package_mapping: HashMap<String, String>,
}

impl PackageComparator {
    fn new(
        alt_packages: HashMap<String, String>,
        second_packages: HashMap<String, String>,
        package_mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            alt_packages,
            second_packages,
            package_mapping,
        }
    }

    fn get_package_info(&self, alt_name: &str, second_name: &str) -> PackageInfo {
        let alt_version = self
            .alt_packages
            .get(alt_name)
            .cloned()
            .unwrap_or_else(|| "Not found".to_string());

        let second_version = self
            .second_packages
            .get(second_name)
            .cloned()
            .unwrap_or_else(|| "Not found".to_string());

        PackageInfo {
            name: format_package_name(alt_name, second_name),
            alt_version,
            second_version,
        }
    }

    fn compare_versions(&self, package: &PackageInfo) -> VersionComparisonResult {
        let alt_ver = Version::from(&package.alt_version);
        let second_ver = Version::from(&package.second_version);

        match (alt_ver, second_ver) {
            (Some(alt), Some(second)) => {
                if alt < second {
                    VersionComparisonResult::SecondNewer
                } else if alt > second {
                    VersionComparisonResult::AltNewer
                } else {
                    VersionComparisonResult::Equal
                }
            }
            (None, Some(_)) => VersionComparisonResult::AltMissing,
            (Some(_), None) => VersionComparisonResult::SecondMissing,
            (None, None) => VersionComparisonResult::BothMissing,
        }
    }

    fn should_include_package(
        &self,
        comparison: &VersionComparisonResult,
        alt_newer: bool,
        second_newer: bool,
    ) -> bool {
        if !alt_newer && !second_newer {
            return true;
        }

        match comparison {
            VersionComparisonResult::AltNewer => alt_newer,
            VersionComparisonResult::SecondNewer => second_newer,
            VersionComparisonResult::Equal => alt_newer && second_newer,
            _ => !alt_newer && !second_newer,
        }
    }

    fn get_all_packages(
        &self,
        alt_newer: bool,
        second_newer: bool,
    ) -> Vec<(PackageInfo, VersionComparisonResult)> {
        let mut result = Vec::new();

        for (alt_name, second_name) in &self.package_mapping {
            let package = self.get_package_info(alt_name, second_name);
            let comparison = self.compare_versions(&package);

            if self.should_include_package(&comparison, alt_newer, second_newer) {
                result.push((package, comparison));
            }
        }

        result
    }

    fn calculate_stats(&self) -> PackageStats {
        let mut stats = PackageStats::default();

        for (alt_name, second_name) in &self.package_mapping {
            let package = self.get_package_info(alt_name, second_name);
            let comparison = self.compare_versions(&package);

            match comparison {
                VersionComparisonResult::AltNewer => stats.alt_newer += 1,
                VersionComparisonResult::SecondNewer => stats.second_newer += 1,
                VersionComparisonResult::Equal => stats.equal += 1,
                _ => stats.missing += 1,
            }
        }

        stats
    }
}

enum VersionComparisonResult {
    AltNewer,
    SecondNewer,
    Equal,
    AltMissing,
    SecondMissing,
    BothMissing,
}

#[derive(Default)]
struct PackageStats {
    alt_newer: i32,
    second_newer: i32,
    equal: i32,
    missing: i32,
}

struct TableFormatter;

impl TableFormatter {
    fn create_header_row() -> Row {
        Row::new(vec![
            Cell::new("Package (ALT/Second)").style_spec("bFc"),
            Cell::new("ALT Version").style_spec("bFg"),
            Cell::new("Second Version").style_spec("bFm"),
        ])
    }

    fn create_data_row(package: &PackageInfo, comparison: &VersionComparisonResult) -> Row {
        let (alt_style, second_style) = match comparison {
            VersionComparisonResult::AltNewer => ("Fg", "Fr"),
            VersionComparisonResult::SecondNewer => ("Fr", "Fg"),
            VersionComparisonResult::Equal => ("Fw", "Fw"),
            VersionComparisonResult::AltMissing => ("Fr", "Fm"),
            VersionComparisonResult::SecondMissing => ("Fg", "Fr"),
            VersionComparisonResult::BothMissing => ("Fr", "Fr"),
        };

        Row::new(vec![
            Cell::new(&package.name).style_spec("Fy"),
            Cell::new(&package.alt_version).style_spec(alt_style),
            Cell::new(&package.second_version).style_spec(second_style),
        ])
    }

    fn get_colored_versions(
        package: &PackageInfo,
        comparison: &VersionComparisonResult,
    ) -> (String, String) {
        match comparison {
            VersionComparisonResult::SecondNewer => (
                package.alt_version.red().to_string(),
                package.second_version.green().to_string(),
            ),
            VersionComparisonResult::AltNewer => (
                package.alt_version.green().to_string(),
                package.second_version.red().to_string(),
            ),
            VersionComparisonResult::Equal => (
                package.alt_version.white().to_string(),
                package.second_version.white().to_string(),
            ),
            VersionComparisonResult::AltMissing => (
                package.alt_version.red().to_string(),
                package.second_version.magenta().to_string(),
            ),
            VersionComparisonResult::SecondMissing | VersionComparisonResult::BothMissing => (
                package.alt_version.green().to_string(),
                package.second_version.red().to_string(),
            ),
        }
    }

    fn create_table(packages_with_comparison: &[(PackageInfo, VersionComparisonResult)]) -> Table {
        let mut table = Table::new();
        table.add_row(Self::create_header_row());

        for (package, comparison) in packages_with_comparison {
            table.add_row(Self::create_data_row(package, comparison));
        }

        table
    }

    fn create_formatted_table_for_terminal(
        packages_with_comparison: &[(PackageInfo, VersionComparisonResult)],
    ) -> Table {
        Self::create_table(packages_with_comparison)
    }

    fn create_formatted_table_for_file(
        packages_with_comparison: &[(PackageInfo, VersionComparisonResult)],
    ) -> String {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new(&"Package (ALT/Second)".cyan().bold().to_string()),
            Cell::new(&"ALT Version".green().bold().to_string()),
            Cell::new(&"Second Version".magenta().bold().to_string()),
        ]));

        for (package, comparison) in packages_with_comparison {
            let (alt_colored, second_colored) = Self::get_colored_versions(package, comparison);

            table.add_row(Row::new(vec![
                Cell::new(&package.name.yellow().to_string()),
                Cell::new(&alt_colored),
                Cell::new(&second_colored),
            ]));
        }

        format!("{}", table)
    }
}

struct StatsFormatter;

impl StatsFormatter {
    fn format_stats(stats: &PackageStats, detailed: bool) -> String {
        if detailed {
            format!(
                "Package Statistics:\n- ALT newer: {}\n- Second newer: {}\n- Equal: {}\n- Missing: {}",
                stats.alt_newer, stats.second_newer, stats.equal, stats.missing
            )
        } else {
            format!(
                "ALT newer: {}, Second newer: {}, Equal: {}, Missing: {}",
                stats.alt_newer, stats.second_newer, stats.equal, stats.missing
            )
        }
    }
}

fn format_package_name(alt_name: &str, second_name: &str) -> String {
    if alt_name == second_name {
        alt_name.to_string()
    } else {
        format!("{} / {}", alt_name, second_name)
    }
}

pub fn display_comparison(
    alt_packages: &HashMap<String, String>,
    second_packages: &HashMap<String, String>,
    package_mapping: &HashMap<String, String>,
    alt_newer: bool,
    second_newer: bool,
    args: &Args,
) -> Result<(), Box<dyn Error>> {
    let comparator = PackageComparator::new(
        alt_packages.clone(),
        second_packages.clone(),
        package_mapping.clone(),
    );

    let packages_with_comparison = comparator.get_all_packages(alt_newer, second_newer);
    let header = "\nALT vs Second Repo Package Comparison\n"
        .blue()
        .bold()
        .to_string();

    // File output
    if let Some(ref output_file) = args.output_file {
        let table_content =
            TableFormatter::create_formatted_table_for_file(&packages_with_comparison);
        let output = format!("{}{}", header, table_content);

        let mut file = File::create(output_file)?;
        file.write_all(output.as_bytes())?;
    }

    if !args.silent {
        let table = TableFormatter::create_formatted_table_for_terminal(&packages_with_comparison);
        println!("{}", header);
        table.printstd();
    }

    Ok(())
}

pub fn display_stats(
    alt_packages: &HashMap<String, String>,
    second_packages: &HashMap<String, String>,
    package_mapping: &HashMap<String, String>,
    detailed: bool,
    args: &Args,
) -> Result<(), Box<dyn Error>> {
    let comparator = PackageComparator::new(
        alt_packages.clone(),
        second_packages.clone(),
        package_mapping.clone(),
    );

    let stats = comparator.calculate_stats();
    let output = StatsFormatter::format_stats(&stats, detailed);

    if let Some(ref output_file) = args.output_file {
        let mut file = File::create(output_file)?;
        file.write_all(output.as_bytes())?;
    }

    if !args.silent {
        println!("{}", output);
    }

    Ok(())
}
