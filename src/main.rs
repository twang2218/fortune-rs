use anyhow::{Context, Result};
use clap::Parser;
use rand::seq::SliceRandom;
use std::fs::{self, read_to_string};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    version,
    about = "A Rust implementation of the classic fortune program"
)]
struct Args {
    /// Path to fortune file or directory
    #[arg(default_value = "data")]
    path: String,

    /// Show the source file of the fortune
    #[arg(short, long)]
    file: bool,
}

struct Fortune {
    text: String,
    source: PathBuf,
}

fn read_fortunes(path: &Path) -> Result<Vec<Fortune>> {
    let content = read_to_string(path)
        .with_context(|| format!("Failed to read fortune file: {}", path.display()))?;

    // Split fortunes by % on a line by itself
    let fortunes: Vec<Fortune> = content
        .split("\n%\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|text| Fortune {
            text,
            source: path.to_path_buf(),
        })
        .collect();

    Ok(fortunes)
}

fn collect_fortune_files(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut fortune_files = Vec::new();
    for entry in fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && !path.to_string_lossy().ends_with(".dat") {
            fortune_files.push(path);
        }
    }

    if fortune_files.is_empty() {
        anyhow::bail!("No fortune files found in directory: {}", path.display());
    }

    Ok(fortune_files)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.path);

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Collect all fortune files
    let fortune_files = collect_fortune_files(path)?;

    // Read all fortunes from all files
    let mut all_fortunes = Vec::new();
    for file in &fortune_files {
        let fortunes = read_fortunes(file)?;
        all_fortunes.extend(fortunes);
    }

    if all_fortunes.is_empty() {
        anyhow::bail!("No fortunes found in any of the files");
    }

    // Select and display a random fortune
    let mut rng = rand::thread_rng();
    if let Some(fortune) = all_fortunes.choose(&mut rng) {
        if args.file {
            println!("({})\n", fortune.source.display());
        }
        println!("{}", fortune.text);
    }

    Ok(())
}
