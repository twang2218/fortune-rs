use anyhow::{Error, Result};
use rust_embed::Embed;

pub const EMBED_PREFIX: &str = "embed:";

#[derive(Embed)]
#[folder = "cookies/"]
#[exclude = "**/*.md"]
#[exclude = "**/.*"]
pub struct Embedded;

impl Embedded {
    pub fn exists(path: &str) -> bool {
        !Embedded::find(path).unwrap().is_empty()
    }

    pub fn read_to_string(path: &str) -> Result<String> {
        let path = Embedded::trim_prefix(path);
        let file =
            Embedded::get(path).ok_or(Error::msg(format!("{}{} not found", EMBED_PREFIX, path)))?;
        let content = std::str::from_utf8(file.data.as_ref())?;
        Ok(content.to_string())
    }

    pub fn find(path: &str) -> Result<Vec<String>> {
        let path = Embedded::trim_prefix(path);
        let matches = Embedded::iter()
            .map(|entry| entry.to_string())
            .filter(|entry| entry.starts_with(path))
            .collect();
        Ok(matches)
    }

    pub fn format_path(path: &str) -> String {
        if path.starts_with(EMBED_PREFIX) {
            path.to_string()
        } else {
            format!("{}{}", EMBED_PREFIX, path)
        }
    }

    fn trim_prefix(path: &str) -> &str {
        path.trim_start_matches(EMBED_PREFIX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded() {
        let path = "embed:zh";
        assert_eq!(Embedded::exists(path), true, "should found {}", path);
        let path = "embed:zh/lunyu";
        assert!(
            Embedded::read_to_string(path).unwrap().len() > 2000,
            "read_to_string({}) should read at least 2000 bytes",
            path
        );
        assert_eq!(
            vec!["zh/lunyu"],
            Embedded::find("embed:zh/lunyu").unwrap(),
            "find(embed:zh/lunyu) should return [zh/lunyu]"
        );
        assert!(
            Embedded::find("embed:").unwrap().len() > 1,
            "find(embed:) should return more than 1 entry"
        );

        assert_eq!(
            "embed:file1",
            Embedded::format_path("file1"),
            "format_path(file1) should return 'embed:file1'"
        );
        assert_eq!(
            "embed:file1",
            Embedded::format_path("embed:file1"),
            "format_path(embed:file1) should return the same value 'embed:file1'"
        );
        assert_eq!(
            "embed:path/to/file1",
            Embedded::format_path("path/to/file1"),
            "format_path(path/to/file1) should return 'embed:path/to/file1'"
        );

        assert_eq!(
            "file1",
            Embedded::trim_prefix("file1"),
            "trim_prefix(file1) should return 'file1'"
        );
        assert_eq!(
            "file1",
            Embedded::trim_prefix("embed:file1"),
            "trim_prefix(embed:file1) should return 'file1'"
        );
        assert_eq!(
            "path/to/file1",
            Embedded::trim_prefix("embed:path/to/file1"),
            "trim_prefix(embed:path/to/file1) should return 'path/to/file1'"
        );
    }
}
