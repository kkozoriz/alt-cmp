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
    #[arg(
        long = "alt-url",
        default_value = "https://ftp.altlinux.org/pub/distributions/ALTLinux/Sisyphus/files/list/bin.list.xz"
    )]
    pub alt_url: String,

    /// URL for the second repository package list (e.g., Proxmox)
    #[arg(
        long = "second-url",
        default_value = "http://download.proxmox.com/debian/pve/dists/bookworm/pve-no-subscription/binary-amd64/Packages"
    )]
    pub second_url: String,

    /// Path to the package mapping file
    #[arg(
        short = 'm',
        long = "mapping-file",
        default_value = "package_mapping.txt"
    )]
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
    // /// Check file only output
    // pub fn is_file_output_only(&self) -> bool {
    //     self.output_file.is_some() && self.silent
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::path::PathBuf;

    fn parse_args(args: &[&str]) -> Args {
        Args::parse_from(args)
    }

    #[test]
    fn test_default_args_compare() {
        let args = parse_args(&["alt-cmp", "compare"]);

        assert!(
            matches!(args.command, Commands::Compare { alt_newer, second_newer }
            if !alt_newer && !second_newer)
        );
        assert_eq!(
            args.alt_url,
            "https://ftp.altlinux.org/pub/distributions/ALTLinux/Sisyphus/files/list/bin.list.xz"
        );
        assert_eq!(args.second_url, "http://download.proxmox.com/debian/pve/dists/bookworm/pve-no-subscription/binary-amd64/Packages");
        assert_eq!(args.mapping_file, PathBuf::from("package_mapping.txt"));
        assert!(!args.silent);
        assert!(args.output_file.is_none());
    }

    #[test]
    fn test_compare_with_flags() {
        let args = parse_args(&["alt-cmp", "compare", "--alt-newer", "--second-newer"]);

        assert!(
            matches!(args.command, Commands::Compare { alt_newer, second_newer }
            if alt_newer && second_newer)
        );
    }

    #[test]
    fn test_fetch_with_custom_dir() {
        let args = parse_args(&["alt-cmp", "fetch", "--dest-dir", "/tmp/packages"]);

        if let Commands::Fetch { dest_dir } = args.command {
            assert_eq!(dest_dir, PathBuf::from("/tmp/packages"));
        } else {
            panic!("Expected Fetch command");
        }
    }

    #[test]
    fn test_stats_with_detailed() {
        let args = parse_args(&["alt-cmp", "stats", "--detailed"]);

        assert!(matches!(args.command, Commands::Stats { detailed } if detailed));
    }

    #[test]
    fn test_custom_urls_and_mapping() {
        let args = parse_args(&[
            "alt-cmp",
            "--alt-url",
            "http://example.com/alt",
            "--second-url",
            "http://example.com/second",
            "--mapping-file",
            "custom_mapping.txt",
            "compare",
        ]);

        assert_eq!(args.alt_url, "http://example.com/alt");
        assert_eq!(args.second_url, "http://example.com/second");
        assert_eq!(args.mapping_file, PathBuf::from("custom_mapping.txt"));
    }

    #[test]
    fn test_output_file_and_silent() {
        let args = parse_args(&["alt-cmp", "--output-file", "out.txt", "--silent", "compare"]);

        assert_eq!(args.output_file, Some(PathBuf::from("out.txt")));
        assert!(args.silent);
    }

    #[test]
    fn test_fetch_default_dir() {
        let args = parse_args(&["alt-cmp", "fetch"]);
        if let Commands::Fetch { dest_dir } = args.command {
            assert_eq!(dest_dir, PathBuf::from("./"));
        } else {
            panic!("Expected Fetch command");
        }
    }

    #[test]
    #[should_panic]
    fn test_missing_subcommand() {
        parse_args(&["alt-cmp"]);
    }

    #[test]
    fn test_version_flag() {
        let args = parse_args(&["alt-cmp", "--version"]);

        assert!(matches!(args.command, _));
    }
}
