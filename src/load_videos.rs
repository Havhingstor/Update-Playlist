use serde::Deserialize;
use std::process::Command;

use crate::Err;

#[derive(Deserialize)]
struct Playlist {
    entries: Vec<Entry>,
    playlist_count: usize,
    title: String,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
}

pub fn playlist_video_urls(url: &str) -> Result<Vec<String>, Err> {
    let (result, playlist) = playlist_video_urls_internal(url)?;

    if result.len() != playlist.playlist_count {
        let error_msg = format_args!(
            "Possible problem: The playlist has {} entries but we got {}!",
            playlist.playlist_count,
            result.len()
        );

        if result.len() < playlist.playlist_count {
            return Err(format!(
                "{}\nRerun with -d to disregard this error and disable the check",
                error_msg
            )
            .into());
        } else {
            eprintln!("{} - All found videos are added", error_msg)
        }
    }

    Ok(result)
}

pub fn playlist_video_urls_unchecked(url: &str) -> Result<Vec<String>, Err> {
    let (result, _) = playlist_video_urls_internal(url)?;

    Ok(result)
}

fn playlist_video_urls_internal(mut url: &str) -> Result<(Vec<String>, Playlist), Err> {
    url = url.trim_matches('"');
    let output = Command::new("yt-dlp")
        .args(["--flat-playlist", "-J", url])
        .output()?;

    if !output.status.success() {
        return Err(format!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    let mut playlist: Playlist = serde_json::from_slice(&output.stdout)?;

    println!(
        r#"Loading videos from playlist "{}" with {} videos"#,
        playlist.title, playlist.playlist_count
    );

    let result: Vec<_> = playlist
        .entries
        .into_iter()
        .map(|e| format!("https://www.youtube.com/watch?v={}", e.id))
        .collect();

    playlist.entries = vec![];

    Ok((result, playlist))
}
