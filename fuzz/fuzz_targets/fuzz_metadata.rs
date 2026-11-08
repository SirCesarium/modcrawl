#![no_main]

use libfuzzer_sys::fuzz_target;
use modcrawl::{ForgeModFormat, ModType, PluginType, REGISTRY};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let types = [
            ModType::Fabric,
            ModType::Quilt,
            ModType::Forge(ForgeModFormat::ModsToml),
            ModType::Forge(ForgeModFormat::McmodInfo),
            ModType::NeoForge,
            ModType::Plugin(PluginType::Bukkit),
            ModType::Plugin(PluginType::Paper),
            ModType::Plugin(PluginType::Bungee),
            ModType::Plugin(PluginType::Velocity),
        ];
        for t in &types {
            let _ = REGISTRY.read_metadata(t, s);
            let _ = REGISTRY.extract_deps(t, s);
        }
    }
});
