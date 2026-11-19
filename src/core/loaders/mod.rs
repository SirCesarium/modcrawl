//! Handlers for each supported mod and plugin loader.

pub(crate) mod bukkit;
pub(crate) mod bungee;
pub(crate) mod fabric;
pub(crate) mod forge_legacy;
pub(crate) mod forge_modern;
pub(crate) mod neoforge;
pub(crate) mod paper;
pub(crate) mod quilt;
pub(crate) mod velocity;

use crate::core::registry::Registry;

/// Register all known mod/plugin handlers in priority order.
///
/// Order matters: the first matching detection rule wins
/// (Paper > Bukkit, `NeoForge` > Forge, Fabric > Quilt for jars that ship
/// both `fabric.mod.json` and `quilt.mod.json`).
pub fn register_all(registry: &mut Registry) {
    registry.register(Box::new(paper::PaperHandler));
    registry.register(Box::new(bukkit::BukkitHandler));
    registry.register(Box::new(neoforge::NeoForgeHandler));
    registry.register(Box::new(forge_modern::ForgeModernHandler));
    registry.register(Box::new(forge_legacy::ForgeLegacyHandler));
    registry.register(Box::new(fabric::FabricHandler));
    registry.register(Box::new(quilt::QuiltHandler));
    registry.register(Box::new(velocity::VelocityHandler));
    registry.register(Box::new(bungee::BungeeHandler));
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::detect_mod_type::{ForgeModFormat, ModType, PluginType};
    use crate::core::registry::Registry;

    fn test_handler(expect: &ModType, metadata_file: &str) {
        let mut registry = Registry::new();
        register_all(&mut registry);

        let handler = registry.handler_by_file(metadata_file);
        assert!(handler.is_some(), "handler for {metadata_file} not found");
        assert_eq!(&handler.unwrap().mod_type(), expect);
    }

    #[test]
    fn test_fabric_handler() {
        test_handler(&ModType::Fabric, "fabric.mod.json");
    }

    #[test]
    fn test_quilt_handler() {
        test_handler(&ModType::Quilt, "quilt.mod.json");
    }

    #[test]
    fn registry_detect_dual_fabric_quilt() {
        use zipcrawl::ZipEntry;

        let mut registry = Registry::new();
        register_all(&mut registry);

        let entries = vec![
            ZipEntry {
                name: "fabric.mod.json".to_owned(),
                is_dir: false,
                size: 0,
                crc: 0,
            },
            ZipEntry {
                name: "quilt.mod.json".to_owned(),
                is_dir: false,
                size: 0,
                crc: 0,
            },
        ];
        assert_eq!(registry.detect(&entries), ModType::Fabric);
    }

    #[test]
    fn test_forge_modern_handler() {
        test_handler(
            &ModType::Forge(ForgeModFormat::ModsToml),
            "META-INF/mods.toml",
        );
    }

    #[test]
    fn test_neoforge_handler() {
        test_handler(&ModType::NeoForge, "META-INF/neoforge.mods.toml");
    }

    #[test]
    fn test_forge_legacy_handler() {
        test_handler(&ModType::Forge(ForgeModFormat::McmodInfo), "mcmod.info");
    }

    #[test]
    fn test_bukkit_handler() {
        test_handler(&ModType::Plugin(PluginType::Bukkit), "plugin.yml");
    }

    #[test]
    fn test_paper_handler() {
        test_handler(&ModType::Plugin(PluginType::Paper), "paper-plugin.yml");
    }

    #[test]
    fn test_velocity_handler() {
        test_handler(
            &ModType::Plugin(PluginType::Velocity),
            "velocity-plugin.json",
        );
    }

    #[test]
    fn test_bungee_handler() {
        test_handler(&ModType::Plugin(PluginType::Bungee), "bungee.yml");
    }
}
