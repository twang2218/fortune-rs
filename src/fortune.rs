pub mod cookie;

use anyhow::{Ok, Result};
use clap::Parser;
use cookie::{Cookie, CookieCabinet, CookieSieve};
use env_logger::Env;
use log::debug;
use regex::Regex;
use std::{path::PathBuf, str::FromStr};

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

fn show_cookie(cookie: &Cookie, show_file: bool) {
    if show_file {
        println!("({})\n%", cookie.location);
    }
    println!("{}", cookie.content);
}

fn generate_filters(args: &Args) -> CookieSieve {
    let mut filters = CookieSieve::default();
    let length = args.length;
    if args.short_only {
        filters.add_filter(move |q| q.len() + 1 <= length); // +1 for '\n'
    } else if args.long_only {
        filters.add_filter(move |q| q.len() + 1 > length); // +1 for '\n'
    }
    if args.pattern.is_some() {
        let re = if args.ignore_case {
            Regex::new(&format!("(?i){}", args.pattern.as_ref().unwrap())).unwrap()
        } else {
            Regex::new(args.pattern.as_ref().unwrap()).unwrap()
        };
        filters.add_filter(move |q| re.is_match(q));
    }
    filters
}

fn main() -> Result<()> {
    let args = Args::parse();

    let normal = args.all || !args.offensive;
    let offensive = args.all || args.offensive;
    // let with_dat = true;

    let mut cabinet = CookieCabinet::from_string_list(&args.paths)?;
    // let path = Path::new(&args.path);
    // let weighted_paths = parse_weighted_paths(&args.paths)?;

    for shelf in cabinet.iter() {
        let p = PathBuf::from_str(shelf.location.as_str())?;
        if !p.exists() {
            // TODO: search embedded resources
            anyhow::bail!("{} not found.", p.display());
        }
    }

    // Debug output if requested
    if args.debug {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
        debug!("debug output enabled");
        debug!("args: {:?}", std::env::args().collect::<Vec<_>>());
    }

    // Create filters based on command-line arguments
    let filters = generate_filters(&args);

    // Collect all fortune files
    cabinet.load(normal, offensive)?;

    // Filter cookies based on given arguments (length, pattern, etc.)
    if filters.len() > 0 {
        cabinet.filter(&filters)?;
    }

    // -m pattern matching
    //  1. if -m is given, show all matching cookies
    //  2. output cookie file name in '\n%\n' delimiter format to stderr
    //  3. output the cookie in '\n%\n' delimiter format to stdout
    if args.pattern.is_some() {
        let mut found = false;
        for shelf in cabinet.iter() {
            for jar in shelf.jars.iter() {
                let cookies: Vec<&Cookie> = jar
                    .cookies
                    .iter()
                    .filter(|cookie| filters.filter(&cookie.content))
                    .collect();
                if !cookies.is_empty() {
                    found = true;
                    eprintln!("({})\n%", jar.location);
                    for cookie in cookies.iter() {
                        println!("{}\n%", cookie.content);
                    }
                }
            }
        }

        if found {
            return Ok(());
        } else {
            anyhow::bail!(
                "No matching fortune cookies for pattern: {}",
                args.pattern.unwrap()
            );
        }
    }

    // return exit code 1 if empty cookies
    if cabinet.num_of_jars() == 0 {
        anyhow::bail!("Not found any fortune cookies");
    }

    cabinet.calculate_prob(args.equal_size);

    // -f: list files
    if args.list_files {
        for shelf in cabinet.iter() {
            eprintln!("{:5.2}% {}", shelf.probability, shelf.location);
            for jar in shelf.jars.iter() {
                eprintln!("    {:5.2}% {}", jar.probability, jar.location);
            }
        }
        return Ok(());
    }

    let cookie: &Cookie = cabinet.choose(&mut rand::thread_rng()).unwrap();
    show_cookie(cookie, args.show_file);
    if args.wait {
        let wait_time = std::cmp::max(
            (cookie.content.len() as u64 + 1) / CHARS_PER_SEC,
            MIN_WAIT_TIME,
        );
        debug!("Wait time: {}s", wait_time);
        std::thread::sleep(std::time::Duration::from_secs(wait_time));
    }

    Ok(())
}
