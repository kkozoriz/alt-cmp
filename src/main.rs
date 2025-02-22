mod utils;
mod output;
mod cli;

use cli::{Args, Commands};
use alt_parser::{altlinux::AltLinuxParser, proxmox::ProxmoxParser, PackageParser};
use std::error::Error;
use utils::{fetch_package_list, read_package_mapping};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let package_mapping = read_package_mapping(&args.mapping_file)?;

    match args.command {
        Commands::Compare { alt_newer, second_newer } => {
            let client = reqwest::Client::new();
            let alt_data = fetch_package_list(&client, &args.alt_url, "ALT").await?;
            let second_data = fetch_package_list(&client, &args.second_url, "Second Repo").await?;

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
            let client = reqwest::Client::new();
            let alt_data = fetch_package_list(&client, &args.alt_url, "ALT").await?;
            let second_data = fetch_package_list(&client, &args.second_url, "Second Repo").await?;

            utils::save_to_file(&alt_data, dest_dir.join("../alt_packages.txt"))?;
            utils::save_to_file(&second_data, dest_dir.join("second_packages.txt"))?;
            if !args.silent {
                println!("Package lists saved to {:?}", dest_dir);
            }
        }
        Commands::Stats { detailed } => {
            let client = reqwest::Client::new();
            let alt_data = fetch_package_list(&client, &args.alt_url, "ALT").await?;
            let second_data = fetch_package_list(&client, &args.second_url, "Second Repo").await?;

            let alt_packages = AltLinuxParser::parse(&alt_data)?;
            let second_packages = ProxmoxParser::parse(&second_data)?;

            output::display_stats(&alt_packages, &second_packages, &package_mapping, detailed, &args)?;
        }
    }

    Ok(())
}