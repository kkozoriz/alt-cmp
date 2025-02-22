use crate::cli::Args;
use prettytable::{Table, Row, Cell};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use colored::Colorize;
use version_compare::Version;

// TODO!: need refactor

#[derive(Clone)]
struct PackageVersions {
    alt_version: String,
    second_version: String,
    alt_ver: Option<String>,
    second_ver: Option<String>,
}

#[derive(Clone)]
struct VersionInfo {
    alt_version: String,
    second_version: String,
    alt_style: &'static str,
    second_style: &'static str,
}

fn create_header_row(row: &Row) -> Row {
    Row::new(vec![
        Cell::new(&row.get_cell(0).unwrap().get_content()).style_spec("bFc"), // Bold Cyan
        Cell::new(&row.get_cell(1).unwrap().get_content()).style_spec("bFg"), // Bold Green
        Cell::new(&row.get_cell(2).unwrap().get_content()).style_spec("bFm"), // Bold Magenta
    ])
}

fn create_data_row(package: &str, version_info: VersionInfo) -> Row {
    Row::new(vec![
        Cell::new(package).style_spec("Fy"),
        Cell::new(&version_info.alt_version).style_spec(version_info.alt_style),
        Cell::new(&version_info.second_version).style_spec(version_info.second_style),
    ])
}

fn get_package_versions(
    alt_packages: &HashMap<String, String>,
    second_packages: &HashMap<String, String>,
    alt_name: &String,
    second_name: &String,
) -> PackageVersions {
    let alt_version = alt_packages.get(alt_name).unwrap_or(&"Not found".to_string()).clone();
    let second_version = second_packages.get(second_name).unwrap_or(&"Not found".to_string()).clone();

    PackageVersions {
        alt_version: alt_version.clone(),
        second_version: second_version.clone(),
        alt_ver: Some(alt_version.clone()),
        second_ver: Some(second_version.clone()),
    }
}

fn should_include_row(
    alt_ver: &Option<String>,
    second_ver: &Option<String>,
    alt_newer: bool,
    second_newer: bool,
) -> bool {
    match (alt_ver.as_ref().and_then(|v| Version::from(v)), second_ver.as_ref().and_then(|v| Version::from(v))) {
        (Some(alt), Some(second)) => {
            if alt_newer && second_newer { true }
            else if alt_newer { alt > second }
            else if second_newer { second > alt }
            else { true }
        }
        _ => !alt_newer && !second_newer,
    }
}

fn format_package_name(alt_name: &str, second_name: &str) -> String {
    if alt_name == second_name { alt_name.to_string() }
    else { format!("{} / {}", alt_name, second_name) }
}

fn get_colored_versions(versions: &PackageVersions) -> (String, String) {
    let alt_ver = versions.alt_ver.as_ref().and_then(|v| Version::from(v));
    let second_ver = versions.second_ver.as_ref().and_then(|v| Version::from(v));

    match (alt_ver, second_ver) {
        (Some(alt), Some(second)) => {
            if alt < second {
                (versions.alt_version.red().to_string(), versions.second_version.green().to_string())
            } else if alt > second {
                (versions.alt_version.green().to_string(), versions.second_version.red().to_string())
            } else {
                (versions.alt_version.white().to_string(), versions.second_version.white().to_string())
            }
        }
        (None, _) => (versions.alt_version.red().to_string(), versions.second_version.magenta().to_string()),
        (_, None) => (versions.alt_version.green().to_string(), versions.second_version.red().to_string()),
    }
}

fn init_row() -> Table {
    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new("Package (ALT/Second)"),
        Cell::new("ALT Version"),
        Cell::new("Second Version"),
    ]));

    table
}

fn fill_table(
    table: &mut Table,
    package_mapping: &HashMap<String, String>,
    alt_packages: &HashMap<String, String>,
    alt_newer: bool,
    second_newer: bool,
    second_packages: &HashMap<String, String>,
) {
    for (alt_name, second_name) in package_mapping {
        let versions = get_package_versions(alt_packages, second_packages, alt_name, second_name);
        if should_include_row(&versions.alt_ver, &versions.second_ver, alt_newer, second_newer) {
            let package_display = format_package_name(alt_name, second_name);

            table.add_row(Row::new(vec![
                Cell::new(&package_display),
                Cell::new(&versions.alt_version),
                Cell::new(&versions.second_version),
            ]));
        }
    }
}

fn get_version_info(
    alt_packages: &HashMap<String, String>,
    second_packages: &HashMap<String, String>,
    alt_name: &str,
    second_name: &str,
) -> VersionInfo {
    let alt_version = alt_packages.get(alt_name).unwrap_or(&"Not found".to_string()).clone();
    let second_version = second_packages.get(second_name).unwrap_or(&"Not found".to_string()).clone();

    let alt_ver = Version::from(&alt_version);
    let second_ver = Version::from(&second_version);

    let (alt_style, second_style) = match (alt_ver, second_ver) {
        (Some(alt), Some(second)) => {
            if alt < second { ("Fr", "Fg") }
            else if alt > second { ("Fg", "Fr") }
            else { ("Fw", "Fw") }
        }
        (None, _) => ("Fr", "Fm"),
        (_, None) => ("Fg", "Fr"),
    };

    VersionInfo {
        alt_version,
        second_version,
        alt_style,
        second_style,
    }
}

fn terminal_output(
    table: &Table,
    alt_packages: &HashMap<String, String>,
    second_packages: &HashMap<String, String>,
    header: String,
) {
    let mut styled_table = Table::new();

    for (i, row) in table.row_iter().enumerate() {
        if i == 0 {
            styled_table.add_row(create_header_row(row));
        } else {
            let package = row.get_cell(0).unwrap().get_content();
            let alt_name = package.split(" / ").next().unwrap();
            let second_name = package.split(" / ").last().unwrap();

            let version_info = get_version_info(alt_packages, second_packages, alt_name, second_name);
            styled_table.add_row(create_data_row(&package, version_info));
        }
    }

    print!("{}", header);
    styled_table.printstd();
}

fn file_output(
    alt_packages: &HashMap<String, String>,
    second_packages: &HashMap<String, String>,
    package_mapping: &HashMap<String, String>,
    alt_newer: bool,
    second_newer: bool,
    header: &String,
    output_file: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut file_output = Table::new();
    file_output.add_row(Row::new(vec![
        Cell::new(&"Package (ALT/Second)".cyan().bold().to_string()),
        Cell::new(&"ALT Version".green().bold().to_string()),
        Cell::new(&"Second Version".magenta().bold().to_string()),
    ]));

    for (alt_name, second_name) in package_mapping {
        let versions = get_package_versions(alt_packages, second_packages, alt_name, second_name);
        if should_include_row(&versions.alt_ver, &versions.second_ver, alt_newer, second_newer) {
            let (alt_colored, second_colored) = get_colored_versions(&versions);
            let package_display = format_package_name(alt_name, second_name);

            file_output.add_row(Row::new(vec![
                Cell::new(&package_display.yellow().to_string()),
                Cell::new(&alt_colored),
                Cell::new(&second_colored),
            ]));
        }
    }

    let output = format!("{}{}", header, file_output);
    let mut file = File::create(output_file)?;

    file.write_all(output.as_bytes())?;

    Ok(())
}
pub fn display_comparison(
    alt_packages: &HashMap<String, String>,
    second_packages: &HashMap<String, String>,
    package_mapping: &HashMap<String, String>,
    alt_newer: bool,
    second_newer: bool,
    args: &Args,
) -> Result<(), Box<dyn Error>> {
    let mut table = init_row();
    let header = "\nALT vs Second Repo Package Comparison\n".blue().bold().to_string();

    fill_table(&mut table, package_mapping, alt_packages, alt_newer, second_newer, second_packages);

    if let Some(ref output_file) = args.output_file {
        file_output(alt_packages, second_packages, package_mapping, alt_newer, second_newer, &header, output_file)?
    }

    if !args.silent { terminal_output(&table, alt_packages, second_packages, header); }

    Ok(())
}

fn create_output_string(
    detailed: bool,
    alt_newer: i32,
    second_newer: i32,
    equal: i32,
    missing: i32
) -> String {
    if detailed {
        format!(
            "Package Statistics:\n- ALT newer: {}\n- Second newer: {}\n- Equal: {}\n- Missing: {}",
            alt_newer, second_newer, equal, missing
        )
    } else {
        format!(
            "ALT newer: {}, Second newer: {}, Equal: {}, Missing: {}",
            alt_newer, second_newer, equal, missing
        )
    }
}

pub fn display_stats(
    alt_packages: &HashMap<String, String>,
    second_packages: &HashMap<String, String>,
    package_mapping: &HashMap<String, String>,
    detailed: bool,
    args: &Args,
) -> Result<(), Box<dyn Error>> {
    let (mut alt_newer, mut second_newer) = (0, 0);
    let (mut equal, mut missing) = (0, 0);

    for (alt_name, second_name) in package_mapping {
        let alt_version = alt_packages.get(alt_name);
        let second_version = second_packages.get(second_name);

        match (alt_version, second_version) {
            (Some(alt), Some(second)) => {
                let alt_ver = Version::from(alt);
                let second_ver = Version::from(second);

                if let (Some(alt_v), Some(second_v)) = (alt_ver, second_ver) {
                    if alt_v > second_v { alt_newer += 1; }
                    else if alt_v < second_v { second_newer += 1; }
                    else { equal += 1; }
                }
            }
            _ => missing += 1,
        }
    }

    let output = create_output_string(detailed, alt_newer, second_newer, equal, missing);

    if let Some(ref output_file) = args.output_file {
        let mut file = File::create(output_file)?;
        file.write_all(output.as_bytes())?;
    }

    if !args.silent { println!("{}", output); }

    Ok(())
}
