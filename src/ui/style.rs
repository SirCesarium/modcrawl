//! The terminal's color vocabulary: one place that decides what hue a loader,
//! a dependency kind, and a muted label get. Every renderer asks this module
//! for a style instead of hard-coding ANSI values, so the whole CLI reads as
//! one coherent palette.

use anstyle::{AnsiColor, Color, Style};

use crate::core::dep::types::DepKind;
use crate::core::detect_mod_type::{ModType, PluginType};

/// Foreground color for a loader or plugin type.
pub(crate) fn loader_color(t: &ModType) -> Color {
    match t {
        ModType::Fabric => Color::Ansi(AnsiColor::Cyan),
        ModType::Forge(_) => Color::Ansi(AnsiColor::Yellow),
        ModType::NeoForge => Color::Ansi(AnsiColor::Red),
        ModType::Plugin(PluginType::Bukkit) => Color::Ansi(AnsiColor::Green),
        ModType::Plugin(PluginType::Paper) => Color::Ansi(AnsiColor::BrightGreen),
        ModType::Plugin(PluginType::Bungee) => Color::Ansi(AnsiColor::Blue),
        ModType::Plugin(PluginType::Velocity) => Color::Ansi(AnsiColor::Magenta),
        ModType::Unknown => Color::Ansi(AnsiColor::BrightBlack),
    }
}

/// Style for a loader name: bold and in the loader's color.
pub(crate) fn loader_style(t: &ModType) -> Style {
    Style::new().bold().fg_color(Some(loader_color(t)))
}

/// Foreground color for a dependency kind.
pub(crate) fn kind_color(kind: DepKind) -> Color {
    match kind {
        DepKind::Required => Color::Ansi(AnsiColor::Green),
        DepKind::Optional | DepKind::Recommended | DepKind::Suggested => {
            Color::Ansi(AnsiColor::Yellow)
        }
        DepKind::LoadBefore => Color::Ansi(AnsiColor::Cyan),
        DepKind::Incompatible | DepKind::Discouraged => Color::Ansi(AnsiColor::Red),
    }
}

/// Style for a dependency kind label in the deps table.
pub(crate) fn kind_style(kind: DepKind) -> Style {
    Style::new().fg_color(Some(kind_color(kind)))
}

/// Human label for a dependency kind — replaces the old cryptic `*`/`-` legend.
pub(crate) fn kind_label(kind: DepKind) -> &'static str {
    match kind {
        DepKind::Required => "required",
        DepKind::Optional => "optional",
        DepKind::Recommended => "recommended",
        DepKind::Suggested => "suggested",
        DepKind::LoadBefore => "load-before",
        DepKind::Incompatible => "incompatible",
        DepKind::Discouraged => "discouraged",
    }
}

/// The muted style used for table headers, field labels, and rules.
pub(crate) fn dim() -> Style {
    Style::new().dimmed()
}

/// Wrap `text` in `style`. A default (empty) style emits nothing, so callers
/// never branch on "should I color this?" — `anstream` strips whatever is
/// left when the output is piped.
pub(crate) fn paint(style: Style, text: &str) -> String {
    if style.is_plain() {
        text.to_string()
    } else {
        format!("{style}{text}{style:#}")
    }
}
