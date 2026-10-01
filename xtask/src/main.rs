use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let task = args.next().unwrap_or_else(|| "help".into());

    match task.to_str() {
        Some("doctor") => {
            no_extra_args(args)?;
            doctor()
        }
        Some("fmt") => {
            let all = match args.next() {
                None => false,
                Some(arg) if arg == "--all" => true,
                Some(arg) => return Err(format!("unexpected fmt argument: {}", arg.display())),
            };
            no_extra_args(args)?;
            format(all)
        }
        Some("dev") => {
            let path = args.next().ok_or("usage: cargo xtask dev <path/to/file>")?;
            no_extra_args(args)?;
            let mut cmd = command("cargo");
            cmd.args(["run", "--package", "xtask", "--bin", "dev", "--"])
                .arg(path);
            run_command(cmd, "dev")
        }
        Some("help" | "--help" | "-h") => {
            no_extra_args(args)?;
            println!(
                "cargo xtask doctor             Check repo, toolchain, formatting, and build\n\
                 cargo xtask fmt                Format staged, unstaged, and untracked Rust files\n\
                 cargo xtask fmt --all          Format all workspace crates\n\
                 cargo xtask dev <path/to/file> Run the placeholder file CLI"
            );
            Ok(())
        }
        _ => Err(format!(
            "unknown task: {}; use cargo xtask --help",
            task.display()
        )),
    }
}

fn no_extra_args(mut args: impl Iterator<Item = OsString>) -> Result<(), String> {
    match args.next() {
        Some(arg) => Err(format!("unexpected argument: {}", arg.display())),
        None => Ok(()),
    }
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must be located in the repository root")
}

fn command(program: &str) -> Command {
    let executable = if program == "cargo" {
        env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
    } else {
        program.into()
    };
    let mut cmd = Command::new(executable);
    cmd.current_dir(repo_root());
    cmd
}

fn run_command(mut cmd: Command, label: &str) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|error| format!("cannot run {label}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} failed with {status}"))
    }
}

fn doctor() -> Result<(), String> {
    println!("Checking {}", repo_root().display());
    let mut failures = 0;

    for path in [
        "Cargo.toml",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        "xtask/Cargo.toml",
        "parser-rs/Cargo.toml",
        "crates/parser-pdf/Cargo.toml",
        "crates/parser-vid/Cargo.toml",
    ] {
        if repo_root().join(path).is_file() {
            println!("[ok] {path}");
        } else {
            eprintln!("[fail] missing {path}");
            failures += 1;
        }
    }

    let checks: &[(&str, &str, &[&str])] = &[
        (
            "Git repository",
            "git",
            &["rev-parse", "--is-inside-work-tree"],
        ),
        ("Rust compiler", "rustc", &["--version"]),
        ("Cargo", "cargo", &["--version"]),
        ("Rustfmt", "rustfmt", &["--version"]),
        ("Clippy", "cargo", &["clippy", "--version"]),
        (
            "Workspace formatting",
            "cargo",
            &["fmt", "--all", "--", "--check"],
        ),
        ("Workspace build", "cargo", &["check", "--workspace"]),
    ];

    for (label, program, args) in checks {
        match command(program).args(*args).output() {
            Ok(output) if output.status.success() => println!("[ok] {label}"),
            Ok(output) => {
                eprintln!("[fail] {label}: {}", output.status);
                eprint!("{}", String::from_utf8_lossy(&output.stdout));
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                failures += 1;
            }
            Err(error) => {
                eprintln!("[fail] {label}: {error}");
                failures += 1;
            }
        }
    }

    if failures == 0 {
        println!("Repository is ready.");
        Ok(())
    } else {
        Err(format!("doctor found {failures} failed check(s)"))
    }
}

fn format(all: bool) -> Result<(), String> {
    if all {
        let mut cmd = command("cargo");
        cmd.args(["fmt", "--all"]);
        return run_command(cmd, "workspace formatting");
    }

    let mut files = BTreeSet::new();
    for args in [
        &["diff", "--name-only", "--diff-filter=ACMR", "-z"][..],
        &[
            "diff",
            "--cached",
            "--name-only",
            "--diff-filter=ACMR",
            "-z",
        ][..],
        &["ls-files", "--others", "--exclude-standard", "-z"][..],
    ] {
        let output = command("git")
            .args(args)
            .output()
            .map_err(|error| format!("cannot inspect Git changes: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "cannot inspect Git changes: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        for path in output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
        {
            let path = git_path(path)?;
            if path.extension().is_some_and(|extension| extension == "rs")
                && repo_root().join(&path).is_file()
            {
                files.insert(path);
            }
        }
    }

    if files.is_empty() {
        println!("No changed Rust files to format.");
        return Ok(());
    }

    println!("Formatting {} changed Rust file(s).", files.len());
    let mut cmd = command("rustfmt");
    cmd.args(["--edition", "2024", "--config", "skip_children=true", "--"])
        .args(files);
    run_command(cmd, "changed-file formatting")
}

fn git_path(bytes: &[u8]) -> Result<PathBuf, String> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(OsString::from_vec(bytes.to_vec()).into())
    }
    #[cfg(not(unix))]
    {
        String::from_utf8(bytes.to_vec())
            .map(PathBuf::from)
            .map_err(|error| format!("invalid Git path: {error}"))
    }
}
