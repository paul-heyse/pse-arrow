// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed package version requirements (ADR-0123 Outcome 7).
use pse_model::generated::{enums::ModelingVersionOperator, structures::VersionRequirement};

/// The exact requirement written `1.2.3` or `=1.2.3`; prereleases, build metadata and
/// ranges name no admitted operator (§6.1 admits exact requirements only).
pub fn exact_requirement(text: &str) -> Option<VersionRequirement> {
    let version = semver::Version::parse(text.strip_prefix('=').unwrap_or(text)).ok()?;
    if !version.pre.is_empty() || !version.build.is_empty() {
        return None;
    }
    Some(VersionRequirement {
        operator: ModelingVersionOperator::Exact,
        major: i64::try_from(version.major).ok()?,
        minor: i64::try_from(version.minor).ok()?,
        patch: i64::try_from(version.patch).ok()?,
    })
}

/// Whether an admitted package version meets `requirement`.
pub fn requirement_admits(requirement: &VersionRequirement, version: &str) -> bool {
    let Ok(version) = semver::Version::parse(version) else {
        return false;
    };
    match requirement.operator {
        ModelingVersionOperator::Exact => {
            version.pre.is_empty()
                && version.build.is_empty()
                && [version.major, version.minor, version.patch]
                    .iter()
                    .zip([requirement.major, requirement.minor, requirement.patch])
                    .all(|(have, want)| u64::try_from(want).is_ok_and(|want| *have == want))
        }
    }
}

/// The version a requirement names, `major.minor.patch`.
pub fn requirement_version(requirement: &VersionRequirement) -> String {
    format!(
        "{}.{}.{}",
        requirement.major, requirement.minor, requirement.patch
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_requirements_admit_exactly_their_version() {
        let requirement = exact_requirement("=1.2.3").unwrap();
        assert_eq!(exact_requirement("1.2.3").unwrap(), requirement);
        assert_eq!(requirement_version(&requirement), "1.2.3");
        assert!(requirement_admits(&requirement, "1.2.3"));
        for other in ["1.2.4", "1.2.3-rc.1", "1.2", "latest"] {
            assert!(!requirement_admits(&requirement, other), "{other}");
        }
        for refused in ["^1.2.3", ">=1.0.0", "1.2.3-rc.1", "1.2.3+build", "1.2"] {
            assert_eq!(exact_requirement(refused), None, "{refused}");
        }
    }
}
