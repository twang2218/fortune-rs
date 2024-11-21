pub mod metadata;

use anyhow::{Ok, Result};
use clap::Parser;
use env_logger::Env;
use glob::glob;
use log::debug;
use metadata::{CookieMetadata, Quote};
use rand::{
    distributions::WeightedIndex,
    prelude::Distribution,
    seq::{IteratorRandom, SliceRandom},
};
use regex::Regex;
use std::path::{Path, PathBuf};

const MIN_WAIT_TIME: u64 = 6;
const CHARS_PER_SEC: u64 = 20;

#[derive(Parser)]
#[command(
    version,
    about = "A Rust implementation of the classic fortune program"
)]
struct Args {
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

    /// [[n%] file/directory/all]
    paths: Vec<String>,
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

fn find_cookies(
    paths: Vec<WeightedPath>,
    normal: bool,
    offensive: bool,
    with_dat: bool,
) -> Result<Vec<WeightedPath>> {
    let mut cookies: Vec<WeightedPath> = Vec::new();
    for path in paths {
        let mut path = path.clone();
        let files = find_cookie_files(&path.path, with_dat, normal, offensive)?;
        for file in files {
            let mut data = CookieMetadata::default();
            data.load_from_cookie_file(&file.to_string_lossy())?;
            path.cookies.push(data);
        }
        cookies.push(path);
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

    fn len(&self) -> usize {
        self.filters.len()
    }
}

fn get_rel_path(path: &Path, base: &Path) -> PathBuf {
    let stripped = path
        .strip_prefix(base)
        .unwrap_or_else(|_| path)
        .to_path_buf();
    if stripped.file_name().is_none() {
        debug!(
            "get_rel_path(path: {:?}, base: {:?}) -> {:?}",
            path, base, path
        );
        path.to_path_buf()
    } else {
        debug!(
            "get_rel_path(path: {:?}, base: {:?}) -> {:?}",
            path, base, stripped
        );
        stripped
    }
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
        debug!("Wait time: {}s", wait_time);
        std::thread::sleep(std::time::Duration::from_secs(wait_time));
    }
}

#[derive(Debug, Clone)]
struct WeightedPath {
    path: PathBuf,
    weight: f64,
    cookies: Vec<CookieMetadata>,
}

impl WeightedPath {
    fn num_quotes(&self) -> usize {
        self.cookies.iter().map(|c| c.quotes.len()).sum()
    }
}

fn parse_weighted_paths(files: &Vec<String>) -> Result<Vec<WeightedPath>> {
    let mut weighted_paths: Vec<WeightedPath> = Vec::new();
    let mut prob: f64 = 0.0;
    for item in files {
        if item.ends_with("%") {
            // this is the probability for the next file
            prob = item.strip_suffix("%").unwrap().parse::<f64>()?;
            // debug!("parse_weighted_paths(): item: {} => prob: {}", item, prob);
        } else {
            // check if an probability is given
            if prob > 0.0 {
                weighted_paths.push(WeightedPath {
                    path: PathBuf::from(item.clone()),
                    weight: prob,
                    cookies: Vec::new(),
                });
                debug!("parse_weighted_paths(): item: {} => prob: {}", item, prob);
                prob = 0.0;
            } else {
                // no probability given, default to 0.0, which to be calculated later
                weighted_paths.push(WeightedPath {
                    path: PathBuf::from(item.clone()),
                    weight: 0.0,
                    cookies: Vec::new(),
                });
                debug!("parse_weighted_paths(): item: {} => prob: {}", item, 0);
            }
        }
    }
    // validate the probability
    let total_prob: f64 = weighted_paths.iter().map(|p| p.weight).sum();
    if total_prob == 100.0 || total_prob == 0.0 {
        // 100% or not given
    } else {
        // error should be raised
        anyhow::bail!("Error: total probability is not 100%: {}%", total_prob);
    }
    debug!("Weighted paths: {:?}", weighted_paths);
    Ok(weighted_paths)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let normal = args.all || !args.offensive;
    let offensive = args.all || args.offensive;
    let with_dat = true;

    // let path = Path::new(&args.path);
    let weighted_paths = parse_weighted_paths(&args.paths)?;

    let path_not_exists: Vec<PathBuf> = weighted_paths
        .iter()
        .map(|p| p.path.to_path_buf())
        .filter(|p| !p.exists())
        .collect();
    if !path_not_exists.is_empty() {
        anyhow::bail!("{:?} files not found.", path_not_exists);
    }

    // Debug output if requested
    if args.debug {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
        debug!("debug output enabled");
        debug!("args: {:?}", std::env::args().collect::<Vec<_>>());
    }

    // Create filters for quotes
    let mut filters = QuoteFilterManager::new();
    if args.short_only {
        filters.add_filter(move |q| {
            debug!(
                "q.len() + 1: {}, args.length: {}: {}",
                q.len() + 1,
                args.length,
                q
            );
            q.len() + 1 <= args.length
        }); // +1 for '\n'
    } else if args.long_only {
        filters.add_filter(move |q| {
            debug!("q.len() + 1: {}, args.length: {}", q.len() + 1, args.length);
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
    let mut weighted_paths = find_cookies(weighted_paths, normal, offensive, with_dat)?;

    // Filter quotes based on given arguments (length, pattern, etc.)
    if !filters.is_empty() {
        let pre_cookies_len: usize = weighted_paths
            .iter()
            .map(|p| p.cookies.len())
            .sum::<usize>();

        for p in weighted_paths.iter_mut() {
            for cookie in p.cookies.iter_mut() {
                let pre_quotes_len = cookie.quotes.len();
                cookie.quotes.retain(|q| filters.filter(&q.content));
                debug!(
                    "> filtered quotes [{}]: {} -> {}",
                    cookie.path.to_string_lossy(),
                    pre_quotes_len,
                    cookie.quotes.len()
                );
            }
            p.cookies.retain(|c| !c.quotes.is_empty());
        }
        let post_cookies_len: usize = weighted_paths
            .iter()
            .map(|p| p.cookies.len())
            .sum::<usize>();
        debug!(
            "Cookies: {} --filter[{}]--> {}",
            pre_cookies_len,
            filters.len(),
            post_cookies_len
        );
    }

    // -m pattern matching
    //  1. if -m is given, show all matching quotes
    //  2. output cookie file name in '\n%\n' delimiter format to stderr
    //  3. output the quote in '\n%\n' delimiter format to stdout
    if args.pattern.is_some() {
        let mut found = false;
        for p in weighted_paths.iter() {
            for cookie in p.cookies.iter() {
                let p = cookie.path.strip_prefix(&p.path).unwrap_or(&cookie.path);
                eprintln!("({})\n%", p.display());
                for quote in cookie.quotes.iter() {
                    if filters.filter(&quote.content) {
                        println!("{}\n%", quote.content);
                        found = true;
                    }
                }
            }
        }
        if found {
            return Ok(());
        } else {
            anyhow::bail!("");
        }
    }

    // return exit code 1 if empty cookies
    if weighted_paths.is_empty() {
        anyhow::bail!("Not found any fortune cookies");
    }

    // fill missing weight
    let total_weight: f64 = weighted_paths.iter().map(|p| p.weight).sum();
    if total_weight == 0.0 {
        // if all weights are 0, set equal weight according to the number of cookies or number of quotes depends on whether -e is given
        let total_num_cookies: usize = weighted_paths.iter().map(|p| p.cookies.len()).sum();
        let total_num_quotes: usize = weighted_paths.iter().map(|p| p.num_quotes()).sum();
        for p in weighted_paths.iter_mut() {
            if args.equal_size {
                p.weight = p.cookies.len() as f64 / total_num_cookies as f64 * 100.0;
            } else {
                p.weight = p.num_quotes() as f64 / total_num_quotes as f64 * 100.0;
            }
        }
    }

    // -f: list files
    if args.list_files {
        for path in weighted_paths.iter() {
            let mut weight: f64 = path.weight as f64;
            if weight == 0.0 {
                weight = 100.0 / weighted_paths.len() as f64;
            }
            eprintln!("{:5.2}% {}", weight, path.path.display());

            for cookie in path.cookies.iter() {
                let p = cookie.path.strip_prefix(&path.path).unwrap_or(&cookie.path);
                let prob = if args.equal_size {
                    weight / path.cookies.len() as f64
                } else {
                    weight * cookie.quotes.len() as f64 / path.num_quotes() as f64
                };
                eprintln!("    {:5.2}% {}", prob, p.display());
            }
        }

        return Ok(());
    }

    debug!("weighted Path:");
    for path in weighted_paths.iter() {
        debug!(
            "> (path: {:?}, weight: {:5.2}%, cookies: [{}])",
            path.path,
            path.weight,
            path.cookies
                .iter()
                .map(|p| p.path.to_string_lossy())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let choosen_index = WeightedIndex::new(
        weighted_paths
            .iter()
            .map(|p| p.weight as f64)
            .collect::<Vec<f64>>(),
    )
    .unwrap()
    .sample(&mut rand::thread_rng());
    let choosen_path = &weighted_paths[choosen_index];
    debug!("choosen path: {:?}", choosen_path.path);
    if args.equal_size {
        let choosen_cookie = choosen_path
            .cookies
            .choose(&mut rand::thread_rng())
            .unwrap();
        if choosen_cookie.quotes.is_empty() {
            // original fortune just output nothing and exit
            return Ok(());
        }
        debug!("choosen_cookie: {:?}", choosen_cookie.path);
        let choosen_quote = choosen_cookie
            .quotes
            .choose(&mut rand::thread_rng())
            .unwrap();
        let path = get_rel_path(&choosen_cookie.path, &choosen_path.path);
        show_quote(&path, choosen_quote, args.show_file, args.wait);
    } else {
        let (path, choosen_quote) = choosen_path
            .cookies
            .iter()
            .flat_map(|c| c.quotes.iter().map(|q| (c.path.clone(), q)))
            .choose(&mut rand::thread_rng())
            .unwrap();
        let path = get_rel_path(&path, &choosen_path.path);
        show_quote(&path, choosen_quote, args.show_file, args.wait);
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parse_weighted_paths() {
        let testcases = [
            (
                "data1 data2 data3",
                vec![("data1", 0.0), ("data2", 0.0), ("data3", 0.0)],
                false,
            ),
            (
                "50% data1 50% data2",
                vec![("data1", 50.0), ("data2", 50.0)],
                false,
            ),
            ("50% data1 50% data2 10% data3", vec![], true),
        ];

        for (line, expected, is_err) in testcases.iter() {
            let files = line.split_whitespace().map(|s| s.to_string()).collect();
            let weighted_paths = super::parse_weighted_paths(&files);
            if *is_err {
                assert!(weighted_paths.is_err());
            } else {
                let weighted_paths = weighted_paths.unwrap();
                assert_eq!(weighted_paths.len(), expected.len());
                for (i, (filename, weight)) in expected.iter().enumerate() {
                    assert_eq!(weighted_paths[i].path.to_string_lossy(), *filename);
                    assert_eq!(weighted_paths[i].weight, *weight);
                }
            }
        }
    }

    #[test]
    fn test_get_rel_path() {
        let testcases = [
            ("/a/b/c", "/a/b/c", "/a/b/c"),
            ("/a/b/c", "/a/b", "c"),
            ("/a/b/c", "/a", "b/c"),
            ("/a/b/c", "/a/b/c/d", "/a/b/c"),
            ("/a/b/c", "/a/b/c/d/e", "/a/b/c"),
            ("aa/bb/cc", "aa/bb", "cc"),
            ("aa/bb/cc", "aa", "bb/cc"),
        ];

        for (path, base, expected) in testcases.iter() {
            let path = std::path::Path::new(path);
            let base = std::path::Path::new(base);
            let expected = std::path::Path::new(expected);
            let rel_path = super::get_rel_path(path, base);
            assert_eq!(
                rel_path,
                expected,
                "path: {}, base: {} -> expected: {}, got: {}",
                path.display(),
                base.display(),
                expected.display(),
                rel_path.display()
            );
        }
    }

    #[test]
    fn test_quote_filter_manager() {
        let testcases = [
            ("1234", false),
            ("12345", false),
            ("123456", true),
            ("1234567", true),
            ("12345678", true),
            ("123456789", true),
            ("1234567890", false),
        ];

        for (quote, expected) in testcases.iter() {
            let mut filters = super::QuoteFilterManager::new();
            assert_eq!(filters.is_empty(), true, "expected: empty, got: not empty");
            filters.add_filter(|q| q.len() < 10);
            assert_eq!(filters.is_empty(), false, "expected: not empty, got: empty");
            filters.add_filter(|q| q.len() > 5);
            let result = filters.filter(quote);
            assert_eq!(
                result, *expected,
                "quote: {}, expected: {}, got: {}",
                quote, expected, result
            );
        }
    }

    #[test]
    fn test_find_cookie_files() {
        let path = std::path::Path::new("tests/data");

        let testcases = [
            (path, true, true, false, 4),
            (path, true, false, true, 1),
            (path, true, true, true, 5),
            (path, false, true, false, 4),
            (path, false, false, true, 1),
            (path, false, true, true, 5),
        ];

        for (path, with_dat, normal, offensive, expected) in testcases.iter() {
            let files = super::find_cookie_files(path, *with_dat, *normal, *offensive);
            assert!(
                files.is_ok(),
                "path: {} (with_dat: {}, normal: {}, offensive: {}) => got error: {:?}",
                path.display(),
                with_dat,
                normal,
                offensive,
                files.err()
            );
            let files = files.unwrap();
            assert_eq!(
                files.len(),
                *expected,
                "path: {} (with_dat: {}, normal: {}, offensive: {}) => expected: {}, got: {}",
                path.display(),
                with_dat,
                normal,
                offensive,
                expected,
                files.len()
            );
        }
    }
}
