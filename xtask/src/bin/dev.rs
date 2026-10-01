use clap::Parser;
use parser_rs::{FileKind, ImageOptions, PdfRequest, VideoRequest};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "squigit-parser",
    about = "Local PDF/Office and video range parsing into compact WebP collages"
)]
struct Arguments {
    path: Option<PathBuf>,
    #[arg(
        long,
        help = "Start page (default 1) or start time in milliseconds (default 0)"
    )]
    from: Option<u64>,
    #[arg(
        long,
        help = "Inclusive last PDF page or exclusive video end time in milliseconds"
    )]
    to: Option<u64>,
    #[arg(long, help = "Video sampling interval in milliseconds")]
    jump: Option<u64>,
    #[arg(long, help = "New output directory")]
    output: Option<PathBuf>,
    #[arg(long, default_value_t = 1024)]
    tile_long_edge: u32,
    #[arg(long, default_value_t = 75.0)]
    quality: f32,
    #[arg(long, default_value_t = 512)]
    max_image_kib: u32,
    #[arg(long, help = "Print the two AI tool definitions as JSON")]
    tools: bool,
}

fn main() -> ExitCode {
    match run(Arguments::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    if arguments.tools {
        println!(
            "{}",
            serde_json::to_string_pretty(&parser_rs::tool_definitions())?
        );
        return Ok(());
    }
    let path = arguments
        .path
        .ok_or("Provide a local file path or use --tools")?;
    let to = arguments.to.ok_or("Provide --to")?;
    let output_dir = arguments
        .output
        .ok_or("Provide --output naming a new directory")?;
    let images = ImageOptions {
        tile_long_edge: arguments.tile_long_edge,
        quality: arguments.quality,
        max_bytes: usize::try_from(arguments.max_image_kib)?
            .checked_mul(1024)
            .ok_or("Image budget is too large")?,
    };
    let output = match parser_rs::file_kind(&path)? {
        FileKind::Document => {
            if arguments.jump.is_some() {
                return Err("--jump is only used for video".into());
            }
            parser_rs::parse_pdf(PdfRequest {
                path,
                from: u32::try_from(arguments.from.unwrap_or(1))?,
                to: u32::try_from(to)?,
                output_dir,
                images,
            })?
        }
        FileKind::Video => parser_rs::parse_video(VideoRequest {
            path,
            from: arguments.from.unwrap_or(0),
            to,
            jump: arguments
                .jump
                .ok_or("Video requires --jump in milliseconds")?,
            output_dir,
            images,
        })?,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
