use std::{cmp::Ordering, fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub enum Version {
    Semver(semver::Version),
    PartialSemver(semver::Version, String),
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.version().eq(other.version())
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version().cmp(other.version())
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Version::Semver(v) => &v.to_string(),
            Version::PartialSemver(_, s) => s,
        };

        write!(f, "{}", v)
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match semver::Version::from_str(s) {
            Ok(v) => Ok(Self::Semver(v)),
            Err(_) => {
                let parts: Vec<&str> = s.split('.').collect();
                if parts.len() >= 2 {
                    if let (Ok(major), Ok(minor)) =
                        (parts[0].parse::<u64>(), parts[1].parse::<u64>())
                    {
                        let partial_semver = semver::Version::new(major, minor, 0);
                        return Ok(Self::PartialSemver(partial_semver, s.to_string()));
                    }
                }
                Err(anyhow::Error::msg(format!("Invalid version format: {}", s)))
            }
        }
    }
}

impl Version {
    #[inline]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self::Semver(semver::Version::new(major, minor, patch))
    }

    #[inline]
    pub fn version(&self) -> &semver::Version {
        match self {
            Version::Semver(v) => v,
            Version::PartialSemver(v, _) => v,
        }
    }

    #[inline]
    pub fn version_opt(&self) -> Option<&semver::Version> {
        match self {
            Version::Semver(v) => Some(v),
            Version::PartialSemver(..) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::version::Version;

    #[test]
    fn test() {
        assert_eq!(
            Version::from_str("24.04").unwrap(),
            Version::PartialSemver(semver::Version::new(24, 4, 0), "24.04".into())
        );

        assert_eq!(
            Version::from_str("2024.4").unwrap(),
            Version::PartialSemver(semver::Version::new(2024, 4, 0), "2024.4".into())
        );

        assert_eq!(
            Version::from_str("03.04").unwrap().version(),
            &semver::Version::new(3, 4, 0)
        );

        assert_eq!(
            Version::from_str("24.00").unwrap().version(),
            &semver::Version::new(24, 0, 0)
        );
    }
}
