use crate::{Error, Manifest, ParseOutput, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub fn source_path(path: &Path) -> Result<PathBuf> {
    let path = fs::canonicalize(path)?;
    let metadata = fs::metadata(&path)?;
    if !metadata.is_file() {
        return Err(Error::InvalidInput("Input must be a local file".into()));
    }
    Ok(path)
}

pub struct OutputDirectory {
    destination: PathBuf,
    staging: TempDir,
}

impl OutputDirectory {
    pub fn new(path: &Path) -> Result<Self> {
        let absolute = std::path::absolute(path)?;
        let name = absolute
            .file_name()
            .ok_or_else(|| Error::InvalidInput("Output must name a new directory".into()))?;
        let parent = absolute
            .parent()
            .ok_or_else(|| Error::InvalidInput("Output must have a parent directory".into()))?;
        fs::create_dir_all(parent)?;
        let destination = fs::canonicalize(parent)?.join(name);
        if destination.try_exists()? {
            return Err(Error::InvalidInput(format!(
                "Output already exists; choose a new directory: {}",
                destination.display()
            )));
        }
        let staging = tempfile::Builder::new()
            .prefix(".parser-")
            .tempdir_in(parent)?;
        Ok(Self {
            destination,
            staging,
        })
    }

    pub fn path(&self) -> &Path {
        self.staging.path()
    }

    pub fn finish(self, manifest: Manifest) -> Result<ParseOutput> {
        fs::write(
            self.path().join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        fs::create_dir(&self.destination)?;
        let mut moved = Vec::new();
        let result = (|| -> Result<()> {
            for entry in fs::read_dir(self.path())? {
                let entry = entry?;
                let destination = self.destination.join(entry.file_name());
                fs::rename(entry.path(), &destination)?;
                moved.push(destination);
            }
            Ok(())
        })();
        if let Err(error) = result {
            for path in moved {
                let _ = fs::remove_file(path);
            }
            let _ = fs::remove_dir(&self.destination);
            return Err(error);
        }
        Ok(ParseOutput {
            output_dir: self.destination.clone(),
            manifest,
        })
    }
}
