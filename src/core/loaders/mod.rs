pub(crate) mod bukkit;
pub(crate) mod fabric;
pub(crate) mod forge_legacy;
pub(crate) mod forge_modern;
pub(crate) mod neoforge;
pub(crate) mod paper;

use crate::core::registry::Registry;

/// Register all known mod/plugin handlers in priority order.
///
/// Order matters: the first matching detection rule wins
/// (Paper > Bukkit, `NeoForge` > Forge).
pub fn register_all(registry: &mut Registry) {
    registry.register(Box::new(paper::PaperHandler));
    registry.register(Box::new(bukkit::BukkitHandler));
    registry.register(Box::new(neoforge::NeoForgeHandler));
    registry.register(Box::new(forge_modern::ForgeModernHandler));
    registry.register(Box::new(forge_legacy::ForgeLegacyHandler));
    registry.register(Box::new(fabric::FabricHandler));
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
            "Forge (mods.toml)" => "META-INF/mods.toml",
            "NeoForge" => "META-INF/neoforge.mods.toml",
            "Forge (mcmod.info)" => "mcmod.info",
            "Plugin (Bukkit/Spigot)" => "plugin.yml",
            "Plugin (Paper)" => "paper-plugin.yml",
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
}
