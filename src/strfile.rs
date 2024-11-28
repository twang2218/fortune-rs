pub mod cookie;

use anyhow::Result;
use clap::{Arg, Command};
use cookie::serializer::Serializer;
use cookie::CookieJar;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::Write;

/// Configuration options parsed from command line arguments.
#[derive(Default)]
struct Args {
    infile: String,   // Input file path
    outfile: String,  // Output file path
    delimch: char,    // Delimiter character
    sflag: bool,      // Silent mode
    oflag: bool,      // Order strings
    iflag: bool,      // Ignore case when ordering
    rflag: bool,      // Randomize strings
    xflag: bool,      // Set rotated flag
    lflag: bool,      // Load and display cookie file
    platform: String, // Platform to use for serialization, one of: homebrew, linux, freebsd
}

/// Parses command line arguments and returns a Config struct.
/// Uses clap for argument parsing with support for various options
/// that control the processing of fortune cookie files.
fn getargs() -> Args {
    let matches = Command::new("strfile")
        .arg(
            Arg::new("infile")
                .required(true)
                .help("Input file containing strings separated by delimiter"),
        )
        .arg(
            Arg::new("outfile")
                .required(false)
                .help("Output data file (default: infile.dat)"),
        )
        .arg(
            Arg::new("delimch")
                .short('c')
                .value_parser(clap::value_parser!(String))
                .help("Change delimiting character from '%' to specified character"),
        )
        .arg(
            Arg::new("sflag")
                .short('s')
                .action(clap::ArgAction::SetTrue)
                .help("Silent mode - do not show summary of data processed"),
        )
        .arg(
            Arg::new("oflag")
                .short('o')
                .action(clap::ArgAction::SetTrue)
                .help("Order the strings in alphabetical order"),
        )
        .arg(
            Arg::new("iflag")
                .short('i')
                .action(clap::ArgAction::SetTrue)
                .help("Ignore case when ordering strings"),
        )
        .arg(
            Arg::new("rflag")
                .short('r')
                .action(clap::ArgAction::SetTrue)
                .help("Randomize the order of the strings"),
        )
        .arg(
            Arg::new("xflag")
                .short('x')
                .action(clap::ArgAction::SetTrue)
                .help("Set the rotated bit"),
        )
        .arg(
            Arg::new("lflag")
                .short('l')
                .action(clap::ArgAction::SetTrue)
                .help("Load a data file and display its contents"),
        )
        .arg(
            Arg::new("platform")
                .long("platform")
                .value_parser(clap::value_parser!(String))
                .help("Platform to use for serialization: homebrew, linux, freebsd"),
        )
        .get_matches();

    let infile = matches.get_one::<String>("infile").unwrap();
    Args {
        infile: infile.trim_end_matches(".dat").to_string(),
        outfile: matches
            .get_one::<String>("outfile")
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}.dat", infile.trim_end_matches(".dat"))),
        delimch: matches
            .get_one::<String>("delimch")
            .map(|s| s.chars().next().unwrap())
            .unwrap_or('%'),
        sflag: matches.get_flag("sflag"),
        oflag: matches.get_flag("oflag"),
        iflag: matches.get_flag("iflag"),
        rflag: matches.get_flag("rflag"),
        xflag: matches.get_flag("xflag"),
        lflag: matches.get_flag("lflag"),
        platform: matches
            .get_one::<String>("platform")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "".to_string()),
    }
}

/// Main function that processes fortune cookie files.
/// Handles command line arguments and orchestrates the file processing.
fn main() -> Result<()> {
    // Parse command-line arguments
    let cfg = getargs();

    // If -l flag is set, load and display data file
    if cfg.lflag {
        let data = CookieJar::from_dat(&cfg.outfile)?;
        println!("File: {}", cfg.outfile);
        println!("{}", data);
        return Ok(());
    }

    // Parse input cookie file
    let mut jar = CookieJar::from_text_file(&cfg.infile, cfg.delimch)?;

    // Apply ordering if -o flag is set
    if cfg.oflag {
        jar.cookies.sort_by(|a, b| {
            if cfg.iflag {
                a.content.to_lowercase().cmp(&b.content.to_lowercase())
            } else {
                a.content.cmp(&b.content)
            }
        });
        jar.flags |= cookie::FLAGS_ORDERED;
    }

    // Randomize if -r flag is set
    if cfg.rflag {
        jar.cookies.shuffle(&mut thread_rng());
        jar.flags |= cookie::FLAGS_RANDOMIZED;
    }

    // Set rotated flag if -x flag is set
    if cfg.xflag {
        jar.flags |= cookie::FLAGS_ROTATED;
    }

    // Write output data file
    let bytes = Serializer::to_bytes(
        &jar,
        &Serializer::get_type_by_platform(&cfg.platform.as_str()),
    );
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&cfg.outfile)
        .expect(format!("Error opening output file: {}", cfg.outfile).as_str());
    f.write_all(&bytes).unwrap();

    // Display summary unless -s flag is set
    if !cfg.sflag {
        println!("'{}' created", cfg.outfile);
        if jar.cookies.len() == 1 {
            println!("There was 1 string");
        } else {
            println!("There were {} strings", jar.cookies.len());
        }
        println!(
            "Longest string: {} byte{}",
            jar.max_length,
            if jar.max_length == 1 { "" } else { "s" }
        );
        println!(
            "Shortest string: {} byte{}",
            jar.min_length,
            if jar.min_length == 1 { "" } else { "s" }
        );
    }

    Ok(())
}
