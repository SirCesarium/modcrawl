import type { Component } from 'svelte';
import docsIndex from '../../../../../docs/index.json';
import type { DocItem, DocSection, LoadedDoc, DocMetadata } from './types';

interface DocsIndexSection {
	name: string;
	pages: string[];
}

type DocsIndexEntry = DocsIndexSection | string;

const docModules = import.meta.glob<Record<string, unknown>>('../../../../../docs/*.md');

function getSlugFromPath(filePath: string): string {
	const filename = filePath.split('/').pop() ?? '';
	return filename.replace(/\.md$/, '');
}

async function loadDocItem(path: string): Promise<DocItem> {
	const slug = getSlugFromPath(path);
	const mod = await docModules[path]();
	const metadata = (mod.metadata ?? {}) as DocMetadata;

	return {
		slug,
		path,
		metadata: {
			title: metadata.title || slug,
			description: metadata.description,
			topics: metadata.topics ?? [],
			section: metadata.section,
			icon: metadata.icon,
			loaders: metadata.loaders,
			updated: metadata.updated
		}
	};
}

/**
 * Lists all doc items with metadata, sorted by title.
 */
export async function getAllDocs(): Promise<DocItem[]> {
	const items = await Promise.all(Object.keys(docModules).map(loadDocItem));

	return items.sort((a, b) => a.metadata.title.localeCompare(b.metadata.title));
}

/**
 * Builds the sidebar navigation from `docs/index.json`. Sections group their
 * pages in the declared order; loose strings render as standalone pages.
 */
export async function getDocsNavigation(): Promise<DocSection[]> {
	const docs = await getAllDocs();
	const docsBySlug = new Map(docs.map((doc) => [doc.slug, doc]));
	const sections: DocSection[] = [];

	for (const entry of docsIndex as DocsIndexEntry[]) {
		if (typeof entry === 'string') {
			const doc = docsBySlug.get(entry);
			if (doc) sections.push({ name: null, docs: [doc] });
			continue;
		}

		const pages = entry.pages
			.map((page) => docsBySlug.get(page))
			.filter((doc): doc is DocItem => Boolean(doc));

		if (pages.length > 0) {
			sections.push({ name: entry.name, docs: pages });
		}
	}

	return sections;
}

/**
 * Returns a loaded document by slug, or null if not found.
 */
export async function getDocBySlug(slug: string): Promise<LoadedDoc | null> {
	const path = Object.keys(docModules).find((candidate) => getSlugFromPath(candidate) === slug);
	if (!path) return null;

	const mod = await docModules[path]();
	const item = await loadDocItem(path);

	return {
		...item,
		component: mod.default as unknown as Component
	};
}

/**
 * Returns all slugs for static prerendering (excluding 'home' which lives at /docs).
 */
export function getDocSlugs(): string[] {
	return Object.keys(docModules)
		.map(getSlugFromPath)
		.filter((slug) => slug !== 'home');
}
