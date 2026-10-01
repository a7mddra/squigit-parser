pub use parser_common::{
    CollageInfo, Error, ImageOptions, Manifest, ParseOutput, Result, Selection, TileInfo,
};
pub use parser_pdf::PdfRequest;
pub use parser_vid::VideoRequest;
use serde_json::{Value, json};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Document,
    Video,
}

pub fn file_kind(path: &Path) -> Result<FileKind> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| {
            Error::InvalidInput(format!(
                "Missing or invalid file extension: {}",
                path.display()
            ))
        })?
        .to_ascii_lowercase();

    match extension.as_str() {
        "pdf" | "docx" | "xlsx" | "pptx" => Ok(FileKind::Document),
        "mov" | "mp4" | "m4v" | "mkv" | "avi" | "webm" | "mpg" | "mpeg" | "wmv" | "flv" | "mts"
        | "m2ts" | "3gp" | "3g2" | "ogv" | "vob" => Ok(FileKind::Video),
        _ => Err(Error::InvalidInput(format!(
            "Unsupported file extension: .{extension}"
        ))),
    }
}

pub fn parse_pdf(request: PdfRequest) -> Result<ParseOutput> {
    parser_pdf::parse(request)
}

pub fn parse_video(request: VideoRequest) -> Result<ParseOutput> {
    if file_kind(&request.path)? != FileKind::Video {
        return Err(Error::InvalidInput(
            "parse_video requires a video file".into(),
        ));
    }
    parser_vid::parse(request)
}

pub fn execute_tool(name: &str, arguments: Value) -> Result<ParseOutput> {
    match name {
        "parse_pdf" => parse_pdf(serde_json::from_value(arguments)?),
        "parse_video" => parse_video(serde_json::from_value(arguments)?),
        _ => Err(Error::InvalidInput(format!("Unknown parser tool: {name}"))),
    }
}

pub fn tool_definitions() -> Vec<Value> {
    let image_options = json!({"type":"object", "additionalProperties":false, "properties":{
        "tile_long_edge":{"type":"integer","minimum":256,"maximum":2048,"default":1024},
        "quality":{"type":"number","minimum":35,"maximum":100,"default":75},
        "max_bytes":{"type":"integer","minimum":32768,"maximum":4194304,"default":524288}
    }});
    vec![
        json!({"type":"function","function":{
            "name":"parse_pdf",
            "description":"Read a local PDF, DOCX, XLSX, or PPTX page range. Returns best-effort text and numbered WebP collages with at most six pages each, three across and two down. from/to are inclusive, one-based page numbers, with at most 300 pages per call. Office ranges refer to the converted PDF. Output directory must be new. No network requests or OCR.",
            "parameters":{"type":"object","additionalProperties":false,"properties":{
                "path":{"type":"string","description":"Local document path"},
                "from":{"type":"integer","minimum":1},
                "to":{"type":"integer","minimum":1},
                "output_dir":{"type":"string","description":"New output directory"},
                "images":image_options.clone()
            },"required":["path","from","to","output_dir"]}
        }}),
        json!({"type":"function","function":{
            "name":"parse_video",
            "description":"Read a local video clip. from is inclusive and to is exclusive, both in milliseconds. jump is the sampling interval in milliseconds. Frames are sampled at from + n*jump while less than to, grouped into WebP collages of at most six, with timestamp labels. Also extracts only the selected clip's audio to MP3 when a track exists. Increase jump for a long overview; decrease it for detailed action. At most 300 frames per call. Output directory must be new. No transcription or network requests.",
            "parameters":{"type":"object","additionalProperties":false,"properties":{
                "path":{"type":"string","description":"Local video path"},
                "from":{"type":"integer","minimum":0},
                "to":{"type":"integer","minimum":1},
                "jump":{"type":"integer","minimum":1,"description":"Milliseconds between selected frames; 2000 selects six frames from a 12000 ms clip"},
                "output_dir":{"type":"string","description":"New output directory"},
                "images":image_options
            },"required":["path","from","to","jump","output_dir"]}
        }}),
    ]
}
