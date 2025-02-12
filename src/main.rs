use std::{
    env::current_dir,
    fs::{copy, remove_dir_all, rename},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use owo_colors::OwoColorize;
use walkdir::{DirEntry, WalkDir};

/// Flattens a folder structure
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Target directory to flatten (defaults to current directory)
    target: Option<String>,

    /// Delete original files after flattening
    #[arg(short, long)]
    delete: bool,

    /// Rename files if a file with the same name already exists
    #[arg(short, long)]
    rename: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let target_dir = match args.target {
        Some(dir) => PathBuf::from(dir),
        None => current_dir().context("Failed to get current directory")?,
    };
    let mut folders = Vec::new();
    for entry in WalkDir::new(&target_dir)
        .into_iter()
        .filter_entry(|e| !is_root_file(e, &target_dir))
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            let mut new_path = target_dir.join(file_name);
            if new_path.exists() {
                if args.rename {
                    let mut counter = 1;
                    while new_path.exists() {
                        let new_file_name = format!(
                            "{}_{}{}",
                            new_path.file_stem().unwrap().to_string_lossy(),
                            counter,
                            new_path.extension().map_or("".to_string(), |ext| format!(
                                ".{}",
                                ext.to_string_lossy()
                            ))
                        );
                        new_path = target_dir.join(new_file_name);
                        counter += 1;
                    }
                    eprintln!(
                        "Renaming '{}' to '{}' to avoid collision.",
                        path.display().yellow(),
                        new_path.display().blue()
                    );
                } else {
                    eprintln!(
                        "{} File '{}' already exists in target directory. Skipping.",
                        " Warning:".black().on_yellow(),
                        new_path.display().blue()
                    );
                    continue;
                }
            }
            if args.delete {
                rename(path, &new_path).with_context(|| {
                    format!(
                        "Failed to move file from '{}' to '{}'",
                        path.display(),
                        new_path.display()
                    )
                })?;
            } else {
                copy(path, &new_path).with_context(|| {
                    format!(
                        "Failed to copy file from '{}' to '{}'",
                        path.display(),
                        new_path.display()
                    )
                })?;
            }
        } else if args.delete && path != target_dir {
            folders.push(path.to_owned());
        }
    }
    if args.delete {
        for folder in folders {
            remove_dir_all(folder)?;
        }
    }
    Ok(())
}

fn is_root_file(entry: &DirEntry, root_dir: &Path) -> bool {
    entry.file_type().is_file() && entry.path().parent() == Some(root_dir)
}
