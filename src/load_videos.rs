use serde::Deserialize;
use std::process::Command;

use crate::{Err, args::Order};

#[derive(Deserialize)]
struct Playlist {
    entries: Vec<Option<Entry>>,
    playlist_count: usize,
    title: String,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    timestamp: Option<u32>,
    title: Option<String>,
}

pub fn playlist_video_urls(url: &str, order: &Order) -> Result<Vec<String>, Err> {
    let (result, playlist, orig_video_number) = playlist_video_urls_internal(url, order)?;

    // We can't use `result.len()`, because hidden videos might've been filtered out already
    if orig_video_number != playlist.playlist_count {
        let error_msg = format_args!(
            "Possible problem: The playlist has {} entries but we got {}!",
            playlist.playlist_count, orig_video_number
        );

        if orig_video_number < playlist.playlist_count {
            return Err(format!(
                "{}\nRerun with -d to disregard this error and disable the check",
                error_msg
            )
            .into());
        } else {
            eprintln!("{} - All found videos are added", error_msg)
        }
    }

    let hidden_vids = playlist.playlist_count - result.len();
    if hidden_vids > 0 {
        eprintln!(
            "NOTE: {hidden_vids} hidden videos have been filtered out - The actual number of videos is {}",
            result.len()
        )
    }

    Ok(result)
}

pub fn playlist_video_urls_unchecked(url: &str, order: &Order) -> Result<Vec<String>, Err> {
    let (result, _, _) = playlist_video_urls_internal(url, order)?;

    Ok(result)
}

fn playlist_video_urls_internal(
    mut url: &str,
    order: &Order,
) -> Result<(Vec<String>, Playlist, usize), Err> {
    url = url.trim_matches('"');
    let playlist_load_arg = if matches!(order, Order::UploadDate | Order::Title) {
        "--skip-download"
    } else {
        "--flat-playlist"
    };
    let output = Command::new("yt-dlp")
        .args(["-i", playlist_load_arg, "-J", url])
        .output()?;

    if !output.status.success() && contains_actual_errors(&output.stderr) {
        return Err(format!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    let mut playlist: Playlist = serde_json::from_slice(&output.stdout)?;

    println!(
        r#"Loading videos from playlist "{}" with {} videos"#,
        playlist.title, playlist.playlist_count
    );

    let mut entries: Vec<_> = playlist.entries.iter().filter_map(Option::as_ref).collect();

    match order {
        Order::None => {}
        Order::UploadDate => {
            for video in &entries {
                if video.timestamp.is_none() {
                    return Err(format!(
                        "Can't order by title because video with id {} has no title!",
                        video.id
                    )
                    .into());
                };
            }
            entries.sort_by_key(|video| video.timestamp.unwrap())
        }
        Order::Title => {
            for video in &entries {
                let Some(ref title) = video.title else {
                    return Err(format!(
                        "Can't order by title because video with id {} has no title!",
                        video.id
                    )
                    .into());
                };

                if title.find(|c: char| c.is_ascii_digit()).is_none() {
                    return Err(
                        format!("The video title \"{title}\" doesn't contain a number!").into(),
                    );
                }
            }
            entries.sort_by_cached_key(|video| {
                let title = video.title.as_ref().unwrap();
                let first = title.find(|ch: char| ch.is_ascii_digit()).unwrap();

                let after_last = title[first..]
                    .find(|ch: char| !ch.is_ascii_digit())
                    .map_or(title.len(), |short_len| short_len + first);

                title[first..after_last].parse::<usize>().unwrap()
            });
        }
        Order::Reverse => entries.reverse(),
    }

    let result: Vec<_> = entries
        .into_iter()
        .map(|e| format!("https://www.youtube.com/watch?v={}", e.id))
        .collect();

    let orig_video_number = playlist.entries.len();

    playlist.entries = vec![];

    Ok((result, playlist, orig_video_number))
}

fn contains_actual_errors(stderr: &[u8]) -> bool {
    let stderr = String::from_utf8_lossy(stderr);

    for line in stderr.lines() {
        if line.starts_with("ERROR:") && !line.contains("Private video") {
            return true;
        }
    }

    false
}
