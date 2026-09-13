use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// The file in which the updated or new playlist videos should be written
    pub file: PathBuf,
    #[arg(short, long)]
    /// Don't abort the update even if not all videos could be loaded
    pub disable_length_checks: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Creates & initialises a new file for the playlist
    Add {
        /// The URL of the playlist
        playlist: String,
        #[arg(short, long, value_enum, default_value_t = Order::None)]
        /// The order of the videos
        order: Order,
    },
}

#[derive(Clone, Copy, ValueEnum, Default, strum::Display, strum::EnumString)]
/// The order of the videos
pub enum Order {
    #[default]
    /// Use the default (manual) order of the youtube playlist
    None,
    /// Download all the metadata for each video to order by upload date
    ///
    /// This will be used to order the videos (oldest to newest), but it takes longer and might
    /// require a JS runtime to be installed (this is only the case if `yt-dlp` complains when
    /// executing)
    UploadDate,
    /// Download all the metadata for each video to order the videos by the first encountered number in the title
    ///
    /// If there are videos that don't have a number in the title, the command fails
    /// It takes longer and might require a JS runtime to be installed (this is only the case if
    /// `yt-dlp` complains when executing)
    Title,
    /// Reverse the (manual) order of the playlist
    Reverse,
}
