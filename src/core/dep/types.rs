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

        if let Some(max) = trimmed.strip_prefix('<') {
            return Self {
                min: None,
                max: Some(max.to_owned()),
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
