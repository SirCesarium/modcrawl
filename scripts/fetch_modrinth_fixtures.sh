#!/usr/bin/env bash
# Download curated, open-source mod fixtures from Modrinth CDN.
# Zero binary JARs are committed into the Git repository.

set -euo pipefail

DEST_DIR=".smoke_fixtures"
INCLUDE_HEAVY="${INCLUDE_HEAVY:-0}"

for arg in "$@"; do
  case "$arg" in
    --include-heavy)
      INCLUDE_HEAVY=1
      ;;
    --dest=*)
      DEST_DIR="${arg#*=}"
      ;;
  esac
done

mkdir -p "$DEST_DIR"

download_mod() {
  local filename="$1"
  local url="$2"
  local expected_sha1="$3"
  local filepath="$DEST_DIR/$filename"

  if [ -f "$filepath" ]; then
    local current_sha1
    current_sha1=$(sha1sum "$filepath" | awk '{print $1}')
    if [ "$current_sha1" = "$expected_sha1" ]; then
      echo "[SKIP] $filename (already cached, SHA-1 verified)"
      return 0
    fi
  fi

  echo "[FETCH] $filename from Modrinth CDN..."
  curl -sSL "$url" -o "$filepath.tmp"
  local downloaded_sha1
  downloaded_sha1=$(sha1sum "$filepath.tmp" | awk '{print $1}')

  if [ "$downloaded_sha1" != "$expected_sha1" ]; then
    echo "[ERROR] Checksum mismatch for $filename!" >&2
    echo "  Expected: $expected_sha1" >&2
    echo "  Got:      $downloaded_sha1" >&2
    rm -f "$filepath.tmp"
    return 1
  fi

  mv "$filepath.tmp" "$filepath"
  echo "[OK] $filename ($downloaded_sha1)"
}

# 1. Fabric: AppleSkin (Unlicense) ~1 MB
download_mod \
  "appleskin-fabric-mc1.20.1-2.5.2.jar" \
  "https://cdn.modrinth.com/data/EsAfCjCV/versions/N5XeV21r/appleskin-fabric-mc1.20.1-2.5.2.jar" \
  "f859bb13130788ae2dc2e76a9c80c89e4bbbef5f"

# 2. Quilt: Quilt Loading Screen (MIT) ~417 KB
download_mod \
  "quilt_loading_screen-6.1.0+1.20.1.jar" \
  "https://cdn.modrinth.com/data/VPU6VYVP/versions/CxbdfZp6/quilt_loading_screen-6.1.0%2B1.20.1.jar" \
  "95ad28ba01e197aa24a1510eb947653eae56109e"

# 3. NeoForge: FerriteCore (MIT) ~72 KB
download_mod \
  "ferritecore-9.0.0-neoforge.jar" \
  "https://cdn.modrinth.com/data/uXXizFIs/versions/LtVvw4uS/ferritecore-9.0.0-neoforge.jar" \
  "b8bfb14ba6ce7068aae08c3a132ca40bda6bd143"

# 4. Forge Modern: FerriteCore (MIT) ~123 KB
download_mod \
  "ferritecore-6.0.1-forge.jar" \
  "https://cdn.modrinth.com/data/uXXizFIs/versions/DG5Fn9Sz/ferritecore-6.0.1-forge.jar" \
  "417fb6ce8f52abf40bd9d0390371790f9576f8ba"

# 5. Forge Legacy (1.12.2): AppleSkin (Unlicense) ~34 KB
download_mod \
  "AppleSkin-mc1.12-1.0.14.jar" \
  "https://cdn.modrinth.com/data/EsAfCjCV/versions/Tsz4BT2X/AppleSkin-mc1.12-1.0.14.jar" \
  "d798af79de48be0f00c772943e5ec5a2fb5659fa"

# 6. Bukkit/Spigot: Chunky (GPL-3.0) ~400 KB
download_mod \
  "Chunky-Bukkit-1.5.66.jar" \
  "https://cdn.modrinth.com/data/fALzjamp/versions/selGpja4/Chunky-Bukkit-1.5.66.jar" \
  "175b6b19a508437ea6f5c84594f7652300d1298e"

# 7. Velocity: LuckPerms Velocity (MIT) ~1.5 MB
download_mod \
  "LuckPerms-Velocity-5.5.71.jar" \
  "https://cdn.modrinth.com/data/Vebnzrzj/versions/tamnmXad/LuckPerms-Velocity-5.5.71.jar" \
  "6e797ac9fc72b74870b6744a97ea1c2337cbb68a"

# 8. BungeeCord: BungeeViaProxy ~34 KB
download_mod \
  "bungeeviaproxy-1.0-SNAPSHOT.jar" \
  "https://cdn.modrinth.com/data/MIr3w1uG/versions/E1zUdg1y/bungeeviaproxy-1.0-SNAPSHOT.jar" \
  "3921707b6f945942196bfe69c0597a35c7ac6c68"

# 9. Forge: JEI (Just Enough Items - MIT) ~4.8 MB
download_mod \
  "jei-1.20.1-forge-15.74.0.234.jar" \
  "https://cdn.modrinth.com/data/u6dRKJwZ/versions/6E4ps2ER/jei-1.20.1-forge-15.74.0.234.jar" \
  "1ae408badfe8b57417f465a06fc4d03215dba993"

# 10. Fabric: Sodium (PolyForm Shield 1.0) ~970 KB
download_mod \
  "sodium-fabric-0.5.13+mc1.20.1.jar" \
  "https://cdn.modrinth.com/data/AANobbMI/versions/OihdIimA/sodium-fabric-0.5.13%2Bmc1.20.1.jar" \
  "bcdbf37d9494e405cba210d7a80ed58e756297b5"

# 11. Fabric/Quilt: Architectury API (LGPL-3.0) ~585 KB
download_mod \
  "architectury-9.2.14-fabric.jar" \
  "https://cdn.modrinth.com/data/lhGA9TYQ/versions/WbL7MStR/architectury-9.2.14-fabric.jar" \
  "4fa65e32dbf63a871a30fe8ec6fbff9a4b8d007d"

# Heavy fixtures (Create ~19 MB, Sable ~17 MB)
if [ "$INCLUDE_HEAVY" = "1" ]; then
  download_mod \
    "create-1.20.1-6.0.8.jar" \
    "https://cdn.modrinth.com/data/LNytGWDc/versions/8amzvn9x/create-1.20.1-6.0.8.jar" \
    "b13d912b9247a38d66d11598c121442585a1c1e9"

  download_mod \
    "sable-fabric-1.21.1-2.0.6.jar" \
    "https://cdn.modrinth.com/data/T9PomCSv/versions/OLG7vDD6/sable-fabric-1.21.1-2.0.6.jar" \
    "b2ce24e54cde01a326b3a4be635664182ba0de39"
fi

echo "All fixtures ready in $DEST_DIR."
