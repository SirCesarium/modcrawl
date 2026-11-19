/**
 * The docs home lives at `/docs`; every other page uses its slug.
 */
export function getDocHref(slug: string): string {
	return slug === 'home' ? '/docs' : `/docs/${slug}`;
}
