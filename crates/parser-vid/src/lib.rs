use parser_common::{
    CollageBuilder, Error, ImageOptions, MAX_ITEMS, Manifest, OutputDirectory, ParseOutput, Result,
    Selection, Tile, source_path,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VideoRequest {
    pub path: PathBuf,
    pub from: u64,
    pub to: u64,
    pub jump: u64,
    pub output_dir: PathBuf,
    #[serde(default)]
    pub images: ImageOptions,
}

#[derive(Deserialize)]
struct Probe {
    streams: Vec<Stream>,
    format: Format,
}

#[derive(Deserialize)]
struct Stream {
    index: u32,
    codec_type: Option<String>,
    duration: Option<String>,
    #[serde(default)]
    disposition: Disposition,
}

#[derive(Default, Deserialize)]
struct Disposition {
    #[serde(default)]
    attached_pic: u32,
}

#[derive(Deserialize)]
struct Format {
    duration: Option<String>,
}

pub fn parse(request: VideoRequest) -> Result<ParseOutput> {
    request.images.validate()?;
    if request.from >= request.to || request.jump == 0 {
        return Err(Error::InvalidInput(
            "Video times are milliseconds; require from < to and jump > 0".into(),
        ));
    }
    let count = (request.to - request.from).div_ceil(request.jump);
    if count > MAX_ITEMS {
        return Err(Error::InvalidInput(format!(
            "Selection produces {count} frames; increase jump or select a shorter clip (maximum {MAX_ITEMS})"
        )));
    }
    let path = source_path(&request.path)?;
    let mut command = Command::new("ffprobe");
    command
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(&path);
    let probe: Probe = serde_json::from_slice(&run(command)?.stdout)?;
    let video = probe
        .streams
        .iter()
        .find(|stream| {
            stream.codec_type.as_deref() == Some("video") && stream.disposition.attached_pic == 0
        })
        .ok_or_else(|| Error::InvalidInput("Input has no video stream".into()))?;
    let duration = video
        .duration
        .as_deref()
        .and_then(duration_ms)
        .or_else(|| probe.format.duration.as_deref().and_then(duration_ms))
        .ok_or_else(|| Error::Parse("Video duration could not be determined".into()))?;
    if request.to > duration {
        return Err(Error::InvalidInput(format!(
            "Requested time {} ms but video duration is {duration} ms",
            request.to
        )));
    }
    let directory = OutputDirectory::new(&request.output_dir)?;
    let mut collages = CollageBuilder::new(request.images.clone())?;
    for index in 0..count {
        let timestamp = request.from + index * request.jump;
        let mut command = ffmpeg(&path, timestamp, request.to - timestamp);
        command.args([
            "-map",
            &format!("0:{}", video.index),
            "-frames:v",
            "1",
            "-an",
            "-vf",
            &format!(
                "scale={0}:{0}:force_original_aspect_ratio=decrease,setsar=1",
                request.images.tile_long_edge
            ),
            "-threads",
            "1",
            "-f",
            "image2pipe",
            "-c:v",
            "png",
            "pipe:1",
        ]);
        let frame = run(command)?.stdout;
        if frame.is_empty() {
            return Err(Error::Parse(format!(
                "No video frame is available at {timestamp} ms"
            )));
        }
        let image = image::load_from_memory_with_format(&frame, image::ImageFormat::Png)?.to_rgb8();
        collages.push(
            directory.path(),
            Tile {
                image,
                label: timestamp_label(timestamp),
                page: None,
                timestamp_ms: Some(timestamp),
            },
        )?;
    }
    let mut warnings = Vec::new();
    let audio = if let Some(audio) = probe
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("audio"))
    {
        let mut command = ffmpeg(&path, request.from, request.to - request.from);
        command
            .args([
                "-map",
                &format!("0:{}", audio.index),
                "-vn",
                "-ac",
                "1",
                "-ar",
                "22050",
                "-c:a",
                "libmp3lame",
                "-b:a",
                "64k",
                "-threads",
                "1",
            ])
            .arg(directory.path().join("audio.mp3"));
        run(command)?;
        Some("audio.mp3".into())
    } else {
        warnings.push("Video has no audio track; no MP3 was generated".into());
        None
    };
    let images = collages.finish(directory.path())?;
    directory.finish(Manifest {
        schema: 1,
        source_format: path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase(),
        source: path,
        selection: Selection::Time {
            from: request.from,
            to: request.to,
            jump: request.jump,
            duration_ms: duration,
        },
        images,
        text: None,
        audio,
        warnings,
    })
}

fn ffmpeg(path: &Path, from: u64, duration: u64) -> Command {
    let mut command = Command::new("ffmpeg");
    command
        .args(["-v", "error", "-nostdin", "-y", "-ss", &seconds(from)])
        .arg("-i")
        .arg(path)
        .args(["-t", &seconds(duration)]);
    command
}

fn run(mut command: Command) -> Result<Output> {
    let program = command.get_program().to_string_lossy().to_string();
    let output = command.output().map_err(|error| Error::Process {
        program: program.clone(),
        message: format!("Could not start the local executable: {error}"),
    })?;
    if !output.status.success() {
        return Err(Error::Process {
            program,
            message: format!(
                "{}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
                    .trim()
                    .chars()
                    .take(2000)
                    .collect::<String>()
            ),
        });
    }
    Ok(output)
}

fn seconds(milliseconds: u64) -> String {
    format!("{}.{:03}", milliseconds / 1000, milliseconds % 1000)
}

fn duration_ms(value: &str) -> Option<u64> {
    let seconds = value.parse::<f64>().ok()?;
    (seconds.is_finite() && seconds > 0.0 && seconds < u64::MAX as f64 / 1000.0)
        .then(|| (seconds * 1000.0).ceil() as u64)
}

fn timestamp_label(milliseconds: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        milliseconds / 3_600_000,
        milliseconds / 60_000 % 60,
        milliseconds / 1000 % 60,
        milliseconds % 1000
    )
}
