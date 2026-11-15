import { getDocBySlug } from '#app/docs/loader.ts';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const doc = await getDocBySlug('home');

	if (!doc) {
		throw error(404, 'Home documentation not found in /docs/home.md');
	}

	return { doc };
};
