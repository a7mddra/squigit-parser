use std::env;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: dev <path/to/file>");
        return ExitCode::FAILURE;
    };

    if path == "--help" || path == "-h" {
        println!("usage: dev <path/to/file>");
        return ExitCode::SUCCESS;
    }

    if args.next().is_some() {
        eprintln!("expected exactly one file path; quote paths containing spaces");
        return ExitCode::FAILURE;
    }

    match parser_rs::parse(Path::new(&path)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
