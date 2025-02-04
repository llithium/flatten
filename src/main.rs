use std::{
    env::current_dir,
    fs::{copy, read_dir, remove_dir, rename},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use owo_colors::OwoColorize;
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
    flatten_directory(&target_dir, &target_dir, args.delete, args.rename)?;
    Ok(())
}

fn flatten_directory(
    root_dir: &Path,
    target_dir: &Path,
    delete: bool,
    rename_files: bool,
) -> Result<()> {
    let entries = read_dir(target_dir).context("Failed to read target directory")?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            flatten_directory(root_dir, &path, delete, rename_files)?;

            let sub_entries = read_dir(&path).context("Failed to read subdirectory")?;
            for sub_entry in sub_entries {
                let sub_entry = sub_entry.context("Failed to read subdirectory entry")?;
                let sub_path = sub_entry.path();

                if sub_path.is_file() {
                    let file_name = sub_path.file_name().unwrap();
                    let mut new_path = root_dir.join(file_name);

                    if new_path.exists() {
                        if rename_files {
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
                                new_path = root_dir.join(new_file_name);
                                counter += 1;
                            }
                            eprintln!(
                                "Renaming '{}' to '{}' to avoid collision.",
                                sub_path.display().yellow(),
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
                    if delete {
                        rename(&sub_path, &new_path).with_context(|| {
                            format!(
                                "Failed to move file from '{}' to '{}'",
                                sub_path.display(),
                                new_path.display()
                            )
                        })?;
                    } else {
                        copy(&sub_path, &new_path).with_context(|| {
                            format!(
                                "Failed to copy file from '{}' to '{}'",
                                sub_path.display(),
                                new_path.display()
                            )
                        })?;
                    }
                }
            }

            if delete {
                remove_dir(&path)
                    .with_context(|| format!("Failed to delete directory '{}'", path.display()))?;
            }
        }
    }
    Ok(())
}
