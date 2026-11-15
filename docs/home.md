---
title: "Overview"
topics:
  - "Overview"
  - "Getting Started"
section: "Getting Started"
---

<script>
	import Button from '#app/components/Button.svelte';
	// import DocCard from '#app/docs/DocCard.svelte';
</script>

## Installation

Download the installer for your platform:

<div class="my-5 flex flex-wrap items-center gap-3">
	<Button
		href="https://github.com/SirCesarium/modcrawl/releases/latest/download/modcrawl-windows-x86_64.msi"
		icon="windows"
		pixel
		uppercase
    size="lg"
		particleClass="bg-blue-50 ring-blue-700"
		class="border-blue-300 bg-blue-300 shadow-[3px_3px_0_0_color-mix(in_srgb,var(--color-blue-700)_70%,transparent)] hover:border-blue-200 hover:bg-blue-200 active:border-blue-400 active:bg-blue-400"
	>
		Windows
	</Button>
	<Button
		href="https://github.com/SirCesarium/modcrawl/releases/latest/download/modcrawl-linux-amd64.deb"
		icon="debian"
		color="danger"
		pixel
		uppercase
    size="lg"
	>
		Debian
	</Button>
	<Button
		href="https://github.com/SirCesarium/modcrawl/releases/latest/download/modcrawl-linux-x86_64.rpm"
		icon="fedora"
		pixel
		uppercase
    size="lg"
		particleClass="bg-blue-50 ring-blue-700"
		class="border-blue-400 bg-blue-400 shadow-[3px_3px_0_0_color-mix(in_srgb,var(--color-blue-800)_70%,transparent)] hover:border-blue-300 hover:bg-blue-300 active:border-blue-500 active:bg-blue-500"
	>
		Fedora
	</Button>
	<Button
		href="https://github.com/SirCesarium/modcrawl/releases/latest/download/modcrawl-macos-arm64"
		icon="macos"
		color="neutral"
		pixel
		uppercase
    size="lg"
		class="border-[#e0e2e6] bg-[#e0e2e6] shadow-[3px_3px_0_0_color-mix(in_srgb,var(--color-neutral-600)_70%,transparent)] hover:border-[#eceef2] hover:bg-[#eceef2] active:border-[#c9ccd2] active:bg-[#c9ccd2]"
	>
		macOS
	</Button>
</div>

For other architectures, like Linux ARM64, check
[all releases](https://github.com/SirCesarium/modcrawl/releases).

### Using Cargo

- Make sure you have [Rust](https://rust-lang.org/) installed.

```bash
cargo install modcrawl
```

<!--
## Explore the docs

Lorem ipsum

<div id="explore" class="mt-6 grid gap-3 sm:grid-cols-2">
	<DocCard
		title="Loaders and metadata"
		description="Which loader owns a jar, and what the mod says about itself."
	/>
	<DocCard
		title="Dependencies"
		description="Required and embedded dependencies, and what is missing from the folder."
	/>
	<DocCard
		title="Sides"
		description="Client, server, both, or unknown, and how each jar earns its label."
	/>
	<DocCard
		title="Class files"
		description="Every class, its Java version and access flags, and constant-pool searches."
	/>
	<DocCard
		title="Mixins"
		description="Mixin targets and the targets shared by several mods."
	/>
	<DocCard
		title="Duplicate classes"
		description="Entries that collide on the classpath and the jars behind them."
	/>
	<DocCard
		title="Library"
		description="The same analysis from Rust, or from JavaScript through the WASM package."
	/>
</div> -->

## Need help?

- [Open an issue](https://github.com/SirCesarium/modcrawl/issues).
- [Join the discussions](https://github.com/SirCesarium/modcrawl/discussions) for
  questions and roadmap updates.
