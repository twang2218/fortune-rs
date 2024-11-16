pub mod metadata;

use anyhow::{Result};
use glob::glob;
use clap::Parser;
use metadata::{CookieMetadata, Quote};
use rand::seq::SliceRandom;
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

// struct Fortune {
//     text: String,
//     source: PathBuf,
// }

// fn read_fortunes(path: &Path) -> Result<Vec<Fortune>> {
//     let content = std::fs::read_to_string(path)
//         .with_context(|| format!("Failed to read fortune file: {}", path.display()))?;

//     // Split fortunes by % on a line by itself
//     let fortunes: Vec<Fortune> = content
//         .split("\n%\n")
//         .map(|s| s.trim().to_string())
//         .filter(|s| !s.is_empty())
//         .map(|text| Fortune {
//             text,
//             source: path.to_path_buf(),
//         })
//         .collect();

//     Ok(fortunes)
// }

fn find_cookie_files(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut cookie_files = Vec::new();
    for entry in glob(&format!("{}/**/*.dat", path.to_string_lossy())).expect("Failed to find fortune database files") {
        let path = entry?;
        if path.is_file() {
            let mut cookie_file = path.clone();
            cookie_file.set_extension("");
            if cookie_file.exists() {
                // println!("cookie_file: {:?}", cookie_file);
                cookie_files.push(cookie_file);
            }
        }
    }
    if cookie_files.is_empty() {
        anyhow::bail!("No fortune files found in directory: {}", path.display());
    }

    Ok(cookie_files)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.path);

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Collect all fortune files
    let cookie_files = find_cookie_files(path)?;

    // Read all fortunes from all files
    // let mut all_cookies: Vec<Fortune> = Vec::new();
    // for file in &cookie_files {
    //     let fortunes = read_fortunes(file)?;
    //     all_cookies.extend(fortunes);
    // }

    let mut all_cookies: Vec::<(PathBuf, Quote)> = Vec::new();
    for file in &cookie_files {
        let data = CookieMetadata::from_dat(&file.with_extension("dat").to_string_lossy());
        for quote in data.quotes {
            all_cookies.push((file.to_path_buf(), quote ));
        }
    }

    if all_cookies.is_empty() {
        anyhow::bail!("No fortunes found in any of the files");
    }

    // Select and display a random fortune
    let mut rng = rand::thread_rng();
    if let Some((path, quote)) = all_cookies.choose(&mut rng) {
        if args.file {
            println!("({})\n", path.display());
        }
        let mut data = CookieMetadata::default();
        data.load_from_cookie_file(&path.to_string_lossy());
        let quote = data.quotes.iter().find(|q| q.offset == quote.offset).unwrap();
        println!("{}", quote.content);
    }

    Ok(())
}
