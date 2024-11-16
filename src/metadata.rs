
/// Converts a u64 value to network byte order (big-endian) and returns it as a byte array.
/// This function mimics the behavior of the original C implementation's htonl() function.
///
/// The conversion process:
/// 1. Truncates the input to 32 bits (as u32)
/// 2. Converts to big-endian byte order
/// 3. Converts back to u64
/// 4. Returns the bytes in little-endian order
///
/// # Arguments
/// * `n` - The u64 value to convert
///
/// # Returns
/// * `[u8; 8]` - The converted bytes in little-endian order
fn u64_htonl_to_bytes(n: u64) -> [u8; 8] {
    // from strfile.c:
    //    off_t off;
    //    off_t net = htonl(off); // htonl() truncates to 32 bits
    //    fwrite(&net, 1, sizeof net, fp);
    //    ...
    // 0x1234567890ABCDEF
    //      --(as u32)-->       0x90ABCDEF
    //      --(u32::to_be)-->   0xEFCDAB90
    //      --(as u64)-->       0x00000000EFCDAB90
    //      --(to_le_bytes)-->  [0x90, 0xAB, 0xCD, 0xEF, 0x00, 0x00, 0x00, 0x00]
    (u32::to_be(n as u32) as u64).to_le_bytes()
}

/// Converts a byte array from network byte order (big-endian) back to a u64 value.
/// This function is the inverse of u64_htonl_to_bytes().
///
/// # Arguments
/// * `bytes` - The byte array to convert
///
/// # Returns
/// * `u64` - The converted value
fn u64_ntohl_from_bytes(bytes: [u8; 8]) -> u64 {
    // [0x90, 0xAB, 0xCD, 0xEF, 0x00, 0x00, 0x00, 0x00]
    //      --(from_le_bytes)--> 0x00000000EFCDAB90
    //      --(as u32)-->       0xEFCDAB90
    //      --(u32::from_be)--> 0x90ABCDEF
    //      --(as u64)-->       0x0000000090ABCDEF
    (u64::from_le_bytes(bytes) as u32).to_be() as u64
}

// Utility function for debugging byte arrays (commented out)
// fn bytes_to_string(bytes: &[u8]) -> String {
//     let mut values: Vec<String> = Vec::new();
//     for byte in bytes.iter() {
//         values.push(format!("{:02X}", byte));
//     }
//     format!("[{}]", values.join(" "))
// }

/// Constants defining the data file format and flags
pub const FLAGS_RANDOMIZED: u64 = 0x0001; /* randomized pointers */
pub const FLAGS_ORDERED: u64 = 0x0002; /* ordered pointers */
pub const FLAGS_ROTATED: u64 = 0x0004; /* rot-13'd pointers */
const VERSION_HOMEBREW: u64 = 1;
const VERSION_LINUX: u64 = 2;
const VERSION_FREEBSD: u64 = 1;
const HEADER_SIZE_HOMEBREW: usize = 48;
const HEADER_SIZE_LINUX: usize = 24;
const HEADER_SIZE_FREEBSD: usize = 24;

/// Represents a single fortune cookie quote with its text and position in file.
#[derive(Default)]
pub struct Quote {
    pub content: String, // The actual quote text
    pub offset: u64,     // Byte offset of quote in the source file
}

/// Represents the header structure of a fortune cookie data file.
/// This header contains metadata about the fortune cookie strings.
pub struct CookieMetadata {
    pub platform: String, // Platform to use for serialization, one of: homebrew, linux, freebsd
    pub version: u64,     // Data file format version
    pub num_quotes: u64,  // Number of strings in file
    pub max_length: u64,  // Length of longest string
    pub min_length: u64,  // Length of shortest string
    pub flags: u64,       // File flags (random, ordered, rotated)
    pub delim: char,      // Delimiting character
    pub file_size: u64,   // Total size of source file
    pub quotes: Vec<Quote>, // Offsets of each string in the file
}

impl Default for CookieMetadata {
    /// Creates a new CookieMetadata with default values.
    /// Sets delimiter to '%'.
    fn default() -> Self {
        Self {
            platform: "".to_string(),
            version: 0,
            num_quotes: 0,
            max_length: 0,
            min_length: u64::MAX,
            flags: 0,
            delim: '%',
            file_size: 0,
            quotes: Vec::new(),
        }
    }
}

impl std::fmt::Display for CookieMetadata {
    /// Formats the CookieMetadata struct for display.
    /// Shows the header fields and quote offsets.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CookieMetadata {{\n")?;
        write!(f, "  platform: '{}'\n", self.platform)?;
        write!(f, "  version: {}\n", self.version)?;
        write!(f, "  num_quotes: {}\n", self.num_quotes)?;
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

        write!(
            f,
            "  offsets: [{}]\n",
            self.quotes
                .iter()
                .map(|quote| format!("{}", quote.offset))
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        write!(f, "}}")
    }
}

impl CookieMetadata {
    /// Loads the metadata from a fortune cookie file.
    /// Parses the file content and extracts the metadata fields.
    /// The file content is expected to be in the format of a fortune cookie file.
    /// The delimiter character is set to '%' by default.
    /// # Arguments
    /// * `filename` - The path to the fortune cookie file
    pub fn load_from_cookie_file(&mut self, filename: &str) {
        let content = std::fs::read_to_string(filename).unwrap_or_else(|_| {
            eprintln!("Error reading cookie file: {}", filename);
            std::process::exit(1);
        });
        // Split content by delimiter pattern
        let splitter = format!("\n{}\n", self.delim);
        let parts: Vec<&str> = content.split(splitter.as_str()).collect();
        let mut offset = 0;

        // Process each quote, tracking offsets and updating metaself
        for part in &parts {
            if part.trim().is_empty() {
                continue;
            }
            self.quotes.push(Quote {
                content: part.to_string(),
                offset: offset,
            });
            let len = part.len() as u64 + 1; // 1 = len('\n')
            offset += len + 2; // 2 = len('%\n')
                            // Update max_length and min_length
            self.max_length = self.max_length.max(len);
            if len > 1 {
                self.min_length = self.min_length.min(len);
            }
        }
        self.num_quotes = self.quotes.len() as u64;
        self.file_size = content.len() as u64;
    }
}


/// Trait defining the interface for serializing and deserializing CookieMetadata
/// for different platform formats (Homebrew, Linux, FreeBSD).
pub trait Serialize {
    fn to_bytes(data: &CookieMetadata) -> Vec<u8>;
    fn from_bytes(bytes: &Vec<u8>) -> CookieMetadata;
}

/// Implementation of Serializer for Homebrew platform format.
/// Uses 64-bit values for offsets and sizes.
pub struct SerializerHomebrew;
impl Serialize for SerializerHomebrew {
    fn to_bytes(data: &CookieMetadata) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Metadata fields
        let version = if data.version != 0 {
            data.version
        } else {
            VERSION_HOMEBREW
        };
        bytes.extend_from_slice(&u64_htonl_to_bytes(version));
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.num_quotes));
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.max_length));
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.min_length));
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.flags));
        bytes.push(data.delim as u8);
        bytes.extend_from_slice(&[0; 7]); // padding
                                          //  offset fields
        for quote in &data.quotes {
            bytes.extend_from_slice(&u64_htonl_to_bytes(quote.offset));
        }
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.file_size));
        bytes
    }

    fn from_bytes(bytes: &Vec<u8>) -> CookieMetadata {
        let mut data = CookieMetadata {
            // Metadata fields
            platform: "homebrew".to_string(),
            version: u64_ntohl_from_bytes(bytes[0..8].try_into().unwrap()),
            num_quotes: u64_ntohl_from_bytes(bytes[8..16].try_into().unwrap()),
            max_length: u64_ntohl_from_bytes(bytes[16..24].try_into().unwrap()),
            min_length: u64_ntohl_from_bytes(bytes[24..32].try_into().unwrap()),
            flags: u64_ntohl_from_bytes(bytes[32..40].try_into().unwrap()),
            delim: bytes[40] as char,
            // offset fields
            quotes: Vec::new(),
            file_size: u64_ntohl_from_bytes(
                bytes[bytes.len() - 8..bytes.len()].try_into().unwrap(),
            ),
        };
        for i in (HEADER_SIZE_HOMEBREW..bytes.len() - 8).step_by(8) {
            data.quotes.push(Quote {
                content: "".to_string(),
                offset: u64_ntohl_from_bytes(bytes[i..i + 8].try_into().unwrap()),
            });
        }
        data
    }
}

/// Implementation of CookieMetadataSerializer for Linux platform format.
/// Uses 32-bit values for offsets and sizes.
pub struct SerializerLinux;
impl Serialize for SerializerLinux {
    fn to_bytes(data: &CookieMetadata) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Metadata fields
        let version = if data.version != 0 {
            data.version
        } else {
            VERSION_LINUX
        };
        bytes.extend_from_slice(&(version as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.num_quotes as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.max_length as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.min_length as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.flags as u32).to_be_bytes());
        bytes.push(data.delim as u8);
        bytes.extend_from_slice(&[0; 7]); // padding
                                          //  offset fields
        for quote in &data.quotes {
            bytes.extend_from_slice(&(quote.offset as u32).to_be_bytes());
        }
        bytes.extend_from_slice(&(data.file_size as u32).to_be_bytes());
        bytes
    }

    fn from_bytes(bytes: &Vec<u8>) -> CookieMetadata {
        let mut data = CookieMetadata {
            // Metadata fields
            platform: "linux".to_string(),
            version: u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as u64,
            num_quotes: u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as u64,
            max_length: u32::from_be_bytes(bytes[8..12].try_into().unwrap()) as u64,
            min_length: u32::from_be_bytes(bytes[12..16].try_into().unwrap()) as u64,
            flags: u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as u64,
            delim: bytes[20] as char,
            // offset fields
            quotes: Vec::new(),
            file_size: u32::from_be_bytes(bytes[bytes.len() - 4..bytes.len()].try_into().unwrap())
                as u64,
        };
        for i in (HEADER_SIZE_LINUX..bytes.len() - 4).step_by(4) {
            data.quotes.push(Quote {
                content: "".to_string(),
                offset: u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap()) as u64,
            });
        }
        println!(
            "data.num_quotes = {}, data.quotes.len() = {}",
            data.num_quotes,
            data.quotes.len()
        );
        assert!(data.num_quotes == data.quotes.len() as u64);
        data
    }
}

/// Implementation of CookieMetadataSerializer for FreeBSD platform format.
/// Uses 64-bit values for offsets and sizes, with a different byte order than Homebrew.
pub struct SerializerFreeBSD;
impl Serialize for SerializerFreeBSD {
    fn to_bytes(data: &CookieMetadata) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Metadata fields
        let version = if data.version != 0 {
            data.version
        } else {
            VERSION_FREEBSD
        };
        bytes.extend_from_slice(&(version as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.num_quotes as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.max_length as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.min_length as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.flags as u32).to_be_bytes());
        bytes.push(data.delim as u8);
        bytes.extend_from_slice(&[0; 7]); // padding
                                          //  offset fields
        for quote in &data.quotes {
            bytes.extend_from_slice(&quote.offset.to_be_bytes());
        }
        bytes.extend_from_slice(&data.file_size.to_be_bytes());
        bytes
    }

    fn from_bytes(bytes: &Vec<u8>) -> CookieMetadata {
        let mut data = CookieMetadata {
            // Metadata fields
            platform: "freebsd".to_string(),
            version: u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as u64,
            num_quotes: u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as u64,
            max_length: u32::from_be_bytes(bytes[8..12].try_into().unwrap()) as u64,
            min_length: u32::from_be_bytes(bytes[12..16].try_into().unwrap()) as u64,
            flags: u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as u64,
            delim: bytes[20] as char,
            // offset fields
            quotes: Vec::new(),
            file_size: u64::from_be_bytes(bytes[bytes.len() - 8..bytes.len()].try_into().unwrap()),
        };
        for i in (HEADER_SIZE_FREEBSD..bytes.len() - 8).step_by(8) {
            data.quotes.push(Quote {
                content: "".to_string(),
                offset: u64::from_be_bytes(bytes[i..i + 8].try_into().unwrap()),
            });
        }
        data
    }
}

/// Enum representing the different platform serialization formats.
#[derive(PartialEq, Eq)]
pub enum SerializerType {
    Homebrew,
    Linux,
    FreeBSD,
}

pub struct Serializer;
impl Serializer {
    pub fn to_bytes(data: &CookieMetadata, t: SerializerType) -> Vec<u8> {
        match t {
            SerializerType::Homebrew => SerializerHomebrew::to_bytes(data),
            SerializerType::Linux => SerializerLinux::to_bytes(data),
            SerializerType::FreeBSD => SerializerFreeBSD::to_bytes(data),
        }
    }

    pub fn from_bytes(bytes: &Vec<u8>, t: SerializerType) -> CookieMetadata {
        match t {
            SerializerType::Homebrew => SerializerHomebrew::from_bytes(bytes),
            SerializerType::Linux => SerializerLinux::from_bytes(bytes),
            SerializerType::FreeBSD => SerializerFreeBSD::from_bytes(bytes),
        }
    }

    pub fn get_type_by_name(platform: &str) -> SerializerType {
        match platform {
            "homebrew" => SerializerType::Homebrew,
            "linux" => SerializerType::Linux,
            "freebsd" => SerializerType::FreeBSD,
            _ => Serializer::get_type_by_current_platform(), // Default to current platform
        }
    }

    pub fn get_type_by_bytes(bytes: &Vec<u8>) -> SerializerType {
        // Detect file format based on byte patterns
        if bytes[0..8] == [0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00] {
            // Homebrew format has strange version of 64-bit big-endian
            return SerializerType::Homebrew;
        } else if bytes[0..4] == [0x00, 0x00, 0x00, 0x01]
            && bytes[30..34] == [0x00, 0x00, 0x00, 0x00]
            && bytes[40..44] == [0x00, 0x00, 0x00, 0x00]
        {
            // FreeBSD format has version 1, 32-bits header and 64-bits for offsets
            // since the offsets are 64-bits, so the high 32-bits are always zero
            return SerializerType::FreeBSD;
        } else if bytes[0..4] == [0x00, 0x00, 0x00, 0x02]
            && bytes[4..8] != [0x00, 0x00, 0x00, 0x00]
            && bytes[30..34] != [0x00, 0x00, 0x00, 0x00]
            && bytes[34..38] != [0x00, 0x00, 0x00, 0x00]
        {
            // Linux format has version 2, 32-bits header and 32-bits for offsets
            return SerializerType::Linux;
        } else {
            return Serializer::get_type_by_current_platform();   // Default to current platform
        }
    }

    pub fn get_type_by_current_platform() -> SerializerType {
        let platform = std::env::consts::OS;
        match platform {
            "macos" => SerializerType::Homebrew,
            "linux" => SerializerType::Linux,
            "freebsd" => SerializerType::FreeBSD,
            _ => SerializerType::Linux, // Default to Linux format
        }
    }
}
