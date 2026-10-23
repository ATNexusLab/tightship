use std::fmt;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::failure::Failure;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u64, u64, u64);

impl Version {
    fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.').map(|part| {
            let digits = Some(part).filter(|part| {
                !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
            })?;
            digits.parse().ok()
        });
        let version = Self(parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(version)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.0, self.1, self.2)
    }
}

pub struct Manifest {
    pub name: String,
    pub version: Version,
    pub repository: String,
}

impl Manifest {
    pub fn read(path: &Path) -> Result<Self, Failure> {
        let invalid = || Failure::ManifestInvalid(path.to_path_buf());
        let text = fs::read_to_string(path).map_err(|_| invalid())?;
        let value: Value =
            serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|_| invalid())?;
        let text_of = |name| value.get(name).and_then(Value::as_str);
        let name = text_of("name").filter(|name| !name.is_empty());
        let version = text_of("version").and_then(Version::parse);
        let repository = text_of("repository")
            .map(|repository| repository.trim_end_matches('/'))
            .filter(|repository| !repository.is_empty());
        match (name, version, repository) {
            (Some(name), Some(version), Some(repository)) => Ok(Self {
                name: name.to_owned(),
                version,
                repository: repository.to_owned(),
            }),
            _ => Err(invalid()),
        }
    }
}

pub struct Release {
    pub version: Version,
    pub tag: String,
    pub archive: String,
    pub sha256: String,
}

impl Release {
    pub fn parse(text: &str, url: &str, harness: &str) -> Result<Self, Failure> {
        let invalid = || Failure::UpdateReleaseInvalid(url.to_owned());
        let value: Value =
            serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|_| invalid())?;
        let version = value
            .get("version")
            .and_then(Value::as_str)
            .and_then(Version::parse)
            .ok_or_else(invalid)?;
        let tag = value
            .get("tag")
            .and_then(Value::as_str)
            .filter(|tag| is_plain_name(tag))
            .ok_or_else(invalid)?;
        let assets = value
            .get("assets")
            .and_then(Value::as_object)
            .ok_or_else(invalid)?;
        let asset = assets
            .get(harness)
            .ok_or_else(|| Failure::UpdateNoArchive {
                tag: tag.to_owned(),
                harness: harness.to_owned(),
            })?;
        let archive = asset
            .get("archive")
            .and_then(Value::as_str)
            .filter(|archive| is_plain_name(archive))
            .ok_or_else(invalid)?;
        let sha256 = asset
            .get("sha256")
            .and_then(Value::as_str)
            .filter(|sha256| {
                sha256.len() == 64 && sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
            .ok_or_else(invalid)?;
        Ok(Self {
            version,
            tag: tag.to_owned(),
            archive: archive.to_owned(),
            sha256: sha256.to_ascii_lowercase(),
        })
    }
}

fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}
