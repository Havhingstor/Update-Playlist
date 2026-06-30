use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::ops::ControlFlow;
use std::path::Path;

use clap::Parser;

use crate::args::Args;
use crate::args::Commands;
use crate::load_videos::playlist_video_urls;

mod args;
mod load_videos;

type Err = Box<dyn Error>;

/// The index that the first video is referred by
const FIRST_INDEX: usize = 1;
/// The line index that the first video is on in the file
const FIRST_VIDEO_LINE: usize = 2;

#[derive(Default)]
struct State {
    url: String,
    videos_up_to_next: Vec<String>,
}

fn main() -> Result<(), Err> {
    let args = Args::parse();

    let path = &args.file;

    let state = args.command.map_or_else(
        || load_old_file(path),
        |val| {
            let Commands::Add { playlist } = val;
            Ok(State {
                url: playlist,
                ..Default::default()
            })
        },
    )?;

    let videos = playlist_video_urls(&state.url)?;

    let mut next_video = None;
    if let Some(next_url) = state.videos_up_to_next.last() {
        let video_eval = |(idx, new_video)| {
            if new_video == next_url {
                ControlFlow::Break((idx, next_url))
            } else {
                if !state.videos_up_to_next.contains(new_video) {
                    println!("New video before the current: {new_video}");
                }

                ControlFlow::Continue(())
            }
        };

        next_video = videos
            .iter()
            .enumerate()
            .try_for_each(video_eval)
            .break_value();
    }

    let index = if let Some((index, next_url)) = next_video {
        println!("Next video: {next_url}");
        // We must modify the indices correctly. Since this index is already offset regarding the
        // first video, we only add the FIRST_INDEX offset
        index + FIRST_INDEX
    } else {
        FIRST_INDEX
    };

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(state.url.as_bytes())?;
    writer.write_all(b"\n")?;

    writeln!(writer, "{index}")?;

    videos.iter().try_for_each(|video| {
        writer.write_all(video.as_bytes())?;
        writer.write_all(b"\n")
    })?;

    println!("Wrote index {index} and {} videos into file.", videos.len());

    Ok(())
}

fn load_old_file(path: &Path) -> Result<State, Err> {
    let file = File::open(path)?;

    let reader = BufReader::new(file);

    let mut iter = reader.lines().map_while(|line| line.ok()).enumerate();

    let mut result = if let Some((_, line)) = iter.next() {
        State {
            url: line,
            ..Default::default()
        }
    } else {
        return Err(format!(
            "The file {} was empty! It must at least contain the URL of the playlist",
            path.to_string_lossy()
        )
        .into());
    };

    if let Some((_, line)) = iter.next()
        && let Ok(idx) = line.parse::<usize>()
    {
        // We must modify the indices correctly
        let index = idx.saturating_sub(FIRST_INDEX) + FIRST_VIDEO_LINE;
        result.videos_up_to_next = iter
            .map_while(|(idx, line)| if idx <= index { Some(line) } else { None })
            .collect()
    }

    Ok(result)
}
