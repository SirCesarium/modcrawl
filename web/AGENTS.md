# web

SvelteKit app that runs modcrawl in the browser through WASM.

This is Svelte 5 with runes, and runes are forced for this project in
`vite.config.ts`. Svelte 4 syntax (`export let`, stores for component state,
slots) does not compile here. Do not port patterns from older Svelte code you
may have seen.

Read `web/ARCHITECTURE.md` before adding components. It defines the tiers and
the allowed import direction:

```text
routes -> widgets -> components
routes, widgets -> stores and utilities
```

## Commands

Run from `web/`.

```sh
bun install --frozen-lockfile
bun run lint      # prettier --check + eslint
bun run check     # svelte-check
bun run test      # vitest in a browser
bun run build
bun run format    # prettier --write
```

## Rules

- Tests run in Vitest browser mode with Playwright and Chromium, and each test
  must contain an assertion (`requireAssertions: true`).
- Do not start the dev server unless the user asks.
- The app consumes the published `modcrawl` package from npm, not the local
  crate. A change under `src/` does not reach `web/` until the package is
  published.
- Formatting comes from `web/prettier.config.js`: tabs, single quotes, width 100.
- Write canonical Tailwind utilities. If a built-in or a token in the `@theme`
  block of `src/routes/layout.css` produces the same CSS, use it: `w-px` instead
  of `w-[1px]`, `p-4` instead of `p-[16px]`, `gap-2` instead of `gap-[8px]`.
  Keep arbitrary values for sizes, colors, and values the theme does not define.
  The Tailwind CSS IntelliSense extension flags the rest through
  `tailwindCSS.lint.suggestCanonicalClasses`; `bun run lint` does not, so a green
  CI does not mean the classes are canonical.
- For Svelte syntax or API questions, load the `svelte` skill instead of
  guessing.
