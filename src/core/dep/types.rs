use std::fmt;

/// The kind/semantics of a dependency entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum DepKind {
    Required,
    Optional,
    Recommended,
    Suggested,
    Incompatible,
    Discouraged,
    LoadBefore,
}

impl DepKind {
    #[must_use]
    pub fn priority(self) -> u8 {
        match self {
            Self::Required => 5,
            Self::LoadBefore => 4,
            Self::Optional => 3,
            Self::Recommended => 2,
            Self::Suggested => 1,
            Self::Incompatible | Self::Discouraged => 0,
        }
    }

    #[must_use]
    pub fn is_excluded(self) -> bool {
        matches!(self, Self::Incompatible | Self::Discouraged)
    }

    fn marker(self) -> &'static str {
        match self {
            Self::Required => "*",
            Self::Optional | Self::Recommended | Self::Suggested => "-",
            Self::LoadBefore => "<",
            Self::Incompatible | Self::Discouraged => "!",
        }
    }
}

/// A parsed version range with structured bounds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct VersionRange {
    pub min: Option<String>,
    pub max: Option<String>,
    pub min_inclusive: bool,
    pub max_inclusive: bool,
    pub raw: Option<String>,
}

impl VersionRange {
    #[must_use]
    pub fn parse(raw: Option<String>) -> Self {
        let owned = raw.as_ref().filter(|s| !s.is_empty() && *s != "*").cloned();
        match owned {
            None => Self {
                min: None,
                max: None,
                min_inclusive: true,
                max_inclusive: false,
                raw,
            },
            Some(s) if s.starts_with('[') || s.starts_with('(') => Self::parse_maven(&s, raw),
            Some(s) => Self::parse_simple(&s, raw),
        }
    }

    fn parse_maven(s: &str, raw: Option<String>) -> Self {
        let min_inclusive = s.starts_with('[');
        let max_inclusive = s.ends_with(']');

        let inner = s
            .trim_start_matches('[')
            .trim_start_matches('(')
            .trim_end_matches(']')
            .trim_end_matches(')');

        if let Some((l, r)) = inner.split_once(',') {
            Self {
                min: Some(l.trim()).filter(|s| !s.is_empty()).map(String::from),
                max: Some(r.trim()).filter(|s| !s.is_empty()).map(String::from),
                min_inclusive,
                max_inclusive,
                raw,
            }
        } else {
            Self {
                min: Some(inner.to_owned()),
                max: Some(inner.to_owned()),
                min_inclusive: true,
                max_inclusive: true,
                raw,
            }
        }
    }

    fn parse_simple(s: &str, raw: Option<String>) -> Self {
        let trimmed = s.trim();

        // Single < must be checked before split_once('<') so `<2.0`
        // doesn't get caught by the `>=a<b` branch as `("", "2.0")`.
        if let Some(max) = trimmed.strip_prefix('<') {
            return Self {
                min: None,
                max: Some(max.to_owned()),
                min_inclusive: true,
                max_inclusive: false,
                raw,
            };
        }

        if let Some((ge, lt)) = trimmed.split_once('<') {
            return Self {
                min: Some(ge.trim().strip_prefix(">=").unwrap_or(ge.trim()).to_owned()),
                max: Some(lt.trim().to_owned()),
                min_inclusive: true,
                max_inclusive: false,
                raw,
            };
        }

        if let Some(min) = trimmed.strip_prefix(">=") {
            return Self {
                min: Some(min.to_owned()),
                max: None,
                min_inclusive: true,
                max_inclusive: false,
                raw,
            };
        }

        Self {
            min: Some(trimmed.to_owned()),
            max: Some(trimmed.to_owned()),
            min_inclusive: true,
            max_inclusive: true,
            raw,
        }
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(raw) = &self.raw {
            return write!(f, "{raw}");
        }
        write!(f, "*")
    }
}

/// A single dependency entry extracted from mod metadata.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DepEntry {
    pub name: String,
    pub kind: DepKind,
    pub version_range: VersionRange,
}

impl DepEntry {
    #[must_use]
    pub fn new(name: impl Into<String>, kind: DepKind, version_range: VersionRange) -> Self {
        Self {
            name: name.into(),
            kind,
            version_range,
        }
    }
}

/// An embedded JAR found inside the main archive.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct JarInJar {
    pub path: String,
}

/// Full dependency analysis report for a single JAR.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DepReport {
    pub dependencies: Vec<DepEntry>,
    pub jar_in_jar: Vec<JarInJar>,
}

impl fmt::Display for DepEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = self.kind.marker();
        write!(f, "  {marker} {}", self.name)?;
        if self.version_range.raw.is_some()
            || self.version_range.min.is_some()
            || self.version_range.max.is_some()
        {
            write!(f, " ({})", self.version_range)?;
        }
        Ok(())
    }
}

impl fmt::Display for JarInJar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  / {}", self.path)
    }
}

impl fmt::Display for DepReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dependencies.is_empty() && self.jar_in_jar.is_empty() {
            return Ok(());
        }
        let mut has_optional = false;
        let mut has_loadbefore = false;
        for dep in &self.dependencies {
            match dep.kind {
                DepKind::Optional | DepKind::Recommended | DepKind::Suggested => {
                    has_optional = true;
                }
                DepKind::LoadBefore => {
                    has_loadbefore = true;
                }
                _ => {}
            }
        }
        writeln!(f, "* required")?;
        if has_optional {
            writeln!(f, "- optional/recommended/suggested")?;
        }
        if has_loadbefore {
            writeln!(f, "< load before")?;
        }
        if !self.jar_in_jar.is_empty() {
            writeln!(f, "/ jar-in-jar")?;
        }
        writeln!(f, "------")?;
        writeln!(f)?;

        if !self.dependencies.is_empty() {
            writeln!(f, "Dependencies:")?;
            for dep in &self.dependencies {
                writeln!(f, "{dep}")?;
            }
        }
        if !self.jar_in_jar.is_empty() {
            if !self.dependencies.is_empty() {
                writeln!(f)?;
            }
            writeln!(f, "Jar-in-jar:")?;
            for jij in &self.jar_in_jar {
                writeln!(f, "{jij}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn dep_kind_priority() {
        assert_eq!(DepKind::Required.priority(), 5);
        assert_eq!(DepKind::LoadBefore.priority(), 4);
        assert_eq!(DepKind::Optional.priority(), 3);
        assert_eq!(DepKind::Recommended.priority(), 2);
        assert_eq!(DepKind::Suggested.priority(), 1);
        assert_eq!(DepKind::Incompatible.priority(), 0);
        assert_eq!(DepKind::Discouraged.priority(), 0);
    }

    #[test]
    fn dep_kind_is_excluded() {
        assert!(!DepKind::Required.is_excluded());
        assert!(!DepKind::LoadBefore.is_excluded());
        assert!(!DepKind::Optional.is_excluded());
        assert!(!DepKind::Recommended.is_excluded());
        assert!(!DepKind::Suggested.is_excluded());
        assert!(DepKind::Incompatible.is_excluded());
        assert!(DepKind::Discouraged.is_excluded());
    }

    #[test]
    fn dep_kind_marker() {
        assert_eq!(DepKind::Required.marker(), "*");
        assert_eq!(DepKind::Optional.marker(), "-");
        assert_eq!(DepKind::Recommended.marker(), "-");
        assert_eq!(DepKind::Suggested.marker(), "-");
        assert_eq!(DepKind::LoadBefore.marker(), "<");
        assert_eq!(DepKind::Incompatible.marker(), "!");
        assert_eq!(DepKind::Discouraged.marker(), "!");
    }

    #[test]
    fn version_range_none() {
        let v = VersionRange::parse(None);
        assert_eq!(v.min, None);
        assert_eq!(v.max, None);
        assert!(v.min_inclusive);
        assert!(!v.max_inclusive);
        assert_eq!(v.raw, None);
    }

    #[test]
    fn version_range_wildcard() {
        let v = VersionRange::parse(Some("*".to_owned()));
        assert_eq!(v.min, None);
        assert_eq!(v.max, None);
        assert_eq!(v.raw, Some("*".to_owned()));
    }

    #[test]
    fn version_range_empty() {
        let v = VersionRange::parse(Some(String::new()));
        assert_eq!(v.min, None);
        assert_eq!(v.max, None);
    }

    #[test]
    fn version_range_maven_inclusive() {
        let v = VersionRange::parse(Some("[1.0,2.0]".to_owned()));
        assert_eq!(v.min, Some("1.0".to_owned()));
        assert_eq!(v.max, Some("2.0".to_owned()));
        assert!(v.min_inclusive);
        assert!(v.max_inclusive);
    }

    #[test]
    fn version_range_maven_exclusive_min() {
        let v = VersionRange::parse(Some("(1.0,2.0]".to_owned()));
        assert_eq!(v.min, Some("1.0".to_owned()));
        assert_eq!(v.max, Some("2.0".to_owned()));
        assert!(!v.min_inclusive);
        assert!(v.max_inclusive);
    }

    #[test]
    fn version_range_maven_unbounded_max() {
        let v = VersionRange::parse(Some("[1.0,)".to_owned()));
        assert_eq!(v.min, Some("1.0".to_owned()));
        assert_eq!(v.max, None);
        assert!(v.min_inclusive);
        assert!(!v.max_inclusive);
    }

    #[test]
    fn version_range_maven_unbounded_min() {
        let v = VersionRange::parse(Some("(,2.0]".to_owned()));
        assert_eq!(v.min, None);
        assert_eq!(v.max, Some("2.0".to_owned()));
        assert!(!v.min_inclusive);
        assert!(v.max_inclusive);
    }

    #[test]
    fn version_range_maven_exact() {
        let v = VersionRange::parse(Some("[1.0]".to_owned()));
        assert_eq!(v.min, Some("1.0".to_owned()));
        assert_eq!(v.max, Some("1.0".to_owned()));
        assert!(v.min_inclusive);
        assert!(v.max_inclusive);
    }

    #[test]
    fn version_range_simple_combined() {
        let v = VersionRange::parse(Some(">=1.0<2.0".to_owned()));
        assert_eq!(v.min, Some("1.0".to_owned()));
        assert_eq!(v.max, Some("2.0".to_owned()));
        assert!(v.min_inclusive);
        assert!(!v.max_inclusive);
    }

    #[test]
    fn version_range_simple_min_only() {
        let v = VersionRange::parse(Some(">=1.0".to_owned()));
        assert_eq!(v.min, Some("1.0".to_owned()));
        assert_eq!(v.max, None);
        assert!(v.min_inclusive);
        assert!(!v.max_inclusive);
    }

    #[test]
    fn version_range_simple_max_only() {
        let v = VersionRange::parse(Some("<2.0".to_owned()));
        assert_eq!(v.min, None);
        assert_eq!(v.max, Some("2.0".to_owned()));
        assert!(v.min_inclusive);
        assert!(!v.max_inclusive);
    }

    #[test]
    fn version_range_simple_exact() {
        let v = VersionRange::parse(Some("1.0".to_owned()));
        assert_eq!(v.min, Some("1.0".to_owned()));
        assert_eq!(v.max, Some("1.0".to_owned()));
        assert!(v.min_inclusive);
        assert!(v.max_inclusive);
    }

    #[test]
    fn version_range_display_falls_back_to_raw() {
        let v = VersionRange::parse(Some("[1.0,2.0]".to_owned()));
        assert_eq!(v.to_string(), "[1.0,2.0]");
    }

    #[test]
    fn version_range_display_wildcard_when_no_raw() {
        let v = VersionRange {
            min: Some("1.0".to_owned()),
            max: Some("2.0".to_owned()),
            min_inclusive: true,
            max_inclusive: true,
            raw: None,
        };
        assert_eq!(v.to_string(), "*");
    }

    #[test]
    fn dep_entry_new() {
        let vr = VersionRange::parse(Some(">=1.0".to_owned()));
        let e = DepEntry::new("test-mod", DepKind::Required, vr);
        assert_eq!(e.name, "test-mod");
        assert_eq!(e.kind, DepKind::Required);
        assert_eq!(e.version_range.min, Some("1.0".to_owned()));
    }

    #[test]
    fn dep_entry_display_with_version() {
        let e = DepEntry::new(
            "foo",
            DepKind::Required,
            VersionRange::parse(Some("1.0".to_owned())),
        );
        let s = e.to_string();
        assert!(s.contains('*'));
        assert!(s.contains("foo"));
        assert!(s.contains("1.0"));
    }

    #[test]
    fn dep_entry_display_without_version() {
        let e = DepEntry::new("bar", DepKind::Optional, VersionRange::parse(None));
        let s = e.to_string();
        assert!(s.contains('-'));
        assert!(s.contains("bar"));
        assert!(!s.contains('('));
    }

    #[test]
    fn jar_in_jar_display() {
        let j = JarInJar {
            path: "libs/foo.jar".to_owned(),
        };
        assert_eq!(j.to_string(), "  / libs/foo.jar");
    }

    #[test]
    fn dep_report_empty() {
        let r = DepReport {
            dependencies: vec![],
            jar_in_jar: vec![],
        };
        assert_eq!(r.to_string(), "");
    }

    #[test]
    fn dep_report_only_required() {
        let r = DepReport {
            dependencies: vec![DepEntry::new(
                "a",
                DepKind::Required,
                VersionRange::parse(None),
            )],
            jar_in_jar: vec![],
        };
        let s = r.to_string();
        assert!(s.contains("* required"));
        assert!(!s.contains("optional"));
        assert!(!s.contains("load before"));
        assert!(!s.contains("jar-in-jar"));
        assert!(s.contains("Dependencies:"));
    }

    #[test]
    fn dep_report_with_optional() {
        let r = DepReport {
            dependencies: vec![
                DepEntry::new("a", DepKind::Required, VersionRange::parse(None)),
                DepEntry::new("b", DepKind::Optional, VersionRange::parse(None)),
            ],
            jar_in_jar: vec![],
        };
        let s = r.to_string();
        assert!(s.contains("* required"));
        assert!(s.contains("- optional/recommended/suggested"));
        assert!(!s.contains("load before"));
    }

    #[test]
    fn dep_report_with_loadbefore() {
        let r = DepReport {
            dependencies: vec![
                DepEntry::new("a", DepKind::Required, VersionRange::parse(None)),
                DepEntry::new("b", DepKind::LoadBefore, VersionRange::parse(None)),
            ],
            jar_in_jar: vec![],
        };
        let s = r.to_string();
        assert!(s.contains("< load before"));
    }

    #[test]
    fn dep_report_with_jar_in_jar() {
        let r = DepReport {
            dependencies: vec![DepEntry::new(
                "a",
                DepKind::Required,
                VersionRange::parse(None),
            )],
            jar_in_jar: vec![JarInJar {
                path: "libs/foo.jar".to_owned(),
            }],
        };
        let s = r.to_string();
        assert!(s.contains("/ jar-in-jar"));
        assert!(s.contains("Jar-in-jar:"));
    }

    #[test]
    fn dep_report_legend_only_required() {
        let r = DepReport {
            dependencies: vec![DepEntry::new(
                "x",
                DepKind::Required,
                VersionRange::parse(None),
            )],
            jar_in_jar: vec![],
        };
        let s = r.to_string();
        assert!(!s.contains("- optional"));
        assert!(!s.contains("< load"));
        assert!(!s.contains("/ jar-in-jar"));
    }
}
