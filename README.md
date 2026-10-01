# squigit-parser

Two local Rust tools produce small WebP collages from document pages or sampled
video frames. They write results to a new directory and return a JSON manifest.
This repository runs independently of Squigit and makes no provider requests.

## Tools

| Tool | Input and range | Output |
| --- | --- | --- |
| `parse_pdf` | PDF, DOCX, XLSX, PPTX; inclusive pages `from..=to`, starting at 1 | WebP collages + `text.txt` |
| `parse_video` | Video; milliseconds `[from, to)`, sampled every `jump` milliseconds | WebP collages + `audio.mp3` when audio exists |

Each collage contains at most six tiles, ordered left to right, then top to
bottom, with at most three columns and two rows. Document tiles carry page
numbers; video tiles carry timestamps relative to the original video. A partial
last collage contains only the remaining tiles. A call can select at most 300
pages or frames; use smaller ranges or a larger video `jump` for longer inputs.

PDF pages are rendered locally with [Hayro](https://github.com/LaurenzV/hayro).
Text extraction is best effort: scanned pages still produce images, and failures
are listed in the manifest. No OCR is performed. Office documents are converted
in Rust with [office2pdf 0.8.0](https://github.com/developer0hye/office2pdf), then
use the same PDF pipeline. Office page numbers refer to that conversion; fonts
and complex layouts can differ from Microsoft Office. Legacy binary Office
formats (`.doc`, `.xls`, `.ppt`) are unsupported.

Video uses local FFmpeg and FFprobe. For a 12-second overview, `from=0`,
`to=12000`, `jump=2000` selects six frames at 0, 2, 4, 6, 8, and 10 seconds.
Seek targets follow the source's available frames; they are not a claim of
sub-frame accuracy. Audio is clipped to the same requested range and encoded as
mono, 22.05 kHz, 64 kbit/s MP3. MP3 duration includes small encoder padding.
Silent videos return `audio: null` with a warning.

## Run against the assets

Install Rust with Rustup. Install FFmpeg/FFprobe with PNG encoding and
`libmp3lame` available on `PATH` for video. PDF and Office rendering require no
external document converter. The first build compiles the Office renderer and
can take several minutes; limiting build jobs helps on smaller machines.

```sh
CARGO_BUILD_JOBS=1 cargo build --workspace

cargo xtask dev assets/example.pdf --from 1 --to 20 --output output/pdf-full
cargo xtask dev assets/example.pdf --from 7 --to 13 --output output/pdf-range
cargo xtask dev assets/example.docx --from 1 --to 1 --output output/office-first
cargo xtask dev assets/example.mp4 --from 0 --to 12000 --jump 2000 --output output/video-12s

cargo xtask dev --tools
```

Quote paths containing spaces. `--from` defaults to 1 for documents and 0 for
video; `--to` and `--output` are required. Video also requires `--jump`.
The CLI can run directly with
`cargo run --package xtask --bin dev -- <path> <options>`.
Successful calls print JSON to stdout; failures print an error to stderr and
return a nonzero status.

Output directories must not already exist. Parsing uses a temporary sibling
directory and publishes completed files only after success; failed calls clean
up their intermediate files. Existing results are never overwritten.
`output/` is ignored by Git.

## Image size controls

Defaults are a 1024-pixel long edge per tile, WebP quality 75, and a **512 KiB
maximum per collage**. The encoder lowers quality, then resolution as needed
to meet that budget. If the budget cannot be met at the minimum allowed size,
the call fails with guidance to increase it.

```sh
cargo xtask dev assets/example.pdf --from 1 --to 6 --output output/pdf-detail \
  --tile-long-edge 1536 --quality 85 --max-image-kib 1024
```

Rust/JSON `images` options use `tile_long_edge` (256–2048), `quality` (35–100),
and `max_bytes` (32 KiB–4 MiB). Text accompanies document collages so small print
does not depend entirely on compressed pixels. Higher resolution is useful for
fine diagrams or scanned text. Combining pages reduces uploads; vision token
cost still depends on the receiving model and image resolution.

## Rust and AI tool API

`parser-rs` exports `parse_pdf(PdfRequest)`, `parse_video(VideoRequest)`,
`tool_definitions()` (exactly two function definitions), and
`execute_tool(name, arguments)` for JSON tool dispatch. The requests are:

```json
{
  "path": "/absolute/report.pdf",
  "from": 2,
  "to": 7,
  "output_dir": "/absolute/output/report-pages",
  "images": { "tile_long_edge": 1024, "quality": 75, "max_bytes": 524288 }
}
```

```json
{
  "path": "/absolute/clip.mp4",
  "from": 5000,
  "to": 17000,
  "jump": 2000,
  "output_dir": "/absolute/output/clip-range"
}
```

These synchronous APIs perform local filesystem, rendering, and process work;
an asynchronous host should call them on a blocking worker. Unknown request
fields and invalid ranges are rejected.

`ParseOutput` returns `output_dir` plus `manifest`. The same manifest is saved as
`manifest.json`: source path/format, selected range, total pages or duration,
collage filenames/dimensions/bytes/quality, each tile's pixel rectangle and page
or timestamp, text/audio filenames, and warnings. Artifact filenames are relative
to `output_dir`. This gives a future caller enough information to load the actual
images and text without remote URIs.

## Audio transcription boundary

This phase produces the selected MP3 locally. The future Squigit host will
enforce paid-only transcription and choose a live transcription-capable model.
OpenRouter's documented endpoint is
[`/api/v1/audio/transcriptions`](https://openrouter.ai/docs/guides/overview/multimodal/stt);
this repository does not accept API keys or make that request.

## Development

- `parser-rs/`: public facade, file dispatch, AI tool definitions.
- `crates/parser-pdf/`: document conversion, page rendering and text extraction.
- `crates/parser-vid/`: probing, frame sampling and audio extraction.
- `crates/parser-common/`: shared manifests, WebP layouts and output handling.
- `xtask/`: development CLI and repository tasks.

```sh
cargo xtask doctor
cargo xtask fmt
cargo xtask fmt --all
cargo clippy --workspace -- -D warnings
```

`doctor` checks repository files, Git, Rust, FFmpeg/FFprobe, formatting, and
`cargo check --workspace`. `fmt` formats changed Rust files; `--all` formats
every workspace crate. No automated test suite is configured.
