use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

/// Runtime trust tier for devices, packages, and communication endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    #[default]
    Untrusted,
    Restricted,
    Trusted,
    Certified,
}

impl TrustLevel {
    pub fn all() -> &'static [TrustLevel] {
        &[
            Self::Untrusted,
            Self::Restricted,
            Self::Trusted,
            Self::Certified,
        ]
    }

    pub fn rank(self) -> u8 {
        match self {
            Self::Untrusted => 0,
            Self::Restricted => 1,
            Self::Trusted => 2,
            Self::Certified => 3,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Untrusted => "untrusted",
            Self::Restricted => "restricted",
            Self::Trusted => "trusted",
            Self::Certified => "certified",
        }
    }

    pub fn satisfies(self, required: TrustLevel) -> bool {
        self.rank().cmp(&required.rank()) != Ordering::Less
    }
}

impl FromStr for TrustLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "untrusted" => Ok(Self::Untrusted),
            "restricted" => Ok(Self::Restricted),
            "trusted" => Ok(Self::Trusted),
            "certified" => Ok(Self::Certified),
            other => Err(format!("unknown trust level '{other}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_ordering() {
        assert!(TrustLevel::Certified.satisfies(TrustLevel::Trusted));
        assert!(!TrustLevel::Restricted.satisfies(TrustLevel::Trusted));
    }
}
