use crate::cookie::{Cookie, CookieJar};

const VERSION_HOMEBREW: u64 = 1;
const VERSION_LINUX: u64 = 2;
const VERSION_FREEBSD: u64 = 1;
const HEADER_SIZE_HOMEBREW: usize = 48;
const HEADER_SIZE_LINUX: usize = 24;
const HEADER_SIZE_FREEBSD: usize = 24;

/// Trait defining the interface for serializing and deserializing CookieJar
/// for different platform formats (Homebrew, Linux, FreeBSD).
pub trait Serialize {
    fn to_bytes(data: &CookieJar) -> Vec<u8>;
    fn from_bytes(bytes: &Vec<u8>) -> CookieJar;
}

/// Implementation of Serializer for Homebrew platform format.
/// Uses 64-bit values for offsets and sizes.
pub struct SerializerHomebrew;
impl Serialize for SerializerHomebrew {
    fn to_bytes(data: &CookieJar) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Metadata fields
        let version = if data.version != 0 {
            data.version
        } else {
            VERSION_HOMEBREW
        };
        bytes.extend_from_slice(&u64_htonl_to_bytes(version));
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.cookies.len() as u64));
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.max_length));
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.min_length));
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.flags));
        bytes.push(data.delim as u8);
        // padding
        bytes.extend_from_slice(&[0; 7]);
        //  offset fields
        let mut offset = 0;
        for cookie in &data.cookies {
            bytes.extend_from_slice(&u64_htonl_to_bytes(offset as u64));
            offset += cookie.content.len() as u64 + 3; // + '\n%\n'
        }
        bytes.extend_from_slice(&u64_htonl_to_bytes(data.file_size));
        bytes
    }

    fn from_bytes(bytes: &Vec<u8>) -> CookieJar {
        let mut data = CookieJar {
            // Metadata fields
            location: "".to_string(),
            probability: 0.0,
            platform: "homebrew".to_string(),
            version: u64_ntohl_from_bytes(bytes[0..8].try_into().unwrap()),
            // num_cookies: u64_ntohl_from_bytes(bytes[8..16].try_into().unwrap()),
            max_length: u64_ntohl_from_bytes(bytes[16..24].try_into().unwrap()),
            min_length: u64_ntohl_from_bytes(bytes[24..32].try_into().unwrap()),
            flags: u64_ntohl_from_bytes(bytes[32..40].try_into().unwrap()),
            delim: bytes[40] as char,
            // offset fields
            cookies: Vec::new(),
            file_size: u64_ntohl_from_bytes(
                bytes[bytes.len() - 8..bytes.len()].try_into().unwrap(),
            ),
        };
        for _ in (HEADER_SIZE_HOMEBREW..bytes.len() - 8).step_by(8) {
            data.cookies.push(Cookie {
                location: "".to_string(),
                content: "".to_string(),
            });
        }
        let num_cookies = u64_ntohl_from_bytes(bytes[8..16].try_into().unwrap());
        assert!(
            num_cookies == data.cookies.len() as u64,
            "Error: Inconsistent number of cookies. num_cookies: {}, cookies.len(): {}",
            num_cookies,
            data.cookies.len()
        );
        data
    }
}

/// Implementation of Serializer for Linux platform format.
/// Uses 32-bit values for offsets and sizes.
pub struct SerializerLinux;
impl Serialize for SerializerLinux {
    fn to_bytes(data: &CookieJar) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Metadata fields
        let version = if data.version != 0 {
            data.version
        } else {
            VERSION_LINUX
        };
        bytes.extend_from_slice(&(version as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.cookies.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.max_length as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.min_length as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.flags as u32).to_be_bytes());
        bytes.push(data.delim as u8);
        // padding
        bytes.extend_from_slice(&[0; 7]);
        //  offset fields
        let mut offset: u32 = 0;
        for cookie in &data.cookies {
            bytes.extend_from_slice(&offset.to_be_bytes());
            offset += cookie.content.len() as u32 + 3; // + '\n%\n'
        }
        bytes.extend_from_slice(&(data.file_size as u32).to_be_bytes());
        bytes
    }

    fn from_bytes(bytes: &Vec<u8>) -> CookieJar {
        let mut data = CookieJar {
            // Metadata fields
            location: "".to_string(),
            probability: 0.0,
            platform: "linux".to_string(),
            version: u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as u64,
            // num_cookies: u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as u64,
            max_length: u32::from_be_bytes(bytes[8..12].try_into().unwrap()) as u64,
            min_length: u32::from_be_bytes(bytes[12..16].try_into().unwrap()) as u64,
            flags: u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as u64,
            delim: bytes[20] as char,
            // offset fields
            cookies: Vec::new(),
            file_size: u32::from_be_bytes(bytes[bytes.len() - 4..bytes.len()].try_into().unwrap())
                as u64,
        };
        for _ in (HEADER_SIZE_LINUX..bytes.len() - 4).step_by(4) {
            data.cookies.push(Cookie {
                location: "".to_string(),
                content: "".to_string(),
                // offset: u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap()) as u64,
            });
        }
        let num_cookies = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as u64;
        assert!(
            num_cookies == data.cookies.len() as u64,
            "Error: Inconsistent number of cookies. num_cookies: {}, cookies.len(): {}",
            num_cookies,
            data.cookies.len()
        );
        data
    }
}

/// Implementation of Serializer for FreeBSD platform format.
/// Uses 64-bit values for offsets and sizes, with a different byte order than Homebrew.
pub struct SerializerFreeBSD;
impl Serialize for SerializerFreeBSD {
    fn to_bytes(data: &CookieJar) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Metadata fields
        let version = if data.version != 0 {
            data.version
        } else {
            VERSION_FREEBSD
        };
        bytes.extend_from_slice(&(version as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.cookies.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.max_length as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.min_length as u32).to_be_bytes());
        bytes.extend_from_slice(&(data.flags as u32).to_be_bytes());
        bytes.push(data.delim as u8);
        // padding
        bytes.extend_from_slice(&[0; 7]);
        //  offset fields
        let mut offset: u64 = 0;
        for cookie in &data.cookies {
            bytes.extend_from_slice(&offset.to_be_bytes());
            offset += cookie.content.len() as u64 + 3; // + '\n%\n'
        }
        bytes.extend_from_slice(&data.file_size.to_be_bytes());
        bytes
    }

    fn from_bytes(bytes: &Vec<u8>) -> CookieJar {
        let mut data = CookieJar {
            // Metadata fields
            location: "".to_string(),
            probability: 0.0,
            platform: "freebsd".to_string(),
            version: u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as u64,
            // num_cookies: u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as u64,
            max_length: u32::from_be_bytes(bytes[8..12].try_into().unwrap()) as u64,
            min_length: u32::from_be_bytes(bytes[12..16].try_into().unwrap()) as u64,
            flags: u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as u64,
            delim: bytes[20] as char,
            // offset fields
            cookies: Vec::new(),
            file_size: u64::from_be_bytes(bytes[bytes.len() - 8..bytes.len()].try_into().unwrap()),
        };
        for _ in (HEADER_SIZE_FREEBSD..bytes.len() - 8).step_by(8) {
            data.cookies.push(Cookie {
                location: "".to_string(),
                content: "".to_string(),
                // offset: u64::from_be_bytes(bytes[i..i + 8].try_into().unwrap()),
            });
        }
        let num_cookies = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as u64;
        assert!(
            num_cookies == data.cookies.len() as u64,
            "Error: Inconsistent number of cookies. num_cookies: {}, cookies.len(): {}",
            num_cookies,
            data.cookies.len()
        );
        data
    }
}

/// Enum representing the different platform serialization formats.
#[derive(Debug, PartialEq, Eq)]
pub enum SerializerType {
    Homebrew,
    Linux,
    FreeBSD,
}

pub struct Serializer;
impl Serializer {
    pub fn to_bytes(data: &CookieJar, t: SerializerType) -> Vec<u8> {
        match t {
            SerializerType::Homebrew => SerializerHomebrew::to_bytes(data),
            SerializerType::Linux => SerializerLinux::to_bytes(data),
            SerializerType::FreeBSD => SerializerFreeBSD::to_bytes(data),
        }
    }

    pub fn from_bytes(bytes: &Vec<u8>, t: SerializerType) -> CookieJar {
        match t {
            SerializerType::Homebrew => SerializerHomebrew::from_bytes(bytes),
            SerializerType::Linux => SerializerLinux::from_bytes(bytes),
            SerializerType::FreeBSD => SerializerFreeBSD::from_bytes(bytes),
        }
    }

    pub fn get_type_by_platform(platform: &str) -> SerializerType {
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
            return Serializer::get_type_by_current_platform(); // Default to current platform
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

    pub fn get_current_platform() -> String {
        let platform = std::env::consts::OS;
        match platform {
            "macos" => "homebrew".to_string(),
            "linux" => "linux".to_string(),
            "freebsd" => "freebsd".to_string(),
            _ => "linux".to_string(), // Default to Linux format
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    // Test cases for u64_htonl_to_bytes() and u64_ntohl_from_bytes()
    #[test]
    fn test_u64_htonl_to_bytes() {
        let testcases = [
            (
                0x1234567890ABCDEF,
                [0x90, 0xAB, 0xCD, 0xEF, 0x00, 0x00, 0x00, 0x00],
            ),
            (
                0x0000000000000001,
                [0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
            ),
            (
                0x1234567800000000,
                [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ),
        ];
        for (n, expected) in testcases.iter() {
            assert_eq!(u64_htonl_to_bytes(*n), *expected);
        }
    }

    #[test]
    fn test_u64_ntohl_from_bytes() {
        let testcases = [
            (
                [0x90, 0xAB, 0xCD, 0xEF, 0x00, 0x00, 0x00, 0x00],
                0x0000000090ABCDEF,
            ),
            (
                [0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
                0x0000000000000001,
            ),
        ];
        for (bytes, expected) in testcases.iter() {
            assert_eq!(u64_ntohl_from_bytes(*bytes), *expected);
        }
    }

    // Test cases for Serializer
    #[test]
    fn test_serializer_get_type_by_platform() {
        assert_eq!(
            Serializer::get_type_by_platform("homebrew"),
            SerializerType::Homebrew
        );
        assert_eq!(
            Serializer::get_type_by_platform("linux"),
            SerializerType::Linux
        );
        assert_eq!(
            Serializer::get_type_by_platform("freebsd"),
            SerializerType::FreeBSD
        );
        // assert_eq!(Serializer::get_type_by_platform("unknown"), SerializerType::Linux);
    }

    #[test]
    fn test_serializer_get_type_by_current_platform() {
        if std::env::consts::OS == "macos" {
            assert_eq!(
                Serializer::get_type_by_current_platform(),
                SerializerType::Homebrew
            );
        } else if std::env::consts::OS == "linux" {
            assert_eq!(
                Serializer::get_type_by_current_platform(),
                SerializerType::Linux
            );
        } else if std::env::consts::OS == "freebsd" {
            assert_eq!(
                Serializer::get_type_by_current_platform(),
                SerializerType::FreeBSD
            );
        } else {
            // assert_eq!(Serializer::get_type_by_current_platform(), SerializerType::Linux);
        }
    }

    #[test]
    fn test_serializer_get_current_platform() {
        if std::env::consts::OS == "macos" {
            assert_eq!(Serializer::get_current_platform(), "homebrew");
        } else if std::env::consts::OS == "linux" {
            assert_eq!(Serializer::get_current_platform(), "linux");
        } else if std::env::consts::OS == "freebsd" {
            assert_eq!(Serializer::get_current_platform(), "freebsd");
        } else {
            // assert_eq!(Serializer::get_current_platform(), "linux");
        }
    }
}
