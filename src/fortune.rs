pub mod cookie;

use argh::FromArgs;
use cookie::{
    embed::{Embedded, EMBED_PREFIX},
    Cookie, CookieCabinet, CookieSieve,
};
use env_logger::Env;
use log::debug;
use regex_lite::Regex;

const MIN_WAIT_TIME: u64 = 6;
const CHARS_PER_SEC: u64 = 20;

#[derive(FromArgs)]
/// A Rust implementation of the classic fortune program
struct Args {
    /// choose from all lists of maxims, both offensive and not
    #[argh(switch, short = 'a')]
    all: bool,

    /// show the cookie file from which the fortune came
    #[argh(switch, short = 'c')]
    show_file: bool,

    /// enable additional debugging output
    #[argh(switch, short = 'D')]
    debug: bool,

    /// consider all fortune files to be of equal size
    #[argh(switch, short = 'e')]
    equal_size: bool,

    /// print out the list of files which would be searched
    #[argh(switch, short = 'f')]
    list_files: bool,

    /// ignore case for -m patterns
    #[argh(switch, short = 'i')]
    ignore_case: bool,

    /// long dictums only
    #[argh(switch, short = 'l')]
    long_only: bool,

    /// print out all fortunes which match the pattern
    #[argh(option, short = 'm')]
    pattern: Option<String>,

    /// set the longest fortune length considered to be "short"
    #[argh(option, short = 'n', default = "160")]
    length: usize,

    /// short apothegms only
    #[argh(switch, short = 's')]
    short_only: bool,

    /// choose only from potentially offensive aphorisms
    #[argh(switch, short = 'o')]
    offensive: bool,

    /// don't translate UTF-8 fortunes to the locale
    #[argh(switch, short = 'u')]
    no_utf8_translate: bool,

    /// wait before termination based on message length
    #[argh(switch, short = 'w')]
    wait: bool,

    /// [[n%] file/directory/all]
    #[argh(positional)]
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

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    // Debug output if requested
    if args.debug {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
        debug!("debug output enabled");
        debug!("args: {:?}", std::env::args().collect::<Vec<_>>());
    }

    if args.no_utf8_translate {
        anyhow::bail!("-u is not supported yet.");
    }

    let normal = args.all || !args.offensive;
    let offensive = args.all || args.offensive;

    let mut cabinet = CookieCabinet::from_string_list(&args.paths)?;

    for shelf in cabinet.shelves.iter_mut() {
        //  if the shelf location is not point to embedded data and not exists, then check if it exists in embedded data
        if !shelf.location.starts_with(EMBED_PREFIX) && !std::fs::exists(&shelf.location)? {
            if Embedded::exists(&shelf.location) {
                // update shelf location if necessary
                shelf.location = Embedded::format_path(&shelf.location);
            } else {
                anyhow::bail!("{} not found.", shelf.location);
            }
        }
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
