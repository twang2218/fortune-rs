pub mod metadata;

use anyhow::Result;
use clap::Parser;
use glob::glob;
use metadata::{CookieMetadata, Quote};
use rand::seq::SliceRandom;
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

// define a macro for replace println! with debug output if given args.debug is true
macro_rules! debug_println {
    ($args:expr, $($arg:tt)*) => {
        if $args.debug {
            println!($($arg)*);
        }
    };
}

#[derive(Parser)]
#[command(
    version,
    about = "A Rust implementation of the classic fortune program"
)]
struct Args {
    /// Path to fortune file or directory
    #[arg(default_value = "data")]
    path: String,

    /// Choose from all lists of maxims, both offensive and not
    #[arg(short = 'a')]
    all: bool,

    /// Show the cookie file from which the fortune came
    #[arg(short = 'c')]
    show_file: bool,

    /// Enable additional debugging output
    #[arg(short = 'D', default_value="false")]
    debug: bool,

    /// Consider all fortune files to be of equal size
    #[arg(short = 'e')]
    equal_size: bool,

    /// Print out the list of files which would be searched
    #[arg(short = 'f')]
    list_files: bool,

    /// Ignore case for -m patterns
    #[arg(short = 'i')]
    ignore_case: bool,

    /// Long dictums only
    #[arg(short = 'l')]
    long_only: bool,

    /// Print out all fortunes which match the pattern
    #[arg(short = 'm')]
    pattern: Option<String>,

    /// Set the longest fortune length considered to be "short"
    #[arg(short = 'n', default_value = "160")]
    length: usize,

    /// Short apothegms only
    #[arg(short = 's')]
    short_only: bool,

    /// Choose only from potentially offensive aphorisms
    #[arg(short = 'o')]
    offensive: bool,

    /// Don't translate UTF-8 fortunes to the locale
    #[arg(short = 'u')]
    no_utf8_translate: bool,

    /// Wait before termination based on message length
    #[arg(short = 'w')]
    wait: bool,

    /// Only load cookies without loading metadata
    #[arg(short = 't', long)]
    text: bool,
}

fn find_cookies_files_with_metadata(path: &Path, args: &Args) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut pattern = format!("{}/**/*.dat", path.to_string_lossy());
    if args.offensive {
        pattern = format!("{}/**/off/*.dat", path.to_string_lossy());
    }

    let mut cookie_files: Vec<PathBuf> = Vec::new();
    for entry in glob(&pattern).expect("Failed to find fortune cookies database files") {
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

fn find_cookies_with_metadata(path: &Path, args: &Args) -> Result<Vec<CookieMetadata>> {
    let files = find_cookies_files_with_metadata(path, args)?;
    let mut cookies: Vec<CookieMetadata> = Vec::new();
    for file in files {
        let mut data = CookieMetadata::from_dat(&file.with_extension("dat").to_string_lossy());
        if args.pattern.is_some() || args.short_only || args.long_only {
            // load the quotes' content for pattern matching
            data.load_from_cookie_file(&file.to_string_lossy());
        }
        cookies.push(data);
    }
    Ok(cookies)
}

fn find_cookies_files_with_text(path: &Path, args: &Args) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut pattern = format!("{}/**/*", path.to_string_lossy());
    if args.offensive {
        pattern = format!("{}/**/off/*", path.to_string_lossy());
    }

    let mut cookie_files: Vec<PathBuf> = Vec::new();
    for entry in glob(&pattern).expect("Failed to find cookies files") {
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

fn find_cookies_with_text(path: &Path, args: &Args) -> Result<Vec<CookieMetadata>> {
    let files = find_cookies_files_with_text(path, args)?;
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

fn filter_quotes<'a>(quotes: Vec<(PathBuf, &'a Quote)>, args: &Args) -> Vec<(PathBuf, &'a Quote)> {
    let mut filtered = quotes;

    // Filter by length
    if args.short_only {
        filtered = filtered
            .into_iter()
            .filter(|(_, q)| q.content.len() <= args.length)
            .collect();
    } else if args.long_only {
        filtered = filtered
            .into_iter()
            .filter(|(_, q)| q.content.len() > args.length)
            .collect();
    }

    // Filter by pattern if specified
    if let Some(pattern) = &args.pattern {
        let re = if args.ignore_case {
            Regex::new(&format!("(?i){}", pattern)).unwrap()
        } else {
            Regex::new(pattern).unwrap()
        };
        filtered = filtered
            .into_iter()
            .filter(|(_, q)| re.is_match(&q.content))
            .collect();
    }

    filtered
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.path);

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Debug output if requested
    if args.debug {
        println!("Arguments: {:?}", std::env::args().collect::<Vec<_>>());
        println!("Path: {}", path.display());
    }

    // List files if requested
    if args.list_files {
        let files = if args.text {
            find_cookies_files_with_text(path, &args)?
        } else {
            find_cookies_files_with_metadata(path, &args)?
        };
        println!("Fortune files:");
        for file in files {
            println!("    {}", file.display());
        }
        return Ok(());
    }

    // Collect all fortune files
    let cookies = if args.text {
        find_cookies_with_text(path, &args)?
    } else {
        find_cookies_with_metadata(path, &args)?
    };

    let quotes: Vec<(PathBuf, &Quote)> = cookies
        .iter()
        .flat_map(|c| c.quotes.iter().map(|q| (c.path.clone(), q)))
        .collect();

    if quotes.is_empty() {
        anyhow::bail!("No fortune cookies found in directory: {}", path.display());
    }

    debug_println!(args, "Found {} quotes", quotes.len());
    // Filter quotes based on arguments
    if args.pattern.is_some() {
        debug_println!(args, "Pattern: {:?}", args.pattern);
    }
    let filtered_quotes = filter_quotes(quotes, &args);
    debug_println!(args, "Filtered quotes: {}", filtered_quotes.len());

    if filtered_quotes.is_empty() {
        anyhow::bail!("No matching fortunes found");
    }

    // If pattern matching is enabled, show all matches
    if args.pattern.is_some() {
        for (file, quote) in filtered_quotes {
            if args.show_file {
                println!("({})\n", file.display());
            }
            println!("{}", quote.content);
            println!("%");
        }
    } else {
        // Otherwise, choose a random fortune
        let (file, quote) = filtered_quotes.choose(&mut rand::thread_rng()).unwrap();
        if args.show_file {
            println!("({})\n", file.display());
        }
        println!("{}", quote.content);

        // Wait if req|uested
        if args.wait {
            let wait_time = (quote.content.len() as u64 + 999) / 1000;
            thread::sleep(Duration::from_secs(wait_time));
        }
    }

    Ok(())
}
