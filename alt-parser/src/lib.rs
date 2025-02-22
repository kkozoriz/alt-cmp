use std::collections::HashMap;
use std::error::Error;

pub mod proxmox;
pub mod altlinux;

pub trait PackageParser {
    fn parse(data: &str) -> Result<HashMap<String, String>, Box<dyn Error>>;
}
