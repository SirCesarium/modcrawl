import { getDocsNavigation } from '#app/docs/loader.ts';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async () => {
	const sections = await getDocsNavigation();
	return { sections };
};
