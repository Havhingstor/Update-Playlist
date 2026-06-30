use serde::Deserialize;
use std::process::Command;

use crate::Err;

#[derive(Deserialize)]
struct Playlist {
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
}

pub fn playlist_video_urls(mut url: &str) -> Result<Vec<String>, Err> {
    url = url.trim_matches('"');
    let output = Command::new("yt-dlp")
        .args(["--flat-playlist", "-J", url])
        .output()?;

    if !output.status.success() {
        return Err(format!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    let playlist: Playlist = serde_json::from_slice(&output.stdout)?;

    Ok(playlist
        .entries
        .into_iter()
        .map(|e| format!("https://www.youtube.com/watch?v={}", e.id))
        .collect())
}
