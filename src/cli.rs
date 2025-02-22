use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "alt-cmp")]
#[command(version = "0.1.0")]
#[command(about = "Compare package versions between ALT Linux Sisyphus and other repositories", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to the output file (if not specified, outputs to console)
    #[arg(short = 'o', long = "output-file", value_name = "FILE")]
    pub(crate) output_file: Option<PathBuf>,

    /// URL for the ALT Linux package list (default: Sisyphus bin.list.xz)
    #[arg(long = "alt-url", default_value = "https://ftp.altlinux.org/pub/distributions/ALTLinux/Sisyphus/files/list/bin.list.xz")]
    pub alt_url: String,

    /// URL for the second repository package list (e.g., Proxmox)
    #[arg(long = "second-url", default_value = "http://download.proxmox.com/debian/pve/dists/bookworm/pve-no-subscription/binary-amd64/Packages")]
    pub second_url: String,

    /// Path to the package mapping file
    #[arg(short = 'm', long = "mapping-file", default_value = "package_mapping.txt")]
    pub mapping_file: PathBuf,

    /// Silent mode (suppress console output except errors)
    #[arg(short = 's', long = "silent")]
    pub silent: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compare all packages and display full comparison table
    Compare {
        /// Show only packages where ALT version is newer
        #[arg(long = "alt-newer")]
        alt_newer: bool,

        /// Show only packages where the second repo version is newer
        #[arg(long = "second-newer")]
        second_newer: bool,
    },

    /// Fetch and save package lists without comparison
    Fetch {
        /// Directory to save fetched package lists
        #[arg(short = 'd', long = "dest-dir", default_value = "./")]
        dest_dir: PathBuf,
    },

    /// Display statistics about package versions
    Stats {
        /// Include detailed breakdown by version differences
        #[arg(long = "detailed")]
        detailed: bool,
    },
}

impl Args {
    /// Check file only output
    pub fn is_file_output_only(&self) -> bool {
        self.output_file.is_some() && self.silent
    }
}