pub mod metadata;

use anyhow::Result;
use clap::Parser;
use glob::glob;
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

    /// Only load cookies without loading metadata
    /// This is useful for loading cookies from a directory without strfile processed files
    #[arg(short, long)]
    text: bool,
}

fn find_cookies_files_with_metadata(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut cookie_files: Vec<PathBuf> = Vec::new();
    for entry in glob(&format!("{}/**/*.dat", path.to_string_lossy()))
        .expect("Failed to find fortune cookies database files")
    {
        let path = entry?;
        if path.is_file() {
            let mut cookie_file = path.clone();
            cookie_file.set_extension("");
            if cookie_file.exists() {
                cookie_files.push(cookie_file);
            }
        }
    }
    if cookie_files.is_empty() {
        anyhow::bail!(
            "No fortune cookies files found in directory: {}",
            path.display()
        );
    }

    Ok(cookie_files)
}

fn find_cookies_with_metadata(path: &Path) -> Result<Vec<CookieMetadata>> {
    let files = find_cookies_files_with_metadata(path)?;
    let mut cookies: Vec<CookieMetadata> = Vec::new();
    for file in files {
        let data = CookieMetadata::from_dat(&file.with_extension("dat").to_string_lossy());
        cookies.push(data);
    }
    Ok(cookies)
}

fn find_cookies_files_with_text(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut cookie_files: Vec<PathBuf> = Vec::new();
    for entry in
        glob(&format!("{}/**/*", path.to_string_lossy())).expect("Failed to find coookies files")
    {
        let path = entry?;
        if path.is_file() && path.extension().unwrap_or_default() != "dat" {
            cookie_files.push(path);
        }
    }
    if cookie_files.is_empty() {
        anyhow::bail!("No fortune files found in directory: {}", path.display());
    }

    Ok(cookie_files)
}

fn find_cookies_with_text(path: &Path) -> Result<Vec<CookieMetadata>> {
    let files = find_cookies_files_with_text(path)?;
    let mut cookies: Vec<CookieMetadata> = Vec::new();
    for file in files {
        let mut data = CookieMetadata::default();
        data.load_from_cookie_file(&file.to_string_lossy());
        // validate the data
        if data.quotes.is_empty() || data.num_quotes < 2 {
            continue;
        }
        cookies.push(data);
    }
    Ok(cookies)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.path);

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Collect all fortune files
    if args.text {
        // Load cookies without metadata
        let cookies = find_cookies_with_text(path)?;
        let quotes: Vec<(PathBuf, &Quote)> = cookies.iter().flat_map(|c| c.quotes.iter().map(|q| (c.path.clone(), q))).collect();
        if quotes.is_empty() {
            anyhow::bail!("No fortune cookies found in directory: {}", path.display());
        }
        let (file, quote) = quotes.choose(&mut rand::thread_rng()).unwrap();
        if args.file {
            println!("({})\n", file.display());
        }
        println!("{}", quote.content);
    } else {
        // Load cookies with metadata
        let cookies = find_cookies_with_metadata(path)?;
        let quotes: Vec<(PathBuf, &Quote)> = cookies.iter().flat_map(|c| c.quotes.iter().map(|q| (c.path.clone(), q))).collect();
        if quotes.is_empty() {
            anyhow::bail!("No fortune cookies found in directory: {}", path.display());
        }
        let (file, quote) = quotes.choose(&mut rand::thread_rng()).unwrap();
        if args.file {
            println!("({})\n", file.display());
        }
        let mut data = CookieMetadata::default();
        data.load_from_cookie_file(&file.to_string_lossy());
        let quote = data
            .quotes
            .iter()
            .find(|q| q.offset == quote.offset)
            .unwrap();
        println!("{}", quote.content);
    };

    Ok(())
}
