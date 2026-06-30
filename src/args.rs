use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// The file in which the updated or new playlist videos should be written
    pub file: PathBuf,
    /// Don't abort the update even if not all videos could be loaded
    #[arg(short, long)]
    pub disable_length_checks: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Creates & initialises a new file for the playlist
    Add {
        /// The URL of the playlist
        playlist: String,
    },
}
