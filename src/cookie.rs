pub mod embed;
pub mod serializer;

use std::path::PathBuf;

use anyhow::Result;
use embed::{Embedded, EMBED_PREFIX};
use glob::glob;
use log::debug;
use oxilangtag::LanguageTag;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::seq::SliceRandom;
use serializer::Serializer;
use sys_locale::get_locale;

/// Constants defining the data file format and flags
pub const FLAGS_RANDOMIZED: u64 = 0x0001; /* randomized pointers */
pub const FLAGS_ORDERED: u64 = 0x0002; /* ordered pointers */
pub const FLAGS_ROTATED: u64 = 0x0004; /* rot-13'd pointers */

pub const DEFAULT_DELIMITER: char = '%';

/// Represents a single fortune cookie with its text.
#[derive(Debug, Clone)]
pub struct Cookie {
    pub location: String, // Path to the source file
    pub content: String,  // The actual cookie text
    pub offset: u64,      // Offset of the cookie in the source file
}

/// Represents the header structure of a fortune cookie data file.
/// This header contains metadata about the fortune cookie strings.
#[derive(Debug, Clone)]
pub struct CookieJar {
    pub location: String, // Path to the source file (relative to the shelf's location)
    pub probability: f64, // Probability of selecting this jar
    pub platform: String, // Platform to use for serialization, one of: homebrew, linux, freebsd
    pub version: u64,     // Data file format version
    pub max_length: u64,  // Length of longest string
    pub min_length: u64,  // Length of shortest string
    pub flags: u64,       // File flags (random, ordered, rotated)
    pub delim: char,      // Delimiting character
    pub file_size: u64,   // Total size of source file
    pub cookies: Vec<Cookie>, // Offsets of each string in the file
}

impl Default for CookieJar {
    /// Creates a new CookieJar with default values.
    /// Sets delimiter to '%'.
    fn default() -> Self {
        Self {
            location: "".to_string(),
            probability: 0.0,
            platform: "".to_string(),
            version: 0,
            max_length: 0,
            min_length: u64::MAX,
            flags: 0,
            delim: '%',
            file_size: 0,
            cookies: Vec::new(),
        }
    }
}

impl std::fmt::Display for CookieJar {
    /// Formats the CookieJar struct for display.
    /// Shows the header fields and cookie offsets.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CookieJar {{\n")?;
        write!(f, "  location: '{}'\n", self.location)?;
        write!(f, "  probability: {}\n", self.probability)?;
        write!(f, "  platform: '{}'\n", self.platform)?;
        write!(f, "  version: {}\n", self.version)?;
        write!(f, "  num_cookies: {}\n", self.cookies.len())?;
        write!(f, "  max_length: {}\n", self.max_length)?;
        write!(f, "  min_length: {}\n", self.min_length)?;

        let mut flags: Vec<&str> = Vec::new();
        if self.flags & FLAGS_RANDOMIZED != 0 {
            flags.push("RANDOM");
        }
        if self.flags & FLAGS_ORDERED != 0 {
            flags.push("ORDERED");
        }
        if self.flags & FLAGS_ROTATED != 0 {
            flags.push("ROTATED");
        }
        write!(f, "  flags: [{}]\n", flags.join(", "))?;

        write!(f, "  delim: '{}'\n", self.delim)?;
        write!(f, "  file_size: {}\n", self.file_size)?;

        // write!(
        //     f,
        //     "  offsets: [{}]\n",
        //     self.cookies
        //         .iter()
        //         .map(|cookie| format!("{}", cookie.offset))
        //         .collect::<Vec<String>>()
        //         .join(", ")
        // )?;
        write!(f, "}}")
    }
}

impl CookieJar {
    pub fn iter(&self) -> std::slice::Iter<Cookie> {
        self.cookies.iter()
    }

    pub fn num_of_cookies(&self) -> usize {
        self.cookies.len()
    }

    pub fn from_dat(filename: &str) -> Result<CookieJar> {
        if !filename.ends_with(".dat") {
            anyhow::bail!("Error: Invalid data file: {}", filename);
        }
        let bytes = std::fs::read(filename)
            .expect(format!("Error reading cookie database: {}", filename).as_str());

        let t = Serializer::get_type_by_bytes(&bytes);
        let mut data = Serializer::from_bytes(&bytes, &t);
        data.location = filename.trim_end_matches(".dat").to_string(); // Remove .dat extension
        Ok(data)
    }

    pub fn from_text(content: &str, location: &str, delim: char) -> Result<CookieJar> {
        debug!("from_text(): content: '{:?}'", content);
        // normalize newline characters
        let content = content.replace("\r\n", "\n").replace("\r", "\n");
        let mut jar = CookieJar::default();
        jar.platform = Serializer::get_current_platform();
        // use the filename without .dat extension
        jar.location = location.trim_end_matches(".dat").to_string();
        jar.delim = delim;
        // Split content by delimiter pattern
        let splitter = format!("\n{}\n", delim);
        let parts: Vec<&str> = content
            .split(splitter.as_str())
            .map(|s| s.trim_end_matches(format!("\n{}", delim).as_str())) // remove the '\n%' for the last cookie
            .collect();
        jar.cookies = parts
            .iter()
            .filter(|part| !part.trim().is_empty())
            .map(|part| Cookie {
                location: jar.location.clone(),
                content: part.to_string(),
                offset: 0, // TODO: offset is not used for text files
            })
            .collect();
        let lengths: Vec<u64> = jar
            .cookies
            .iter()
            .map(|c| c.content.len() as u64 + 1)
            .collect();
        jar.max_length = *lengths.iter().max().unwrap_or(&0);
        jar.min_length = *lengths.iter().min().unwrap_or(&0);
        jar.file_size = content.len() as u64;

        debug!("from_text(): -> (path: {:?}, platform: {:?}, max_length: {}, min_length: {}, num_cookies: {})",
            jar.location, jar.platform, jar.max_length, jar.min_length, jar.cookies.len());

        Ok(jar)
    }

    pub fn from_text_file(filename: &str, delim: char) -> Result<CookieJar> {
        let content = std::fs::read_to_string(filename)?;
        CookieJar::from_text(&content, filename, delim)
    }

    pub fn filter(&mut self, filter: &CookieSieve) -> Result<()> {
        let before_filter_len = self.cookies.len();
        self.cookies.retain(|c| filter.filter(&c.content));
        let after_filter_len = self.cookies.len();
        debug!(
            "CookieJar::filter(): [{}] filtered cookies: {} => {}",
            self.location, before_filter_len, after_filter_len
        );
        Ok(())
    }

    pub fn choose(&self, rng: &mut rand::rngs::ThreadRng) -> Option<&Cookie> {
        self.cookies.choose(rng)
    }

    pub fn update_location(&mut self, parent_location: &str) {
        self.location = trim_parent_path(&self.location, parent_location);
        for c in &mut self.cookies {
            c.location = self.location.clone();
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CookieShelf {
    pub location: String,
    pub probability: f64,
    pub jars: Vec<CookieJar>,
}

#[allow(dead_code)]
impl CookieShelf {
    pub fn new(location: &str, probability: f64) -> Self {
        Self {
            location: location.to_string(),
            probability,
            jars: Vec::new(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<CookieJar> {
        self.jars.iter()
    }

    pub fn num_of_cookies(&self) -> usize {
        self.jars.iter().map(|c| c.cookies.len()).sum()
    }

    pub fn num_of_jars(&self) -> usize {
        self.jars.len()
    }

    pub fn calculate_prob(&mut self, equal_size: bool) {
        // calculate probability for each jar
        if self.probability == 0.0 {
            return;
        }
        if equal_size {
            // if equal_size is given, set equal probability to each jar
            let prob = self.probability / self.num_of_jars() as f64;
            for jar in &mut self.jars {
                jar.probability = prob;
            }
        } else {
            // if equal_size is not given, set probability to each jar based on the number of cookies
            let total_num_cookies: usize = self.num_of_cookies();
            for jar in &mut self.jars {
                jar.probability =
                    jar.cookies.len() as f64 / total_num_cookies as f64 * self.probability;
            }
        }
    }

    pub fn load(&mut self, normal: bool, offensive: bool) -> Result<()> {
        // find jars
        let mut jars: Vec<CookieJar> = Vec::new();
        if self.location.starts_with(EMBED_PREFIX) {
            debug!("Loading embedded cookies from: '{}'", self.location);

            let paths = Embedded::find(&self.location)?;
            for path in paths {
                let content = Embedded::read_to_string(&path)?;
                let jar = CookieJar::from_text(&content, &path, DEFAULT_DELIMITER)?;
                jars.push(jar);
            }
        } else {
            let p = PathBuf::from(&self.location);
            if p.is_file() {
                jars.push(CookieJar::from_text_file(
                    &self.location,
                    DEFAULT_DELIMITER,
                )?);
            } else {
                let pattern_off_dir = glob::Pattern::new(&format!("{}/**/off/*", &self.location))?;
                let pattern_off_file = glob::Pattern::new(&format!("{}/**/*-o", &self.location))?;

                let pattern = format!("{}/**/*", &self.location);
                let files: Vec<String> = glob(&pattern)
                    .expect(&format!("Failed to read glob pattern {}", pattern))
                    .filter_map(Result::ok)
                    // only keep files
                    .filter(|p| p.is_file())
                    // filter out .dat files
                    .filter(|p| p.extension().unwrap_or_default() != "dat")
                    // filter out dot files
                    .filter(|p| !p.file_name().unwrap().to_str().unwrap().starts_with("."))
                    // filter by normal/offensive
                    .filter(|p| {
                        if normal && offensive {
                            return true;
                        }
                        if normal {
                            return !pattern_off_dir.matches_path(p)
                                && !pattern_off_file.matches_path(p);
                        }
                        if offensive {
                            return pattern_off_dir.matches_path(p)
                                || pattern_off_file.matches_path(p);
                        }
                        false
                    })
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();

                jars = files
                    .iter()
                    .map(|f| {
                        let mut jar = CookieJar::from_text_file(f, DEFAULT_DELIMITER)
                            .expect(&format!("Failed to read cookie file: {}", f));
                        jar.update_location(&self.location);
                        jar
                    })
                    .collect();
            }
        }
        self.jars = jars;
        Ok(())
    }

    pub fn filter(&mut self, filter: &CookieSieve) -> Result<()> {
        for jar in &mut self.jars {
            jar.filter(filter)?;
        }
        self.jars.retain(|j| j.num_of_cookies() > 0);
        Ok(())
    }

    pub fn choose(&self, rng: &mut rand::rngs::ThreadRng) -> Option<&Cookie> {
        let index = WeightedIndex::new(
            self.jars
                .iter()
                .map(|j| j.probability)
                .collect::<Vec<f64>>(),
        )
        .unwrap()
        .sample(rng);
        let jar = &self.jars[index];
        jar.choose(rng)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CookieCabinet {
    pub shelves: Vec<CookieShelf>,
}

#[allow(dead_code)]
impl CookieCabinet {
    pub fn new(shelves: Vec<CookieShelf>) -> Self {
        Self { shelves }
    }

    pub fn push(&mut self, shelf: CookieShelf) {
        self.shelves.push(shelf);
    }

    pub fn iter(&self) -> std::slice::Iter<CookieShelf> {
        self.shelves.iter()
    }

    pub fn num_of_cookies(&self) -> usize {
        self.shelves.iter().map(|s| s.num_of_cookies()).sum()
    }

    pub fn num_of_jars(&self) -> usize {
        self.shelves.iter().map(|s| s.num_of_jars()).sum()
    }

    pub fn calculate_prob(&mut self, equal_size: bool) {
        // caclulate probability for each shelf
        let total_prob: f64 = self.shelves.iter().map(|s| s.probability).sum();
        if total_prob == 0.0 {
            // no probability is given
            if equal_size {
                // if equal_size is given, set same probability to each jar
                let prob_per_jar = 100.0 / self.num_of_jars() as f64;
                for shelf in &mut self.shelves {
                    shelf.probability = prob_per_jar * shelf.num_of_jars() as f64;
                }
            } else {
                // if equal_size is not given, set probability to each jar based on the number of cookies
                let total_num_cookies: usize = self.num_of_cookies();
                let prob_per_cookie = 100.0 / total_num_cookies as f64;
                for shelf in &mut self.shelves {
                    shelf.probability = shelf.num_of_cookies() as f64 * prob_per_cookie;
                }
            }
        }

        // call shelf.calculate_prob() to calculate probability for each jar
        for shelf in &mut self.shelves {
            shelf.calculate_prob(equal_size);
        }
    }

    pub fn load(&mut self, normal: bool, offensive: bool) -> Result<()> {
        for shelf in &mut self.shelves {
            shelf.load(normal, offensive)?;
        }
        Ok(())
    }

    pub fn from_string_list(items: &[String]) -> Result<CookieCabinet> {
        let mut shelves: CookieCabinet = CookieCabinet::default();
        if items.is_empty() {
            //  use embedded cookies if no shelves are given
            // get current language
            let lang = if let Some(locale) = get_locale() {
                let tag = LanguageTag::parse(locale.as_str())?;
                debug!(
                    "System locale: {}, primary_language: {}, region: {:?}",
                    locale,
                    tag.primary_language(),
                    tag.region(),
                );
                tag.primary_language().to_string()
            } else {
                "en".to_string()
            };
            // check if the language is supported
            let location = if Embedded::exists(&lang) {
                format!("{}{}", EMBED_PREFIX, lang)
            } else {
                format!("{}{}", EMBED_PREFIX,"en")
            };
            shelves.push(CookieShelf::new(&location, 100.0));
        } else {
            let mut prob: f64 = 0.0;
            for item in items {
                if item.ends_with("%") {
                    // this is the probability for the next shelf
                    prob = item.strip_suffix("%").unwrap().parse::<f64>().unwrap();
                } else {
                    // check if an probability is given
                    if prob > 0.0 {
                        shelves.push(CookieShelf::new(item, prob));
                        prob = 0.0;
                    } else {
                        // no probability given, default to 0.0, which to be calculated later
                        shelves.push(CookieShelf::new(item, 0.0));
                    }
                }
            }
        }
        // check if only partial probabilities are given
        let total_prob: f64 = shelves.iter().map(|s| s.probability).sum();
        if (total_prob - 100.0).abs() > 0.0001 && total_prob > 0.0 {
            // partial probabilities are given
            anyhow::bail!(
                "Error: Partial probabilities are given. Total probability: {}",
                total_prob
            );
        }
        Ok(shelves)
    }

    pub fn filter(&mut self, filter: &CookieSieve) -> Result<()> {
        for shelf in &mut self.shelves {
            shelf.filter(filter)?;
        }
        self.shelves.retain(|s| s.num_of_jars() > 0);
        Ok(())
    }

    pub fn choose(&self, rng: &mut rand::rngs::ThreadRng) -> Option<&Cookie> {
        let index = WeightedIndex::new(
            self.shelves
                .iter()
                .map(|s| s.probability)
                .collect::<Vec<f64>>(),
        )
        .unwrap()
        .sample(rng);
        let shelf = &self.shelves[index];
        shelf.choose(rng)
    }
}

// Cookie filtering mechanism
#[allow(dead_code)]
#[derive(Default)]
pub struct CookieSieve {
    filters: Vec<Box<dyn Fn(&str) -> bool>>,
}

impl CookieSieve {
    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn filter(&self, cookie: &str) -> bool {
        self.filters.iter().all(|f| f(cookie))
    }

    pub fn len(&self) -> usize {
        self.filters.len()
    }
}

fn trim_parent_path(path: &str, parent: &str) -> String {
    if path == parent {
        return path.to_string();
    }

    // normalize path separators
    let path = if std::path::is_separator('\\') {
        path.replace("\\", "/")
    } else {
        path.to_string()
    };

    let parent = if std::path::is_separator('\\') {
        parent.replace("\\", "/")
    } else {
        parent.to_string()
    };

    // trim parent path
    path.trim_start_matches(&parent)
        .trim_start_matches("/")
        .to_string()
}

////////////////
// Unit tests //
////////////////

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::cookie::{FLAGS_ORDERED, FLAGS_RANDOMIZED, FLAGS_ROTATED};

    use super::{embed::EMBED_PREFIX, CookieShelf};
    const TEST_DATA_DIR: &str = "tests/data";

    // CookieJar tests
    #[test]
    fn test_cookie_jar_default() {
        let jar = super::CookieJar::default();
        assert_eq!(jar.location, "");
        assert_eq!(jar.probability, 0.0);
        assert_eq!(jar.platform, "");
        assert_eq!(jar.version, 0);
        assert_eq!(jar.max_length, 0);
        assert_eq!(jar.min_length, std::u64::MAX);
        assert_eq!(jar.flags, 0);
        assert_eq!(jar.delim, '%');
        assert_eq!(jar.file_size, 0);
        assert_eq!(jar.cookies.len(), 0);
    }

    #[test]
    fn test_cookie_jar_from_text() {
        let testcases = [
            (
                "should handle normal case",
                "Every dog has its day.\n%\nA cat has nine lives.\n%",
                "bay",
                '%',
                2,
                22,
                23,
                48,
            ),
            (
                "should handle delimiter other than '%'",
                "Apple is red.\n|\nOrange is orange.\n|\nBanana is yellow.\n|",
                "quay",
                '|',
                3,
                14,
                18,
                55,
            ),
            (
                "should handle last cookie without delimiter",
                "apple\n#\nbanana",
                "valley",
                '#',
                2,
                6,
                7,
                14,
            ),
            (
                "should ignore empty cookies",
                "apple\n#\nbanana\n#\n\n#\ncherry",
                "meadow",
                '#',
                3,
                6,
                7,
                26,
            ),
        ];

        for (msg, content, location, delim, num_cookies, min_length, max_length, file_size) in
            testcases.iter()
        {
            let jar = super::CookieJar::from_text(content, location, *delim).unwrap();
            assert_eq!(*location, jar.location, "{}", msg);
            assert_eq!(0.0, jar.probability, "{}", msg);
            assert_eq!(*num_cookies, jar.cookies.len(), "{}", msg);
            assert_eq!(
                *min_length,
                jar.min_length,
                "{}: cookies: {:?}",
                msg,
                jar.iter().map(|c| &c.content).collect::<Vec<&String>>()
            );
            assert_eq!(
                *max_length,
                jar.max_length,
                "{}: cookies: {:?}",
                msg,
                jar.cookies
                    .iter()
                    .map(|c| &c.content)
                    .collect::<Vec<&String>>()
            );
            assert_eq!(*file_size, jar.file_size, "{}", msg);
            assert_eq!(*delim, jar.delim, "{}", msg);
        }
    }

    #[test]
    fn test_cookie_jar_filter() {
        let filters = [
            |q: &str| q.len() > 6,
            |q: &str| q.len() < 6,
            |q: &str| q.len() > 5,
            |q: &str| q.len() < 7,
            |q: &str| q.contains("a"),
        ];
        let testcases = [
            (
                "should be able to filter by length (> 6)",
                "apple\n#\nbanana\n#\ncherry!",
                vec![filters[0]],
                1,
                ["cherry!"],
            ),
            (
                "should be able to filter by length (< 6)",
                "apple\n#\nbanana\n#\ncherry!",
                vec![filters[1]],
                1,
                ["apple"],
            ),
            (
                "should be able to filter by length (> 5 and < 7)",
                "apple\n#\nbanana\n#\ncherry!",
                vec![filters[2], filters[3]],
                1,
                ["banana"],
            ),
        ];

        for (msg, content, filters, num_cookies, cookies) in testcases.iter() {
            let mut jar = super::CookieJar::from_text(content, "valley", '#').unwrap();
            let mut sieve = super::CookieSieve::default();
            for filter in filters.iter() {
                sieve.add_filter(filter.clone());
            }
            jar.filter(&sieve).unwrap();
            assert_eq!(
                *num_cookies,
                jar.cookies.len(),
                "{}: num_cookies: expected: {}, got: {}",
                msg,
                num_cookies,
                jar.cookies.len()
            );
            for expected_cookie in cookies.iter() {
                assert!(
                    jar.cookies.iter().any(|c| c.content == *expected_cookie),
                    "{}: cannot find '{}' in the result",
                    msg,
                    expected_cookie
                );
            }
        }
    }

    #[test]
    fn test_cookie_jar_choose() {
        let testcases = [
            ("should return None if no cookies", "", 0),
            ("should return a cookie if there is only one", "apple", 1),
            (
                "should return a cookie if there are multiple",
                "apple\n%\nbanana\n%\ncherry\n%\ndurian\n%\nwatermelon",
                5,
            ),
        ];

        for (msg, content, num_cookies) in testcases.iter() {
            let jar = super::CookieJar::from_text(content, "valley", '%').unwrap();
            let mut rng = rand::thread_rng();

            if *num_cookies == 0 {
                assert!(jar.choose(&mut rng).is_none(), "{}", msg);
            } else {
                assert!(jar.choose(&mut rng).is_some(), "{}", msg);
                //  choose 100 times, and check how many variations are selected
                let mut variations: HashSet<String> = HashSet::new();
                for _ in 0..100 {
                    let cookie = jar.choose(&mut rng).unwrap();
                    variations.insert(cookie.content.clone());
                }
                assert_eq!(*num_cookies, variations.len(), "{}", msg);
            }
        }
    }

    #[test]
    fn test_cookie_jar_update_location() {
        let testcases = [
            (
                "should be able to trim parent path",
                "tests/data/cookie/valley",
                "tests/data",
                "cookie/valley",
            ),
            (
                "should not trim parent path if parent is not a prefix",
                "tests/data/cookie/valley",
                "tests/cookie",
                "tests/data/cookie/valley",
            ),
            (
                "should not trim parent path if path equals parent",
                "tests/data/cookie/valley",
                "tests/data/cookie/valley",
                "tests/data/cookie/valley",
            ),
        ];

        for (msg, path, parent, expected) in testcases.iter() {
            let mut jar = super::CookieJar::default();
            jar.location = path.to_string();
            jar.update_location(parent);
            assert_eq!(
                *expected, jar.location,
                "{}: jar.location: expected: {}, got: {}",
                msg, expected, jar.location
            );
            for c in jar.cookies.iter() {
                assert_eq!(
                    *expected, c.location,
                    "{}: cookie.location: expected: {}, got: {}",
                    msg, expected, c.location
                );
            }
        }
    }

    #[test]
    fn test_cookie_jar_display() {
        let jar = super::CookieJar {
            location: "valley".to_string(),
            probability: 12.345,
            platform: "homebrew".to_string(),
            version: 1,
            max_length: 10,
            min_length: 5,
            flags: FLAGS_ORDERED | FLAGS_RANDOMIZED | FLAGS_ROTATED,
            delim: '%',
            file_size: 100,
            cookies: vec![
                super::Cookie {
                    location: "valley".to_string(),
                    content: "apple".to_string(),
                    offset: 0,
                },
                super::Cookie {
                    location: "valley".to_string(),
                    content: "banana".to_string(),
                    offset: 10,
                },
            ],
        };
        let output = format!("{}", jar);
        assert!(output.contains("CookieJar {"), "got: {}", output);
        assert!(output.contains("location: 'valley'"), "got: {}", output);
        assert!(output.contains("probability: 12.345"), "got: {}", output);
        assert!(output.contains("platform: 'homebrew'"), "got: {}", output);
        assert!(output.contains("version: 1"), "got: {}", output);
        assert!(output.contains("num_cookies: 2"), "got: {}", output);
        assert!(output.contains("max_length: 10"), "got: {}", output);
        assert!(output.contains("min_length: 5"), "got: {}", output);
        assert!(
            output.contains("flags: [RANDOM, ORDERED, ROTATED]"),
            "got: {}",
            output
        );
        assert!(output.contains("delim: '%'"), "got: {}", output);
        assert!(output.contains("file_size: 100"), "got: {}", output);
    }

    // CookieShelf tests
    #[test]
    fn test_cookie_shelf_new() {
        let testcases = [
            (
                "should create a new shelf with given location and probability",
                "valley",
                0.0,
            ),
            (
                "should create a new shelf with given location and probability",
                "valley",
                100.0,
            ),
        ];

        for (msg, location, probability) in testcases.iter() {
            let shelf = super::CookieShelf::new(location, *probability);
            assert_eq!(*location, shelf.location, "{}", msg);
            assert_eq!(*probability, shelf.probability, "{}", msg);
            assert_eq!(0, shelf.jars.len(), "{}", msg);
        }
    }

    #[test]
    fn test_cookie_shelf_num_of_cookies() {
        let mut shelf = super::CookieShelf::new("valley", 0.0);
        shelf
            .jars
            .push(super::CookieJar::from_text("apple\n%\nbanana\n%", "valley", '%').unwrap());
        shelf.jars.push(
            super::CookieJar::from_text("cherry\n%\ndurian\n%\npeach", "valley", '%').unwrap(),
        );
        shelf
            .jars
            .push(super::CookieJar::from_text("", "valley", '%').unwrap());
        assert_eq!(5, shelf.num_of_cookies());
    }

    #[test]
    fn test_cookie_shelf_num_of_jars() {
        let mut shelf = super::CookieShelf::new("valley", 0.0);
        shelf
            .jars
            .push(super::CookieJar::from_text("apple\n%\nbanana\n%", "valley", '%').unwrap());
        shelf.jars.push(
            super::CookieJar::from_text("cherry\n%\ndurian\n%\npeach", "valley", '%').unwrap(),
        );
        shelf
            .jars
            .push(super::CookieJar::from_text("", "valley", '%').unwrap());
        assert_eq!(3, shelf.num_of_jars());
    }

    #[test]
    fn test_cookie_shelf_calculate_prob() {
        let testcases = [
            (
                "should set equal probability to each jar if equal_size is given",
                [2, 3, 0],
                true,
                100.0,
                vec![33.333333333333336, 33.333333333333336, 33.333333333333336],
            ),
            (
                "should set probability to each jar based on the number of cookies if equal_size is not given",
                [2, 3, 0],
                false,
                100.0,
                vec![40.0, 60.0, 0.0],
            ),
            (
                "should set equal probability to each jar if equal_size is given (total: 50.0%)",
                [2, 3, 0],
                true,
                50.0,
                vec![16.666666666666668, 16.666666666666668, 16.666666666666668],
            ),
            (
                "should set probability to each jar based on the number of cookies if equal_size is not given (total: 50.0%)",
                [2, 3, 0],
                false,
                50.0,
                vec![20.0, 30.0, 0.0],
            )
        ];

        for (msg, num_cookies, equal_size, total_prob, expected) in testcases.iter() {
            // create a shelf with given number of cookies
            let mut shelf = super::CookieShelf::new("valley", *total_prob);
            for num in num_cookies.iter() {
                let mut jar = super::CookieJar::default();
                for i in 0..*num {
                    jar.cookies.push(super::Cookie {
                        location: "valley".to_string(),
                        content: "apple".to_string(),
                        offset: i * 10,
                    });
                }
                shelf.jars.push(jar);
            }
            // calculate probability
            shelf.calculate_prob(*equal_size);
            for (i, jar) in shelf.jars.iter().enumerate() {
                assert!(
                    (expected[i] - jar.probability).abs() < 0.0001,
                    "{}: jar.probability: expected: {}, got: {}",
                    msg,
                    expected[i],
                    jar.probability
                );
            }
        }
    }

    #[test]
    fn test_cookie_shelf_load() {
        let testcases = [
            (
                "should load a single file",
                TEST_DATA_DIR.to_string() + "/apple",
                true,
                true,
                1,
                5,
            ),
            (
                "should load a directory (normal only)",
                TEST_DATA_DIR.to_string(),
                true,
                false,
                4,
                11,
            ),
            (
                "should load a directory (offensive only)",
                TEST_DATA_DIR.to_string(),
                false,
                true,
                1,
                1,
            ),
            (
                "should load a directory (normal and offensive)",
                TEST_DATA_DIR.to_string(),
                true,
                true,
                5,
                12,
            ),
            (
                "should load embedded file",
                EMBED_PREFIX.to_string() + "en/fortunes",
                true,
                true,
                1,
                433,
            ),
        ];

        for (msg, location, normal, offensive, num_jars, num_cookies) in testcases.iter() {
            let mut shelf = super::CookieShelf::new(location, 0.0);
            shelf.load(*normal, *offensive).unwrap();
            assert_eq!(
                *num_jars,
                shelf.jars.len(),
                "{}: jars: {:?}",
                msg,
                shelf
                    .jars
                    .iter()
                    .map(|j| &j.location)
                    .collect::<Vec<&String>>()
            );
            assert_eq!(*num_cookies, shelf.num_of_cookies(), "{}", msg);
        }
    }

    #[test]
    fn test_cookie_shelf_filter() {
        let filters = [
            |q: &str| q.len() + 1 > 70,
            |q: &str| q.len() + 1 < 17,
            |q: &str| q.contains("Apple"),
            |q: &str| q.contains("Orange"),
        ];
        let testcases = [
            (
                "should be able to filter by length (> 70)",
                vec![filters[0]],
                1,
                "apple",
            ),
            (
                "should be able to filter by length (< 17)",
                vec![filters[1]],
                1,
                "apple",
            ),
            (
                "should be able to filter by content (contains 'Apple')",
                vec![filters[2]],
                4,
                "apple",
            ),
            (
                "should be able to filter by content (contains 'Orange')",
                vec![filters[3]],
                1,
                "orange",
            ),
        ];

        for (msg, filters, num_of_cookies, expected_jar) in testcases.iter() {
            let mut shelf = CookieShelf::new("tests/data", 100.0);
            shelf.load(true, true).unwrap();
            let mut sieve = super::CookieSieve::default();
            for filter in filters.iter() {
                sieve.add_filter(filter.clone());
            }
            shelf.filter(&sieve).unwrap();
            assert_eq!(
                *num_of_cookies,
                shelf.num_of_cookies(),
                "{}: num_of_cookies: expected: {}, got: {}",
                msg,
                num_of_cookies,
                shelf.num_of_cookies()
            );
            assert_eq!(
                *expected_jar,
                shelf.jars[0].location.as_str(),
                "{}: jar.location: expected: {}, got: {}",
                msg,
                expected_jar,
                shelf.jars[0].location
            );
        }
    }

    #[test]
    fn test_cookie_shelf_choose() {
        let testcases = [
            vec![("apple", 10.0), ("banana", 90.0)],
            vec![("cat", 20.0), ("dog", 30.0), ("elephant", 50.0)],
            vec![("mac", 50.0), ("linux", 30.0), ("windows", 20.0)],
        ];

        let mut rng = rand::thread_rng();
        const REPEAT_COUNT: usize = 1000;
        const THRESHOLD: usize = REPEAT_COUNT / 10;
        for jars in testcases.iter() {
            let mut shelf = super::CookieShelf::new("valley", 100.0);
            for (content, prob) in jars.iter() {
                let mut jar = super::CookieJar::from_text(content, content, '%').unwrap();
                jar.probability = *prob;
                shelf.jars.push(jar);
            }

            // choose REPEATS times, and check how many variations are selected
            let mut results: HashMap<String, usize> = HashMap::new();
            for _ in 0..REPEAT_COUNT {
                let cookie = shelf.choose(&mut rng).unwrap();
                *results.entry(cookie.content.clone()).or_insert(0) += 1;
            }

            // check the number of selected variations
            assert_eq!(jars.len(), results.len());
            for (content, prob) in jars.iter() {
                let choosen_count = results.get(*content).unwrap();
                let expected = (REPEAT_COUNT as f64 * prob / 100.0) as usize;
                assert!(
                    choosen_count.abs_diff(expected) < THRESHOLD,
                    "jar: {}, expected: around {}, got: {}",
                    content,
                    expected,
                    choosen_count
                );
            }
        }
    }

    // CookieCabinet tests
    #[test]
    fn test_cookie_cabinet_new() {
        let cabinet = super::CookieCabinet::new(Vec::new());
        assert_eq!(0, cabinet.shelves.len());
    }

    #[test]
    fn test_cookie_cabinet_push() {
        let mut cabinet = super::CookieCabinet::new(Vec::new());
        cabinet.push(super::CookieShelf::new("valley", 0.0));
        assert_eq!(1, cabinet.shelves.len());
        assert_eq!("valley", cabinet.shelves[0].location);
    }

    #[test]
    fn test_cookie_cabinet_num_of_cookies_and_jars() {
        let mut cabinet = super::CookieCabinet::new(Vec::new());
        cabinet.push(super::CookieShelf::new("valley", 0.0));
        cabinet.push(super::CookieShelf::new("mountain", 0.0));
        cabinet.shelves[0]
            .jars
            .push(super::CookieJar::from_text("apple\n%\nbanana\n%", "valley", '%').unwrap());
        cabinet.shelves[1].jars.push(
            super::CookieJar::from_text("cherry\n%\ndurian\n%\npeach", "mountain", '%').unwrap(),
        );
        assert_eq!(5, cabinet.num_of_cookies());
        assert_eq!(2, cabinet.num_of_jars());
    }

    #[test]
    fn test_cookie_cabinet_calculate_prob() {
        let testcases = [
            (
                [("tests/data", 60.0), ("tests/data2", 40.0)],
                (true, false, false), // equal_size = false
                vec![
                    ("apple", 27.27),
                    ("orange", 27.27),
                    ("one", 5.45),
                    ("zero", 0.0),
                    ("cat", 20.0),
                    ("dog", 20.0),
                ],
            ),
            (
                [("tests/data", 20.0), ("tests/data2", 80.0)],
                (true, false, true), // equal_size = true
                vec![
                    ("apple", 5.0),
                    ("orange", 5.0),
                    ("one", 5.0),
                    ("zero", 5.0),
                    ("cat", 40.0),
                    ("dog", 40.0),
                ],
            ),
            (
                [("tests/data", 0.0), ("tests/data2", 0.0)],
                (true, false, false), // equal_size = false
                vec![
                    ("apple", 23.81),
                    ("orange", 23.81),
                    ("one", 4.76),
                    ("zero", 0.0),
                    ("cat", 23.81),
                    ("dog", 23.81),
                ],
            ),
            (
                [("tests/data", 0.0), ("tests/data2", 0.0)],
                (true, false, true), // equal_size = true
                vec![
                    ("apple", 16.67),
                    ("orange", 16.67),
                    ("one", 16.67),
                    ("zero", 16.67),
                    ("cat", 16.67),
                    ("dog", 16.67),
                ],
            ),
        ];

        for (shelves, (normal, offensive, equal_size), expected_probs) in testcases.iter() {
            let mut cabinet = super::CookieCabinet::default();
            for (location, prob) in shelves.iter() {
                cabinet.push(super::CookieShelf::new(location, *prob));
            }
            cabinet.load(*normal, *offensive).unwrap();
            cabinet.calculate_prob(*equal_size);
            // check the probability
            assert_eq!(
                100.0,
                cabinet.iter().map(|shelf| shelf.probability).sum::<f64>(),
                "total probability != 100%"
            );
            // check the probability for each shelf
            let given_total_probs = shelves.iter().map(|(_, p)| p).sum::<f64>();
            if given_total_probs == 100.0 {
                // only check the probability with given probabilities, otherwise, we don't know the expected value
                for (i, shelf) in cabinet.iter().enumerate() {
                    let sum_of_jars_prob =
                        shelf.jars.iter().map(|jar| jar.probability).sum::<f64>();
                    let (_, expected_prob) = shelves[i];
                    assert_eq!(
                        shelf.probability, sum_of_jars_prob,
                        "shelf[{}].probability sum of jars' != 100%, got: {}",
                        i, sum_of_jars_prob
                    );
                    assert_eq!(
                        expected_prob, shelf.probability,
                        "shelf[{}].probability: expected: {}, got: {}",
                        i, expected_prob, shelf.probability
                    );
                }
            }
            // check the probability for each jar
            let expected_dict: HashMap<String, f64> =
                HashMap::from_iter(expected_probs.iter().map(|(k, v)| (k.to_string(), *v)));
            for shelf in cabinet.iter() {
                for jar in shelf.iter() {
                    let prob = expected_dict
                        .get(&jar.location)
                        .expect(format!("{} not found", jar.location).as_str());
                    assert!(
                        (prob - jar.probability).abs() < 0.01,
                        "jar[{}].probabilit: expected: {}, got: {}",
                        jar.location,
                        prob,
                        jar.probability
                    );
                }
            }
        }
    }

    #[test]
    fn test_cookie_cabinet_from_string_list() {
        let testcases = [
            (
                "60% tests/data 40% tests/data2",
                vec![("tests/data", 60.0), ("tests/data2", 40.0)],
            ),
            (
                "15% tests/data 85% tests/data2",
                vec![("tests/data", 15.0), ("tests/data2", 85.0)],
            ),
            (
                "tests/data tests/data2",
                vec![("tests/data", 0.0), ("tests/data2", 0.0)],
            ),
            ("", vec![(EMBED_PREFIX, 100.0)]),
        ];

        for (line, expected) in testcases.iter() {
            let args = line
                .split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            let cabinet = super::CookieCabinet::from_string_list(&args).unwrap();
            assert_eq!(expected.len(), cabinet.shelves.len());
            for (i, (location, prob)) in expected.iter().enumerate() {
                assert_eq!(*location, cabinet.shelves[i].location);
                assert_eq!(*prob, cabinet.shelves[i].probability);
            }
        }

        //  test error cases
        let args = "15% tests/data 85% tests/data2 10% tests/data3";
        assert!(super::CookieCabinet::from_string_list(
            &args
                .split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        )
        .is_err());
    }

    #[test]
    fn test_cookie_cabinet_filter() {
        let filters = [
            |q: &str| q.len() + 1 > 70,
            |q: &str| q.len() + 1 < 17,
            |q: &str| q.contains("Apple"),
            |q: &str| q.contains("Orange"),
        ];
        let testcases = [
            (
                "should be able to filter by length (> 70)",
                vec![filters[0]],
                1,
                "apple",
            ),
            (
                "should be able to filter by length (< 17)",
                vec![filters[1]],
                1,
                "apple",
            ),
            (
                "should be able to filter by content (contains 'Apple')",
                vec![filters[2]],
                4,
                "apple",
            ),
            (
                "should be able to filter by content (contains 'Orange')",
                vec![filters[3]],
                1,
                "orange",
            ),
        ];

        for (msg, filters, num_of_cookies, expected_jar) in testcases.iter() {
            let mut cabinet = super::CookieCabinet::default();
            cabinet.push(super::CookieShelf::new("tests/data", 100.0));
            cabinet.push(super::CookieShelf::new("tests/data2", 100.0));
            cabinet.load(true, true).unwrap();
            let mut sieve = super::CookieSieve::default();
            for filter in filters.iter() {
                sieve.add_filter(filter.clone());
            }
            cabinet.filter(&sieve).unwrap();
            assert_eq!(
                *num_of_cookies,
                cabinet.num_of_cookies(),
                "{}: num_of_cookies: expected: {}, got: {}",
                msg,
                num_of_cookies,
                cabinet.num_of_cookies()
            );
            assert_eq!(
                *expected_jar,
                cabinet.shelves[0].jars[0].location.as_str(),
                "{}: jar.location: expected: {}, got: {}",
                msg,
                expected_jar,
                cabinet.shelves[0].jars[0].location
            );
        }
    }

    #[test]
    fn test_cookie_cabinet_choose() {
        let testcases = [
            vec![("apple", 10.0), ("banana", 90.0)],
            vec![("cat", 20.0), ("dog", 30.0), ("elephant", 50.0)],
            vec![("mac", 50.0), ("linux", 30.0), ("windows", 20.0)],
        ];

        let mut rng = rand::thread_rng();
        const REPEAT_COUNT: usize = 1000;
        const THRESHOLD: usize = REPEAT_COUNT / 20;
        for jars in testcases.iter() {
            let mut cabinet = super::CookieCabinet::default();
            cabinet.push(super::CookieShelf::new("valley", 100.0));
            for (content, prob) in jars.iter() {
                let mut jar = super::CookieJar::from_text(content, content, '%').unwrap();
                jar.probability = *prob;
                cabinet.shelves[0].jars.push(jar);
            }

            // choose REPEATS times, and check how many variations are selected
            let mut results: HashMap<String, usize> = HashMap::new();
            for _ in 0..REPEAT_COUNT {
                let cookie = cabinet.choose(&mut rng).unwrap();
                *results.entry(cookie.content.clone()).or_insert(0) += 1;
            }

            // check the number of selected variations
            assert_eq!(jars.len(), results.len());
            for (content, prob) in jars.iter() {
                let choosen_count = results.get(*content).unwrap();
                let expected = (REPEAT_COUNT as f64 * prob / 100.0) as usize;
                assert!(
                    choosen_count.abs_diff(expected) < THRESHOLD,
                    "jar: {}, expected: around {}, got: {}",
                    content,
                    expected,
                    choosen_count
                );
            }
        }
    }

    // CookieSieve tests
    #[test]
    fn test_cookie_sieve_add_filter() {
        let mut sieve = super::CookieSieve::default();
        sieve.add_filter(|q| q.len() > 6);
        assert_eq!(1, sieve.len());
    }

    #[test]
    fn test_cookie_sieve_filter() {
        let filters = [
            |q: &str| q.len() > 6,
            |q: &str| q.len() < 6,
            |q: &str| q.len() > 5,
            |q: &str| q.len() < 7,
            |q: &str| q.contains("a"),
        ];
        let testcases = [
            (
                "should be able to filter by length (> 6)",
                "apple",
                vec![filters[0]],
                false,
            ),
            (
                "should be able to filter by length (< 6)",
                "apple",
                vec![filters[1]],
                true,
            ),
            (
                "should be able to filter by length (> 5 and < 7)",
                "banana",
                vec![filters[2], filters[3]],
                true,
            ),
            (
                "should be able to filter by content (contains 'a')",
                "banana",
                vec![filters[4]],
                true,
            ),
        ];

        for (msg, content, filters, expected) in testcases.iter() {
            let mut sieve = super::CookieSieve::default();
            for filter in filters.iter() {
                sieve.add_filter(filter.clone());
            }
            assert_eq!(
                *expected,
                sieve.filter(content),
                "{}: content: {}, expected: {}, got: {}",
                msg,
                content,
                expected,
                sieve.filter(content)
            );
        }
    }

    // trim_parent_path tests
    #[test]
    fn test_trim_parent_path() {
        let testcases = [
            (
                "should be able to trim parent path",
                "tests/data/cookie/valley",
                "tests/data",
                "cookie/valley",
            ),
            (
                "should not trim parent path if parent is not a prefix",
                "tests/data/cookie/valley",
                "tests/cookie",
                "tests/data/cookie/valley",
            ),
            (
                "should not trim parent path if path equals parent",
                "tests/data/cookie/valley",
                "tests/data/cookie/valley",
                "tests/data/cookie/valley",
            ),
        ];

        for (msg, path, parent, expected) in testcases.iter() {
            assert_eq!(
                *expected,
                super::trim_parent_path(path, parent),
                "{}: path: {}, parent: {}, expected: {}, got: {}",
                msg,
                path,
                parent,
                expected,
                super::trim_parent_path(path, parent)
            );
        }
    }
}
