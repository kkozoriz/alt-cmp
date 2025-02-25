mod cli;
mod output;
mod utils;

use alt_parser::{altlinux::AltLinuxParser, proxmox::ProxmoxParser, PackageParser};
use clap::Parser;
use cli::{Args, Commands};
use std::error::Error;
use std::path::Path;
use utils::{fetch_package_list, read_package_mapping};

// Configuration constants
struct AppConfig {
    alt_repo_name: String,
    second_repo_name: String,
    alt_output_file: String,
    second_output_file: String,
    parent_dir_prefix: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            alt_repo_name: "ALT".to_string(),
            second_repo_name: "Second Repo".to_string(),
            alt_output_file: "alt_packages.txt".to_string(),
            second_output_file: "second_packages.txt".to_string(),
            parent_dir_prefix: "../".to_string(),
        }
    }
}

struct PackageComparisonApp {
    config: AppConfig,
    args: Args,
    client: reqwest::blocking::Client,
}

impl PackageComparisonApp {
    fn new(args: Args) -> Self {
        Self {
            config: AppConfig::default(),
            args,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn run(&self) -> Result<(), Box<dyn Error>> {
        let package_mapping = read_package_mapping(&self.args.mapping_file)?;

        let alt_data =
            fetch_package_list(&self.client, &self.args.alt_url, &self.config.alt_repo_name)?;

        let second_data = fetch_package_list(
            &self.client,
            &self.args.second_url,
            &self.config.second_repo_name,
        )?;

        match &self.args.command {
            Commands::Compare {
                alt_newer,
                second_newer,
            } => {
                self.handle_compare(
                    &package_mapping,
                    &alt_data,
                    &second_data,
                    *alt_newer,
                    *second_newer,
                )?;
            }
            Commands::Fetch { dest_dir } => {
                self.handle_fetch(dest_dir, &alt_data, &second_data)?;
            }
            Commands::Stats { detailed } => {
                self.handle_stats(&package_mapping, &alt_data, &second_data, *detailed)?;
            }
        }

        Ok(())
    }

    fn handle_compare(
        &self,
        package_mapping: &std::collections::HashMap<String, String>,
        alt_data: &str,
        second_data: &str,
        alt_newer: bool,
        second_newer: bool,
    ) -> Result<(), Box<dyn Error>> {
        let alt_packages = AltLinuxParser::parse(alt_data)?;
        let second_packages = ProxmoxParser::parse(second_data)?;

        output::display_comparison(
            &alt_packages,
            &second_packages,
            package_mapping,
            alt_newer,
            second_newer,
            &self.args,
        )
    }

    fn handle_fetch(
        &self,
        dest_dir: &Path,
        alt_data: &str,
        second_data: &str,
    ) -> Result<(), Box<dyn Error>> {
        let alt_output_path = dest_dir.join(format!(
            "{}{}",
            self.config.parent_dir_prefix, self.config.alt_output_file
        ));

        let second_output_path = dest_dir.join(&self.config.second_output_file);

        utils::save_to_file(alt_data, &alt_output_path)?;
        utils::save_to_file(second_data, &second_output_path)?;

        if !self.args.silent {
            println!("Package lists saved to {:?}", dest_dir);
        }

        Ok(())
    }

    fn handle_stats(
        &self,
        package_mapping: &std::collections::HashMap<String, String>,
        alt_data: &str,
        second_data: &str,
        detailed: bool,
    ) -> Result<(), Box<dyn Error>> {
        let alt_packages = AltLinuxParser::parse(alt_data)?;
        let second_packages = ProxmoxParser::parse(second_data)?;

        output::display_stats(
            &alt_packages,
            &second_packages,
            package_mapping,
            detailed,
            &self.args,
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let app = PackageComparisonApp::new(args);
    app.run()
}
