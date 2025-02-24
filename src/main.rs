mod cli;
mod output;
mod utils;

use alt_parser::{altlinux::AltLinuxParser, proxmox::ProxmoxParser, PackageParser};
use clap::Parser;
use cli::{Args, Commands};
use std::error::Error;
use utils::{fetch_package_list, read_package_mapping};

const ALT_REPO_NAME: &str = "ALT";
const SECOND_REPO_NAME: &str = "Second Repo";
const ALT_OUTPUT_FILE: &str = "alt_packages.txt";
const SECOND_OUTPUT_FILE: &str = "second_packages.txt";
const PARENT_DIR_PREFIX: &str = "../";

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let package_mapping = read_package_mapping(&args.mapping_file)?;
    let client = reqwest::blocking::Client::new();
    let alt_data = fetch_package_list(&client, &args.alt_url, ALT_REPO_NAME)?;
    let second_data = fetch_package_list(&client, &args.second_url, SECOND_REPO_NAME)?;

    match args.command {
        Commands::Compare {
            alt_newer,
            second_newer,
        } => {
            let alt_packages = AltLinuxParser::parse(&alt_data)?;
            let second_packages = ProxmoxParser::parse(&second_data)?;

            output::display_comparison(
                &alt_packages,
                &second_packages,
                &package_mapping,
                alt_newer,
                second_newer,
                &args,
            )?;
        }
        Commands::Fetch { dest_dir } => {
            let alt_output_path =
                dest_dir.join(format!("{}{}", PARENT_DIR_PREFIX, ALT_OUTPUT_FILE));
            let second_output_path = dest_dir.join(SECOND_OUTPUT_FILE);

            utils::save_to_file(&alt_data, &alt_output_path)?;
            utils::save_to_file(&second_data, &second_output_path)?;

            if !args.silent {
                println!("Package lists saved to {:?}", dest_dir);
            }
        }
        Commands::Stats { detailed } => {
            let alt_packages = AltLinuxParser::parse(&alt_data)?;
            let second_packages = ProxmoxParser::parse(&second_data)?;

            output::display_stats(
                &alt_packages,
                &second_packages,
                &package_mapping,
                detailed,
                &args,
            )?;
        }
    }

    Ok(())
}
