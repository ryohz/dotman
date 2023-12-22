use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

use anyhow::{Context, Result};
use xxhash_rust::xxh32;

pub fn is_exists(path: &PathBuf) -> Result<bool> {
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => Ok(false),
            _ => Err(e.into()),
        },
    }
}

pub fn dir_hash(path: &std::path::Path) -> Result<u32> {
    let mut h = 0;

    fn hash(h: &mut u32, path: &std::path::Path) {
        for result in ignore::Walk::new(path) {
            let entry = result.context("no .ignore or .gitignore file").unwrap();
            if entry
                .file_type()
                .context("failed to get file type")
                .unwrap()
                .is_file()
            {
                let p = entry.path();
                let f = fs::File::open(p).unwrap();
                let mut reader = io::BufReader::new(f);
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).unwrap();
                *h = xxh32::xxh32(&buffer, *h);
            } else if entry.file_type().unwrap().is_dir() && *entry.path() != *path {
                hash(h, entry.path());
            }
        }
    }

    hash(&mut h, path);

    Ok(h)
}

pub fn copy_directory(src: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.context("failed to get entry of a directory")?;
        let entry_path = entry.path();

        let relative_path = entry_path
            .strip_prefix(src)
            .context("failed to strip prefix")?;

        let destination = dest.join(relative_path);

        if entry_path.is_dir() {
            fs::create_dir_all(&destination)
                .with_context(|| format!("failed to create directory {}", destination.display()))?;
        } else {
            fs::copy(entry_path, &destination).with_context(|| {
                format!(
                    "failed to copy file {} to {}",
                    entry_path.display(),
                    destination.display()
                )
            })?;
        }
    }

    Ok(())
}
