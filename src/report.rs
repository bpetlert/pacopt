use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Report {
    pub packages: Vec<Package>,
}

#[derive(Debug, Serialize)]
pub struct Package {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "Installed")]
    pub installed: bool,
}

impl Report {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
        }
    }

    pub fn build(&mut self) -> Result<()> {
        todo!()
    }
}
