#!/usr/bin/env bash
# Verify that every `modcrawl_*` symbol declared in the C header is defined by
# every library handed to it.
#
# This is the check that would have caught the v0.5.2 packaging: the musl
# package shipped `modcrawl.h` declaring four functions the library did not
# export, which only shows up as a link error for whoever uses it.
#
# Usage: check-ffi-symbols.sh <header.h> <lib.so|lib.a|lib.dylib> [more libs...]
#
# Libraries that `nm` cannot inspect (e.g. MSVC .lib on a runner without nm)
# are reported and skipped rather than failed, so this runs on Linux and macOS.
set -euo pipefail

if [ "$#" -lt 2 ]; then
    echo "usage: $0 <header.h> <library> [more libraries...]" >&2
    exit 2
fi

header="$1"
shift

mapfile -t declared < <(
    grep -oE 'modcrawl_[a-z0-9_]+[[:space:]]*\(' "$header" \
        | grep -oE 'modcrawl_[a-z0-9_]+' \
        | sort -u
)

if [ "${#declared[@]}" -eq 0 ]; then
    echo "error: no modcrawl_* symbols declared in $header" >&2
    exit 1
fi

echo "header $header declares ${#declared[@]} modcrawl_* symbols"

status=0
for lib in "$@"; do
    if [ ! -f "$lib" ]; then
        echo "error: missing library $lib" >&2
        status=1
        continue
    fi

    if ! command -v nm >/dev/null 2>&1; then
        echo "skip: nm is not available to inspect $lib"
        continue
    fi

    # Portable across GNU nm and Apple nm: keep defined symbols only (drop
    # undefined ones, whose type column is `U`), then drop macOS' underscore.
    mapfile -t defined < <(
        nm -g "$lib" 2>/dev/null \
            | awk 'NF >= 2 && $(NF - 1) == "U" { next } NF >= 1 { print $NF }' \
            | sed 's/^_//' \
            | grep -E '^modcrawl_[a-z0-9_]+$' \
            | sort -u
    ) || true

    if [ "${#defined[@]}" -eq 0 ]; then
        echo "error: $lib defines no modcrawl_* symbols" >&2
        status=1
        continue
    fi

    missing="$(
        comm -23 \
            <(printf '%s\n' "${declared[@]}") \
            <(printf '%s\n' "${defined[@]}")
    )"

    if [ -n "$missing" ]; then
        echo "error: $lib is missing symbols declared in $header:" >&2
        printf '  %s\n' $missing >&2
        status=1
    else
        echo "ok: $lib exports all ${#declared[@]} declared symbols"
    fi
done

exit "$status"
