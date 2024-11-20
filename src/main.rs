pub mod metadata;

use anyhow::{Ok, Result};
use clap::Parser;
use glob::glob;
use metadata::{CookieMetadata, Quote};
use rand::seq::SliceRandom;
use regex::Regex;
use std::path::{Path, PathBuf};

const MIN_WAIT_TIME: u64 = 6;
const CHARS_PER_SEC: u64 = 20;

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
    #[arg(short = 'D', default_value = "false")]
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

#[allow(dead_code)]
fn find_cookies_with_metadata(path: &Path, args: &Args) -> Result<Vec<CookieMetadata>> {
    let normal = args.all || !args.offensive;
    let offensive = args.all || args.offensive;
    let files = find_cookie_files(path, true, normal, offensive)?;
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

fn find_cookie_files(
    path: &Path,
    with_dat: bool,
    normal: bool,
    offensive: bool,
) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let pattern_all = format!("{}/**/*", path.to_string_lossy());
    let pattern_offensive_dir = format!("{}/**/off/*", path.to_string_lossy());
    let pattern_offensive_suffix = format!("{}/**/*-o", path.to_string_lossy());

    let files_not_offensive: Vec<PathBuf> = glob(&pattern_all)
        .expect("Failed to find cookies files")
        .filter_map(|entry| entry.ok())
        // exclude offensive folder
        .filter(|path| path.parent().unwrap().file_name().unwrap() != "off")
        // exclude BSD-style '-o' suffix offensives files
        .filter(|path| !path.ends_with("-o"))
        .collect();
    let files_offensive_dir: Vec<PathBuf> = glob(&pattern_offensive_dir)
        .expect("Failed to find cookies files")
        .filter_map(|entry| entry.ok())
        .collect();
    let files_offensive_suffix: Vec<PathBuf> = glob(&pattern_offensive_suffix)
        .expect("Failed to find cookies files")
        .filter_map(|entry| entry.ok())
        .collect();
    let files_offensive_self: Vec<PathBuf> = if path.is_dir() && path.file_name().unwrap() == "off"
    {
        // include the directory itself if it is 'off' for offensive
        glob(&pattern_all)
            .expect("Failed to find cookies files")
            .filter_map(|entry| entry.ok())
            .collect()
    } else {
        Vec::new()
    };

    // merge all candidates
    let mut cookie_files_candidates: Vec<PathBuf> = Vec::new();
    if normal {
        for file in files_not_offensive.iter() {
            if file.extension().unwrap_or_default() != "dat" {
                cookie_files_candidates.push(file.to_path_buf());
            }
        }
        cookie_files_candidates.extend(files_not_offensive);
    }
    if offensive {
        cookie_files_candidates.extend(files_offensive_dir);
        cookie_files_candidates.extend(files_offensive_suffix);
        cookie_files_candidates.extend(files_offensive_self);
    }
    cookie_files_candidates.sort();
    cookie_files_candidates.dedup();

    // filter out unwanted files
    let cookie_files: Vec<PathBuf> = cookie_files_candidates
        .iter()
        .map(|f| f.to_path_buf())
        // exclude directories
        .filter(|path| path.is_file())
        // exclude *.dat files
        .filter(|path| path.extension().unwrap_or_default() != "dat")
        // exclude files without *.dat if with_dat is true
        .filter(|path| !with_dat || path.with_extension("dat").exists())
        .collect();
    if cookie_files.is_empty() {
        anyhow::bail!("No fortune files found in directory: {}", path.display());
    }

    Ok(cookie_files)
}

fn find_cookies_with_text(path: &Path, args: &Args) -> Result<Vec<CookieMetadata>> {
    let normal = args.all || !args.offensive;
    let offensive = args.all || args.offensive;
    let files = find_cookie_files(path, true, normal, offensive)?;
    let mut cookies: Vec<CookieMetadata> = Vec::new();
    for file in files {
        let mut data = CookieMetadata::default();
        data.load_from_cookie_file(&file.to_string_lossy());
        // validate the data
        // comment out this block because original fortune does not check if data.quotes.is_empty()
        // if data.quotes.is_empty() {
        //     continue;
        // }
        cookies.push(data);
    }
    Ok(cookies)
}

// Quotes filtering mechanism
struct QuoteFilterManager {
    filters: Vec<Box<dyn Fn(&str) -> bool>>,
}

impl QuoteFilterManager {
    fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    fn filter(&self, quote: &str) -> bool {
        self.filters.iter().all(|f| f(quote))
    }

    fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
}

fn get_rel_path(path: &Path, base: &Path) -> PathBuf {
    path.strip_prefix(base)
        .unwrap_or_else(|_| path)
        .to_path_buf()
}

fn show_quote(path: &Path, quote: &Quote, show_file: bool, wait: bool) {
    if show_file {
        println!("({})\n%", path.to_string_lossy());
    }
    println!("{}", quote.content);
    if wait {
        let wait_time = std::cmp::max(
            (quote.content.len() as u64 + 1) / CHARS_PER_SEC,
            MIN_WAIT_TIME,
        );
        // debug_println!(args, "Wait time: {}s", wait_time);
        std::thread::sleep(std::time::Duration::from_secs(wait_time));
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.path);

    if !path.exists() {
        anyhow::bail!("{}: No such file or directory", path.display());
    }

    // Debug output if requested
    if args.debug {
        println!("Arguments: {:?}", std::env::args().collect::<Vec<_>>());
        println!("Path: {}", path.display());
    }

    // Create filters for quotes
    let mut filters = QuoteFilterManager::new();
    if args.short_only {
        filters.add_filter(move |q| {
            debug_println!(
                args,
                "q.len() + 1: {}, args.length: {}: {}",
                q.len() + 1,
                args.length,
                q
            );
            q.len() + 1 <= args.length
        }); // +1 for '\n'
    } else if args.long_only {
        filters.add_filter(move |q| {
            debug_println!(
                args,
                "q.len() + 1: {}, args.length: {}",
                q.len() + 1,
                args.length
            );
            q.len() + 1 > args.length
        }); // +1 for '\n'
    }
    if args.pattern.is_some() {
        let re = if args.ignore_case {
            Regex::new(&format!("(?i){}", args.pattern.as_ref().unwrap())).unwrap()
        } else {
            Regex::new(args.pattern.as_ref().unwrap()).unwrap()
        };
        filters.add_filter(move |q| re.is_match(q));
    }

    // Collect all fortune files
    // let mut cookies = if args.text || !filters.is_empty() {
    //     find_cookies_with_text(path, &args)?
    // } else {
    //     find_cookies_with_metadata(path, &args)?
    // };
    let mut cookies = find_cookies_with_text(path, &args)?;

    // Filter quotes based on given arguments (length, pattern, etc.)
    let pre_cookies_len = cookies.len();
    if !filters.is_empty() {
        for cookie in cookies.iter_mut() {
            let pre_quotes_len = cookie.quotes.len();
            cookie.quotes.retain(|q| filters.filter(&q.content));
            debug_println!(
                args,
                "> Filtered quotes [{}]: {} -> {}",
                cookie.path.to_string_lossy(),
                pre_quotes_len,
                cookie.quotes.len()
            );
        }
        cookies.retain(|c| !c.quotes.is_empty());
    }
    debug_println!(
        args,
        "Filtered cookies: {} -> {}",
        pre_cookies_len,
        cookies.len()
    );

    // -m pattern matching
    //  1. if -m is given, show all matching quotes
    //  2. output cookie file name in '\n%\n' delimiter format to stderr
    //  3. output the quote in '\n%\n' delimiter format to stdout
    if args.pattern.is_some() {
        let mut found = false;
        for cookie in cookies.iter() {
            let p = cookie.path.strip_prefix(&args.path).unwrap_or(&cookie.path);
            eprintln!("({})\n%", p.display());
            for quote in cookie.quotes.iter() {
                if filters.filter(&quote.content) {
                    println!("{}\n%", quote.content);
                    found = true;
                }
            }
        }
        //  The original implementation is exit(find_matches() != 0);
        //  So, if matches are found, exit with code 1
        if found {
            std::process::exit(1); // exit code 1
        } else {
            std::process::exit(0); // exit code 0
        }
    }

    // return exit code 1 if empty cookies
    if cookies.is_empty() {
        std::process::exit(1);
    }

    // -f: list files
    if args.list_files {
        eprintln!("{:5.2}% {}", 100.0, args.path);
        if !args.equal_size {
            // 100.00% tests/data
            //     45.45% apple
            //      9.09% one
            //     45.45% orange
            //      0.00% zero
            let total_quotes: usize = cookies.iter().map(|c| c.quotes.len()).sum();
            for cookie in cookies.iter() {
                let percentage = (cookie.quotes.len() as f64 / total_quotes as f64) * 100.0;
                let p = cookie.path.strip_prefix(&args.path).unwrap_or(&cookie.path);
                eprintln!("    {:5.2}% {}", percentage, p.display());
            }
        } else {
            // -e: equal size
            // 100.00% tests/data
            //     25.00% apple
            //     25.00% one
            //     25.00% orange
            //     25.00% zero
            let num_files = cookies.len();
            for cookie in cookies.iter() {
                let p = cookie.path.strip_prefix(&args.path).unwrap_or(&cookie.path);
                eprintln!("    {:5.2}% {}", 100.0 / num_files as f64, p.display());
            }
        }
        return Ok(());
    }

    if !args.equal_size {
        // normal: equal probability for each QUOTE.
        // given all quotes are equal chance to be chosen, which means larger cookie file has more chance to be chosen
        // so, aggregate all quotes and choose a random quote from the list
        let all_quotes: Vec<(&CookieMetadata, &Quote)> = cookies
            .iter()
            .flat_map(|c| c.quotes.iter().map(move |q| (c, q)))
            .collect();
        let cookies_len = cookies.len();
        debug_println!(
            args,
            "Found {} quotes in {} files",
            all_quotes.len(),
            cookies_len
        );
        let (cookie, quote) = all_quotes.choose(&mut rand::thread_rng()).unwrap();
        let path = get_rel_path(&cookie.path, args.path.as_ref());
        show_quote(&path, quote, args.show_file, args.wait);
    } else {
        // -e: equal probability for each FILE
        // since equal size, choose a random cookie file first, then choose a random quote from the file
        // let mut non_empty_cookies: Vec<&mut CookieMetadata> = cookies.iter_mut().filter(|c| !c.quotes.is_empty()).collect();
        // if non_empty_cookies.is_empty() {
        //     anyhow::bail!("{}: No fortune cookies found in the directory", path.display());
        // }
        if cookies.is_empty() {
            anyhow::bail!(
                "{}: No fortune cookies found in the directory",
                path.display()
            );
        }
        let cookie = cookies.choose_mut(&mut rand::thread_rng()).unwrap();
        debug_println!(args, "Chosen cookie: {}", cookie.path.display());
        if cookie.quotes.is_empty() {
            // original fortune just output nothing and exit
            return Ok(());
        }
        let quote = cookie.quotes.choose(&mut rand::thread_rng()).unwrap();
        let path = get_rel_path(&cookie.path, args.path.as_ref());
        show_quote(&path, quote, args.show_file, args.wait);
    }

    Ok(())
}
