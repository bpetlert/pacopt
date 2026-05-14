use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    /// Package name
    pub name: String,

    /// Show installed packages
    #[arg(long)]
    pub installed: bool,

    /// Show uninstalled packages
    #[arg(long)]
    pub uninstalled: bool,

    /// Show package name only
    #[arg(long)]
    pub name_only: bool,

    /// Create argument list
    #[arg(long)]
    pub xargs: bool,

    /// Output to JSON format without filter
    #[arg(long)]
    pub json: bool,
}
