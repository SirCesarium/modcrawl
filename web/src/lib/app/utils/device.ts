/** True on touch-first devices where hover-driven rotation is not available. */
export const isCoarsePointer = (): boolean =>
	typeof window !== 'undefined' && window.matchMedia('(hover: none), (pointer: coarse)').matches;
