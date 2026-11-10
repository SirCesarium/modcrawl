---
name: svelte
description: Svelte 5 and SvelteKit guidance for the web app, including runes, the forced-runes config in this repository, and the official documentation sets for LLMs. Use when writing, reviewing, or debugging anything in web/.
---

# Svelte in this repository

`web/` runs Svelte 5 with SvelteKit. Runes are enabled for every project file by
`vite.config.ts`:

```js
sveltekit({
  compilerOptions: {
    runes: ({ filename }) =>
      filename.split(/[/\\]/).includes('node_modules') ? undefined : true
  }
})
```

That means every `.svelte` and `.svelte.ts` file is compiled in runes mode,
including a component written with Svelte 4 syntax. `export let`, `$:` for
reactive statements, and the slot API will not work.

## Documentation

Read the official docs before inventing an API:

| Need | Source |
| --- | --- |
| Compact reference, lowest token cost | https://svelte.dev/llms-small.txt |
| Svelte 5 package docs | https://svelte.dev/docs/svelte/llms.txt |
| SvelteKit docs | https://svelte.dev/docs/kit/llms.txt |
| Everything, with examples | https://svelte.dev/llms-full.txt |

`https://svelte.dev/llms.txt` indexes all of them.

## Syntax that applies here

- `$state`, `$derived`, `$effect`, and `$props` replace `let`, `$:`, and
  `export let`.
- Snippets (`{#snippet}` and `{@render}`) replace slots.
- `onclick={handler}` replaces `on:click`.
- `$state.raw` holds data that is replaced instead of mutated.

When a pattern looks unfamiliar, check `llms-small.txt` before writing it. Do not
assume the Svelte 4 API still applies.
