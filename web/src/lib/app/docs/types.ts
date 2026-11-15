import type { Component } from 'svelte';

/** Frontmatter metadata supported in /docs/*.md files. */
export interface DocMetadata {
	/** Title of the document. */
	title: string;
	/** Short description/summary of the document. */
	description?: string;
	/** Topic badges (e.g. ['CLI', 'Minecraft', 'Fabric']). */
	topics?: string[];
	/** Section/category name for grouping (e.g. 'Getting Started', 'Commands'). */
	section?: string;
	/** Icon name corresponding to files in static/icons/ (e.g. 'modcrawl'). */
	icon?: string;
	/** Minecraft loaders applicable (e.g. ['fabric', 'forge', 'neoforge', 'quilt']). */
	loaders?: string[];
	/** Date of last update (e.g. '2026-11-10'). */
	updated?: string;
}

/** Represents a loaded document item with its slug, metadata, and path. */
export interface DocItem {
	slug: string;
	path: string;
	metadata: DocMetadata;
}

/** A sidebar section; `name` is null for pages registered without a section. */
export interface DocSection {
	name: string | null;
	docs: DocItem[];
}

/** Represents a fully resolved document with its Svelte component. */
export interface LoadedDoc extends DocItem {
	component: Component;
}
