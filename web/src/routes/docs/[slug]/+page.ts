import { getDocBySlug, getDocSlugs } from '#app/docs/loader.ts';
import { error } from '@sveltejs/kit';
import type { PageLoad, EntryGenerator } from './$types';

export const entries: EntryGenerator = () => {
	return getDocSlugs().map((slug: string) => ({ slug }));
};

export const load: PageLoad = async ({ params }) => {
	const doc = await getDocBySlug(params.slug);

	if (!doc) {
		throw error(404, `Documentation page not found: ${params.slug}`);
	}

	return { doc };
};
