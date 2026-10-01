use std::path::Path;

/// Route a path to a placeholder parser using its extension, without reading it.
pub fn parse(path: &Path) -> Result<(), String> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| format!("missing or invalid file extension: {}", path.display()))?
        .to_ascii_lowercase();

    match extension.as_str() {
        "pdf" | "doc" | "docx" | "docm" | "dot" | "dotx" | "dotm" | "xls" | "xlsx" | "xlsm"
        | "xlsb" | "xlt" | "xltx" | "xltm" | "ppt" | "pptx" | "pptm" | "pot" | "potx" | "potm"
        | "pps" | "ppsx" | "ppsm" => parser_pdf::parse(path),
        "mp3" | "wav" | "flac" | "m4a" | "aac" | "ogg" | "opus" | "aif" | "aiff" | "wma"
        | "ac3" | "amr" => parser_aud::parse(path),
        "mov" | "mp4" | "m4v" | "mkv" | "avi" | "webm" | "mpg" | "mpeg" | "wmv" | "flv" | "mts"
        | "m2ts" | "3gp" | "3g2" | "ogv" | "vob" => parser_vid::parse(path),
        _ => return Err(format!("unsupported file extension: .{extension}")),
    }

    Ok(())
}
