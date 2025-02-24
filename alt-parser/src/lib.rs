use std::collections::HashMap;
use std::error::Error;

pub mod altlinux;
pub mod proxmox;

pub trait PackageParser {
    fn parse(data: &str) -> Result<HashMap<String, String>, Box<dyn Error>>;
}
