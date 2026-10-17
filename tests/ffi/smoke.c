/* Smoke test for the published C ABI.
 *
 * Compiles the shipped header the way a consumer would — as C *and* as C++ —
 * and exercises every export at least once, including the shapes that used to
 * break: a `char *paths[]` argument for `modcrawl_dupes_json`, NULL inputs,
 * and a zero-length buffer.
 *
 * CI builds this against the freshly built library (see ci.yml); it is not a
 * cargo test target, cargo ignores `.c` files under `tests/`.
 */

#include "modcrawl.h"

#include <stdio.h>
#include <string.h>

/* This test exercises the *full* surface we publish, so the header must declare
 * it. Without this, a header that silently drops the classfile exports (the
 * v0.5.2 musl package) would make the checks below quietly compile out. */
#ifndef MODCRAWL_FEATURE_CLASSFILE
#error "modcrawl.h must declare the classfile exports (MODCRAWL_FEATURE_CLASSFILE)"
#endif

/* Read a returned string the way a caller must: check it, then free it. */
static int expect(char *value, const char *needle, const char *what) {
    if (!value) {
        fprintf(stderr, "FAIL %s: returned NULL\n", what);
        return 1;
    }
    if (needle != NULL && strstr(value, needle) == NULL) {
        fprintf(stderr, "FAIL %s: expected \"%s\" in \"%s\"\n", what, needle, value);
        modcrawl_free_string(value);
        return 1;
    }
    modcrawl_free_string(value);
    return 0;
}

/* A NULL that we never allocated must be accepted, not freed. */
static int expect_null(char *value, const char *what) {
    if (value != NULL) {
        fprintf(stderr, "FAIL %s: expected NULL\n", what);
        modcrawl_free_string(value);
        return 1;
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <jar> [jar...]\n", argv[0]);
        return 2;
    }
    const char *jar = argv[1];
    int failures = 0;

    failures += expect(modcrawl_identify(jar), "Fabric", "modcrawl_identify");
    failures += expect(modcrawl_metadata(jar), "Test Mod", "modcrawl_metadata");
    failures += expect(modcrawl_metadata_json(jar), "testmod", "modcrawl_metadata_json");
    failures += expect(modcrawl_deps(jar, false), "fabricloader", "modcrawl_deps");
    failures += expect(modcrawl_deps_json(jar, true), "fabricloader", "modcrawl_deps_json");

    failures += expect(modcrawl_classes_json(jar), "[]", "modcrawl_classes_json");
    failures += expect(modcrawl_mixins_json(jar), "[]", "modcrawl_mixins_json");
    failures += expect(modcrawl_grep_json(jar, "fabric"), "[]", "modcrawl_grep_json");

    /* The canonical C shape for an array of paths. */
    char *paths[8];
    int count = argc - 1 < 7 ? argc - 1 : 7;
    for (int i = 0; i < count; i++) {
        paths[i] = argv[i + 1];
    }
    paths[count] = NULL;
    failures += expect(modcrawl_dupes_json(paths), NULL, "modcrawl_dupes_json");

    /* Bad input is an error, not a crash. */
    failures += expect_null(modcrawl_identify(NULL), "modcrawl_identify(NULL)");
    failures += expect_null(modcrawl_metadata_json(NULL), "modcrawl_metadata_json(NULL)");
    failures += expect_null(modcrawl_deps(NULL, true), "modcrawl_deps(NULL, true)");
    failures += expect_null(modcrawl_identify_bytes(NULL, 64), "modcrawl_identify_bytes(NULL, 64)");
    failures += expect_null(modcrawl_metadata_json_bytes(NULL, 64),
                            "modcrawl_metadata_json_bytes(NULL, 64)");
    failures += expect_null(modcrawl_identify_bytes((const uint8_t *)"PK\x03\x04", 0),
                            "modcrawl_identify_bytes(..., 0)");
    failures += expect_null(modcrawl_classes_json(NULL), "modcrawl_classes_json(NULL)");
    failures += expect_null(modcrawl_grep_json(jar, NULL), "modcrawl_grep_json(jar, NULL)");
    failures += expect_null(modcrawl_dupes_json(NULL), "modcrawl_dupes_json(NULL)");

    modcrawl_free_string(NULL);

    if (failures != 0) {
        fprintf(stderr, "ffi smoke: %d check(s) failed\n", failures);
        return 1;
    }
    printf("ffi smoke: ok\n");
    return 0;
}
