#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::env::temp_dir;
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// An RAII temporary JAR file that is deleted when dropped.
pub struct TempJar {
    path: PathBuf,
}

impl TempJar {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = process::id();
        let path = temp_dir().join(format!("modcrawl_test_{pid}_{count}.jar"));
        fs::write(&path, bytes).expect("failed to write test jar");
        Self { path }
    }

    pub fn from_entries(entries: &[(&str, &[u8])]) -> Self {
        let bytes = build_jar(entries);
        Self::from_bytes(&bytes)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempJar {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Build an in-memory ZIP/JAR archive from entry name and byte contents.
pub fn build_jar(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

    for (name, contents) in entries {
        zip.start_file(*name, SimpleFileOptions::default())
            .expect("failed to start zip entry");
        zip.write_all(contents)
            .expect("failed to write zip entry content");
    }

    zip.finish().expect("failed to finalize zip").into_inner()
}

/// Fake Java class file header for bytecode inspections.
pub const FAKE_CLASS_BYTES: &[u8] = b"\xca\xfe\xba\xbe\x00\x00\x00\x34\x00\x05\x01\x00\x04Test";

/// A minimal valid class file: `com/example/DemoClass.class`, whose constant
/// pool holds the `Utf8` entries `Test`, `java/lang/Object`, and
/// `net/example/DemoClass`.
pub fn demo_class_bytes() -> Vec<u8> {
    fn utf8(text: &str) -> Vec<u8> {
        let mut entry = vec![0x01];
        entry.extend_from_slice(&u16::try_from(text.len()).expect("fits a u16").to_be_bytes());
        entry.extend_from_slice(text.as_bytes());
        entry
    }

    let mut bytes = vec![0xca, 0xfe, 0xba, 0xbe, 0x00, 0x00, 0x00, 0x34];

    bytes.extend_from_slice(&7u16.to_be_bytes()); // constant_pool_count
    bytes.extend(utf8("Test")); // #1
    bytes.extend([0x07, 0x00, 0x01]); // #2 Class #1
    bytes.extend(utf8("java/lang/Object")); // #3
    bytes.extend([0x07, 0x00, 0x03]); // #4 Class #3
    bytes.extend(utf8("net/example/DemoClass")); // #5
    bytes.extend([0x07, 0x00, 0x05]); // #6 Class #5

    bytes.extend_from_slice(&0x0021u16.to_be_bytes()); // access_flags
    bytes.extend_from_slice(&2u16.to_be_bytes()); // this_class
    bytes.extend_from_slice(&4u16.to_be_bytes()); // super_class
    bytes.extend_from_slice(&0u16.to_be_bytes()); // interfaces
    bytes.extend_from_slice(&0u16.to_be_bytes()); // fields
    bytes.extend_from_slice(&0u16.to_be_bytes()); // methods
    bytes.extend_from_slice(&0u16.to_be_bytes()); // attributes

    bytes
}

/// A JAR holding [`demo_class_bytes`] as its only class file.
pub fn demo_class_jar() -> TempJar {
    let bytes = demo_class_bytes();

    TempJar::from_entries(&[("com/example/DemoClass.class", bytes.as_slice())])
}

/// Create a minimal Fabric mod fixture.
pub fn fabric_jar(id: &str, version: &str) -> TempJar {
    let manifest = format!(
        r#"{{
  "schemaVersion": 1,
  "id": "{id}",
  "version": "{version}",
  "name": "Fabric Test Mod",
  "description": "A test Fabric mod",
  "authors": ["Tester"],
  "contact": {{ "sources": "https://github.com/example/fabric-test" }},
  "license": "MIT",
  "environment": "*",
  "depends": {{
    "fabricloader": ">=0.14.0",
    "minecraft": "~1.20.1"
  }}
}}"#
    );
    TempJar::from_entries(&[
        ("fabric.mod.json", manifest.as_bytes()),
        ("com/example/TestClass.class", FAKE_CLASS_BYTES),
    ])
}

/// Create a minimal Quilt mod fixture.
pub fn quilt_jar(id: &str, version: &str) -> TempJar {
    let manifest = format!(
        r#"{{
  "schema_version": 1,
  "quilt_loader": {{
    "group": "org.example",
    "id": "{id}",
    "version": "{version}",
    "metadata": {{
      "name": "Quilt Test Mod",
      "description": "A test Quilt mod",
      "contributors": {{ "Tester": "Owner" }},
      "license": "Apache-2.0"
    }},
    "depends": [
      {{ "id": "quilt_loader", "versions": ">=0.20.0" }}
    ]
  }}
}}"#
    );
    TempJar::from_entries(&[("quilt.mod.json", manifest.as_bytes())])
}

/// Create a minimal Forge modern (`mods.toml`) fixture.
pub fn forge_modern_jar(id: &str, version: &str) -> TempJar {
    let manifest = format!(
        r#"modLoader="javafml"
loaderVersion="[47,)"
license="MIT"

[[mods]]
modId="{id}"
version="{version}"
displayName="Forge Test Mod"
description="A test Forge mod"
authors="Tester"

[[dependencies.{id}]]
modId="forge"
mandatory=true
versionRange="[47.1.0,)"
ordering="NONE"
side="BOTH"
"#
    );
    TempJar::from_entries(&[("META-INF/mods.toml", manifest.as_bytes())])
}

/// Create a minimal `NeoForge` modern fixture.
pub fn neoforge_jar(id: &str, version: &str) -> TempJar {
    let manifest = format!(
        r#"modLoader="javafml"
loaderVersion="[20.4,)"
license="MIT"

[[mods]]
modId="{id}"
version="{version}"
displayName="NeoForge Test Mod"
description="A test NeoForge mod"
authors="Tester"

[[dependencies.{id}]]
modId="neoforge"
mandatory=true
versionRange="[20.4.0,)"
ordering="NONE"
side="BOTH"
"#
    );
    TempJar::from_entries(&[("META-INF/neoforge.mods.toml", manifest.as_bytes())])
}

/// Create a minimal Forge legacy (`mcmod.info`) fixture.
pub fn forge_legacy_jar(id: &str, version: &str) -> TempJar {
    let manifest = format!(
        r#"[
  {{
    "modid": "{id}",
    "name": "Legacy Test Mod",
    "description": "A test Forge Legacy mod",
    "version": "{version}",
    "mcversion": "1.12.2",
    "authorList": ["Tester"]
  }}
]"#
    );
    TempJar::from_entries(&[("mcmod.info", manifest.as_bytes())])
}

/// Create a minimal Bukkit/Spigot plugin fixture.
pub fn bukkit_jar(name: &str, version: &str) -> TempJar {
    let manifest = format!(
        r"name: {name}
version: {version}
main: org.example.BukkitTestPlugin
description: A test Bukkit plugin
author: Tester
depend: [WorldEdit]
softdepend: [Vault]
"
    );
    TempJar::from_entries(&[("plugin.yml", manifest.as_bytes())])
}

/// Create a minimal Paper plugin fixture.
pub fn paper_jar(name: &str, version: &str) -> TempJar {
    let manifest = format!(
        r"name: {name}
version: {version}
main: org.example.PaperTestPlugin
description: A test Paper plugin
author: Tester
api-version: '1.20'
dependencies:
  server:
    WorldEdit:
      load: BEFORE
      required: true
"
    );
    TempJar::from_entries(&[("paper-plugin.yml", manifest.as_bytes())])
}

/// Create a minimal Velocity plugin fixture.
pub fn velocity_jar(id: &str, version: &str) -> TempJar {
    let manifest = format!(
        r#"{{
  "id": "{id}",
  "name": "Velocity Test Plugin",
  "version": "{version}",
  "description": "A test Velocity plugin",
  "authors": ["Tester"],
  "main": "org.example.VelocityTestPlugin"
}}"#
    );
    TempJar::from_entries(&[("velocity-plugin.json", manifest.as_bytes())])
}

/// Create a minimal `BungeeCord` plugin fixture.
pub fn bungee_jar(name: &str, version: &str) -> TempJar {
    let manifest = format!(
        r"name: {name}
version: {version}
main: org.example.BungeeTestPlugin
author: Tester
description: A test Bungee plugin
"
    );
    TempJar::from_entries(&[("bungee.yml", manifest.as_bytes())])
}

/// Create a Jar-in-Jar fixture: outer JAR contains an inner JAR.
pub fn jar_in_jar_fixture() -> TempJar {
    let inner_bytes = build_jar(&[(
        "fabric.mod.json",
        br#"{
  "schemaVersion": 1,
  "id": "inner_mod",
  "version": "0.5.0",
  "name": "Inner Mod"
}"#,
    )]);

    let outer_manifest = r#"{
  "schemaVersion": 1,
  "id": "outer_mod",
  "version": "1.0.0",
  "name": "Outer Mod",
  "depends": {
    "inner_mod": ">=0.5.0",
    "external_mod": ">=1.0.0"
  },
  "jars": [
    { "file": "META-INF/jars/inner_mod-0.5.0.jar" }
  ]
}"#;

    TempJar::from_entries(&[
        ("fabric.mod.json", outer_manifest.as_bytes()),
        ("META-INF/jars/inner_mod-0.5.0.jar", &inner_bytes),
    ])
}

/// Create an embedded shell jar fixture (outer has no mod metadata, inner does).
pub fn embedded_shell_fixture() -> TempJar {
    let inner_bytes = build_jar(&[(
        "fabric.mod.json",
        br#"{
  "schemaVersion": 1,
  "id": "embedded_mod",
  "version": "1.2.3",
  "name": "Embedded Mod",
  "license": "MIT",
  "depends": {
    "minecraft": "1.20.1"
  }
}"#,
    )]);

    TempJar::from_entries(&[
        ("META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"),
        ("META-INF/jarjar/embedded_mod-1.2.3.jar", &inner_bytes),
    ])
}
