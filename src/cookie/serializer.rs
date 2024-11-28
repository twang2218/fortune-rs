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
            if cookie.offset != 0 {
                // If the offset is already set, use it
                bytes.extend_from_slice(&u64_htonl_to_bytes(cookie.offset));
            } else {
                // Otherwise, calculate the offset based on the current position
                bytes.extend_from_slice(&u64_htonl_to_bytes(offset as u64));
            }
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
        for i in (HEADER_SIZE_HOMEBREW..bytes.len() - 8).step_by(8) {
            data.cookies.push(Cookie {
                location: "".to_string(),
                content: "".to_string(),
                offset: u64_ntohl_from_bytes(bytes[i..i + 8].try_into().unwrap()),
            });
        }
        // let num_cookies = u64_ntohl_from_bytes(bytes[8..16].try_into().unwrap());
        // assert!(
        //     num_cookies == data.cookies.len() as u64,
        //     "Error: Inconsistent number of cookies. num_cookies: {}, cookies.len(): {}",
        //     num_cookies,
        //     data.cookies.len()
        // );
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
        bytes.extend_from_slice(&[0; 3]);
        //  offset fields
        let mut offset: u32 = 0;
        for cookie in &data.cookies {
            if cookie.offset != 0 {
                // If the offset is already set, use it
                bytes.extend_from_slice(&(cookie.offset as u32).to_be_bytes());
            } else {
                // Otherwise, calculate the offset based on the current position
                bytes.extend_from_slice(&(offset as u32).to_be_bytes());
            }
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
        for i in (HEADER_SIZE_LINUX..bytes.len() - 4).step_by(4) {
            data.cookies.push(Cookie {
                location: "".to_string(),
                content: "".to_string(),
                offset: u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap()) as u64,
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
        bytes.extend_from_slice(&[0; 3]);
        //  offset fields
        let mut offset: u64 = 0;
        for cookie in &data.cookies {
            if cookie.offset != 0 {
                // If the offset is already set, use it
                bytes.extend_from_slice(&(cookie.offset as u64).to_be_bytes());
            } else {
                // Otherwise, calculate the offset based on the current position
                bytes.extend_from_slice(&(offset as u64).to_be_bytes());
            }
            offset += cookie.content.len() as u64 + 3; // + '\n%\n'
        }
        bytes.extend_from_slice(&(data.file_size as u64).to_be_bytes());
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
        for i in (HEADER_SIZE_FREEBSD..bytes.len() - 8).step_by(8) {
            data.cookies.push(Cookie {
                location: "".to_string(),
                content: "".to_string(),
                offset: u64::from_be_bytes(bytes[i..i + 8].try_into().unwrap()),
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
    pub fn to_bytes(data: &CookieJar, t: &SerializerType) -> Vec<u8> {
        match t {
            SerializerType::Homebrew => SerializerHomebrew::to_bytes(data),
            SerializerType::Linux => SerializerLinux::to_bytes(data),
            SerializerType::FreeBSD => SerializerFreeBSD::to_bytes(data),
        }
    }

    pub fn from_bytes(bytes: &Vec<u8>, t: &SerializerType) -> CookieJar {
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

    pub fn get_platform_by_type(t: &SerializerType) -> String {
        match t {
            SerializerType::Homebrew => "homebrew".to_string(),
            SerializerType::Linux => "linux".to_string(),
            SerializerType::FreeBSD => "freebsd".to_string(),
        }
    }

    pub fn get_type_by_bytes(bytes: &Vec<u8>) -> SerializerType {
        // Detect file format based on byte patterns
        if bytes[0..8] == [0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00] {
            // Homebrew format has strange version of 64-bit big-endian
            return SerializerType::Homebrew;
        } else if bytes[0..4] == [0x00, 0x00, 0x00, 0x01]
            && bytes[24..28] == [0x00, 0x00, 0x00, 0x00]
            && bytes[32..36] == [0x00, 0x00, 0x00, 0x00]
        {
            // FreeBSD format has version 1, 32-bits header and 64-bits for offsets
            // since the offsets are 64-bits, so the high 32-bits are always zero
            return SerializerType::FreeBSD;
        } else if bytes[0..4] == [0x00, 0x00, 0x00, 0x02]
            && bytes[4..8] != [0x00, 0x00, 0x00, 0x00]
            && bytes[28..32] != [0x00, 0x00, 0x00, 0x00]
            && bytes[32..36] != [0x00, 0x00, 0x00, 0x00]
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
    use crate::cookie::{FLAGS_ORDERED, FLAGS_RANDOMIZED, FLAGS_ROTATED};

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

    fn get_testcases_for_bytes() -> Vec<(
        Vec<u8>,
        (SerializerType, u64, u64, u64, u64, u64, char, Vec<u64>, u64),
    )> {
        vec![
            (
                vec![
                    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, // version 1
                    0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, // num_cookies 2
                    0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, // max_length 16
                    0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, // min_length 5
                    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, // flags FLAGS_RANDOMIZED
                    0x25, // delim '%'
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // padding
                    0x00, 0x00, 0x00, 0x23, 0x00, 0x00, 0x00, 0x00, // offset 35
                    0x00, 0x00, 0x00, 0x78, 0x00, 0x00, 0x00, 0x00, // offset 120
                    0x00, 0x00, 0x00, 0xA0, 0x00, 0x00, 0x00, 0x00, // file_size 160
                ],
                (
                    SerializerType::Homebrew, // Type
                    1,                        // version
                    2,                        // num_cookies
                    16,                       // max_length
                    5,                        // min_length
                    FLAGS_RANDOMIZED,         // flags
                    '%',                      // delim
                    vec![35, 120],            // offsets
                    160,                      // file_size
                ),
            ),
            (
                vec![
                    0x00, 0x00, 0x00, 0x02, // version 2
                    0x00, 0x00, 0x00, 0x02, // num_cookies 2
                    0x00, 0x00, 0x00, 0x11, // max_length 17
                    0x00, 0x00, 0x00, 0x06, // min_length 6
                    0x00, 0x00, 0x00, 0x02, // flags FLAGS_ORDERED
                    0x25, // delim '%'
                    0x00, 0x00, 0x00, // padding
                    0x00, 0x00, 0x00, 0x30, // offset 48
                    0x00, 0x00, 0x00, 0x80, // offset 128
                    0x00, 0x00, 0x00, 0xB0, // file_size 176
                ],
                (
                    SerializerType::Linux, // Type
                    2,                     // version
                    2,                     // num_cookies
                    17,                    // max_length
                    6,                     // min_length
                    FLAGS_ORDERED,         // flags
                    '%',                   // delim
                    vec![48, 128],         // offsets
                    176,                   // file_size
                ),
            ),
            (
                vec![
                    0x00, 0x00, 0x00, 0x01, // version 1
                    0x00, 0x00, 0x00, 0x02, // num_cookies 2
                    0x00, 0x00, 0x00, 0x13, // max_length 19
                    0x00, 0x00, 0x00, 0x09, // min_length 9
                    0x00, 0x00, 0x00, 0x04, // flags FLAGS_ROTATED
                    0x25, // delim '%'
                    0x00, 0x00, 0x00, // padding
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, // offset 8
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x38, // offset 56
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, // offset 192
                ],
                (
                    SerializerType::FreeBSD, // Type
                    1,                       // version
                    2,                       // num_cookies
                    19,                      // max_length
                    9,                       // min_length
                    FLAGS_ROTATED,           // flags
                    '%',                     // delim
                    vec![8, 56],             // offsets
                    192,                     // file_size
                ),
            ),
        ]
    }

    #[test]
    fn test_serializer_get_type_by_bytes() {
        let testcases = get_testcases_for_bytes();
        for (bytes, expected) in testcases.iter() {
            let t = Serializer::get_type_by_bytes(&bytes.to_vec());
            let (expected_type, _, _, _, _, _, _, _, _) = expected;
            assert_eq!(
                *expected_type, t,
                "Got wrong SerializerType: expected: {:?}, got: {:?}",
                expected_type, t
            );
        }
    }

    #[test]
    fn test_serializer_from_bytes() {
        let testcases = get_testcases_for_bytes();
        for (bytes, expected) in testcases.iter() {
            let (
                expected_type,
                expected_version,
                expected_num_cookies,
                expected_max_length,
                expected_min_length,
                expected_flags,
                expected_delim,
                expected_offsets,
                expected_file_size,
            ) = expected;
            let expected_msg = format!(
                "Expected: \ntype: {:?}, version: {}, num_cookies: {}, max_length: {}, min_length: {}, flags: {}, delim: {}, offsets: {:?}",
                expected_type, expected_version, expected_num_cookies, expected_max_length, expected_min_length, expected_flags, expected_delim, expected_offsets
            );
            let data = Serializer::from_bytes(&bytes.to_vec(), &expected_type);
            let msg = format!("{}\nGot: {:?}", expected_msg, data);
            assert_eq!(*expected_version, data.version, "[wrong version]: {}", msg);
            assert_eq!(
                *expected_max_length, data.max_length,
                "[wrong max_length]: {}",
                msg
            );
            assert_eq!(
                *expected_min_length, data.min_length,
                "[wrong min_length]: {}",
                msg
            );
            assert_eq!(*expected_flags, data.flags, "[wrong flags]: {}", msg);
            assert_eq!(*expected_delim, data.delim, "[wrong delim]: {}", msg);
            assert_eq!(
                *expected_file_size, data.file_size,
                "[wrong file_size]: {}",
                msg
            );
            assert_eq!(
                *expected_num_cookies,
                data.cookies.len() as u64,
                "[wrong cookies.len()]: {}",
                msg
            );
            for (i, offset) in expected_offsets.iter().enumerate() {
                assert_eq!(*offset, data.cookies[i].offset, "[wrong offset]: {}", msg);
            }
        }
    }

    #[test]
    fn test_serializer_to_bytes() {
        let testcases = get_testcases_for_bytes();

        for (expected, given) in testcases.iter() {
            let (t, version, num_cookies, max_length, min_length, flags, delim, offsets, file_size) =
                given;
            let mut data = CookieJar {
                location: "".to_string(),
                probability: 0.0,
                platform: Serializer::get_platform_by_type(t),
                version: *version,
                cookies: Vec::new(),
                max_length: *max_length,
                min_length: *min_length,
                flags: *flags,
                delim: *delim,
                file_size: *file_size,
            };
            for offset in offsets.iter() {
                data.cookies.push(Cookie {
                    location: "".to_string(),
                    content: "".to_string(),
                    offset: *offset,
                });
            }
            assert_eq!(
                *num_cookies,
                data.cookies.len() as u64,
                "num_cookies != cookies.len()"
            );
            // let given_msg = format!(
            //     "Given: \ntype: {:?}, version: {}, num_cookies: {}, max_length: {}, min_length: {}, flags: {}, delim: {}, offsets: {:?}",
            //     t, version, num_cookies, max_length, min_length, flags, delim, offsets
            // );
            let given_msg = format!("Given: {:?}", data);
            let bytes = Serializer::to_bytes(&data, t);
            assert_eq!(expected.to_vec(), bytes, "[wrong bytes]: {}", given_msg);
        }
    }
}
