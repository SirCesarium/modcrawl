pub(crate) mod bukkit;
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::registry::Registry;

    fn test_handler(mod_type: &str) {
        let mut registry = Registry::new();
        register_all(&mut registry);

        let handler = registry.handler_by_file(match mod_type {
            "Fabric" => "fabric.mod.json",
            "Quilt" => "quilt.mod.json",
            "Forge (mods.toml)" => "META-INF/mods.toml",
            "NeoForge" => "META-INF/neoforge.mods.toml",
            "Forge (mcmod.info)" => "mcmod.info",
            "Plugin (Bukkit/Spigot)" => "plugin.yml",
            "Plugin (Paper)" => "paper-plugin.yml",
            "Plugin (Velocity)" => "velocity-plugin.json",
            _ => panic!("unknown mod type: {mod_type}"),
        });
        assert!(handler.is_some(), "handler for {mod_type} not found");
        assert_eq!(handler.unwrap().mod_type().to_string(), mod_type);
    }

    #[test]
    fn test_fabric_handler() {
        test_handler("Fabric");
    }

    #[test]
    fn test_quilt_handler() {
        test_handler("Quilt");
    }

    #[test]
    fn registry_detect_dual_fabric_quilt() {
        use zipcrawl::ZipEntry;

        use crate::core::detect_mod_type::ModType;

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
        test_handler("Forge (mods.toml)");
    }

    #[test]
    fn test_neoforge_handler() {
        test_handler("NeoForge");
    }

    #[test]
    fn test_forge_legacy_handler() {
        test_handler("Forge (mcmod.info)");
    }

    #[test]
    fn test_bukkit_handler() {
        test_handler("Plugin (Bukkit/Spigot)");
    }

    #[test]
    fn test_paper_handler() {
        test_handler("Plugin (Paper)");
    }

    #[test]
    fn test_velocity_handler() {
        test_handler("Plugin (Velocity)");
    }
}
