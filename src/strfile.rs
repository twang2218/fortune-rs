pub mod cookie;

use anyhow::Result;
use argh::FromArgs;
use cookie::serializer::Serializer;
use cookie::CookieJar;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::Write;

#[derive(FromArgs)]
/// Create a data file for the fortune program.
struct Args {
    /// input file containing strings separated by delimiter
    #[argh(positional)]
    infile: String,

    /// output data file (default: infile.dat)
    #[argh(positional)]
    outfile: Option<String>,

    /// change delimiting character from '%' to specified character
    #[argh(option, short = 'c')]
    delimch: Option<char>,

    /// silent mode - do not show summary of data processed
    #[argh(switch, short = 's')]
    sflag: bool,

    /// order the strings in alphabetical order
    #[argh(switch, short = 'o')]
    oflag: bool,

    /// ignore case when ordering strings
    #[argh(switch, short = 'i')]
    iflag: bool,

    /// randomize the order of the strings
    #[argh(switch, short = 'r')]
    rflag: bool,

    /// set the rotated bit
    #[argh(switch, short = 'x')]
    xflag: bool,

    /// load a data file and display its contents
    #[argh(switch, short = 'l')]
    lflag: bool,

    /// platform to use for serialization: homebrew, linux, freebsd
    #[argh(option)]
    platform: Option<String>,
}

/// Main function that processes fortune cookie files.
/// Handles command line arguments and orchestrates the file processing.
fn main() -> Result<()> {
    // Parse command-line arguments
    let args = argh::from_env::<Args>();
    let infile = args.infile.trim_end_matches(".dat").to_string();
    let outfile = args
        .outfile
        .unwrap_or_else(|| format!("{}.dat", args.infile.trim_end_matches(".dat")));
    let delimch = args.delimch.unwrap_or('%');
    let platform = args.platform.unwrap_or_else(|| "".to_string());

    // If -l flag is set, load and display data file
    if args.lflag {
        let data = CookieJar::from_dat(&outfile)?;
        println!("File: {}", outfile);
        println!("{}", data);
        return Ok(());
    }

    // Parse input cookie file
    let mut jar = CookieJar::from_text_file(&infile, delimch)?;

    // Apply ordering if -o flag is set
    if args.oflag {
        jar.cookies.sort_by(|a, b| {
            if args.iflag {
                a.content.to_lowercase().cmp(&b.content.to_lowercase())
            } else {
                a.content.cmp(&b.content)
            }
        });
        jar.flags |= cookie::FLAGS_ORDERED;
    }

    // Randomize if -r flag is set
    if args.rflag {
        jar.cookies.shuffle(&mut thread_rng());
        jar.flags |= cookie::FLAGS_RANDOMIZED;
    }

    // Set rotated flag if -x flag is set
    if args.xflag {
        jar.flags |= cookie::FLAGS_ROTATED;
    }

    // Write output data file
    let bytes = Serializer::to_bytes(&jar, &Serializer::get_type_by_platform(&platform));
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&outfile)
        .expect(format!("Error opening output file: {}", outfile).as_str());
    f.write_all(&bytes).unwrap();

    // Display summary unless -s flag is set
    if !args.sflag {
        println!("'{}' created", outfile);
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
