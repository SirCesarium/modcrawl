<script lang="ts">
	import type { Snippet } from 'svelte';
	import type {
		HTMLAnchorAttributes,
		HTMLButtonAttributes,
		MouseEventHandler,
		PointerEventHandler
	} from 'svelte/elements';
	import { twMerge } from 'tailwind-merge';
	import Icon from './Icon.svelte';

	type Variant = 'solid' | 'soft' | 'outline' | 'ghost' | 'link';
	type Size = 'xs' | 'sm' | 'md' | 'lg';
	type IconPosition = 'left' | 'right';
	type Color = 'primary' | 'neutral' | 'success' | 'warning' | 'danger' | 'info';

	interface Props {
		variant?: Variant;
		color?: Color;
		size?: Size;
		full?: boolean;
		pixel?: boolean;
		uppercase?: boolean;
		icon?: string;
		iconPosition?: IconPosition;
		iconSize?: number | string;
		iconClass?: string;
		href?: string;
		target?: HTMLAnchorAttributes['target'];
		rel?: string;
		disabled?: boolean;
		type?: HTMLButtonAttributes['type'];
		class?: string;
		onclick?: MouseEventHandler<HTMLButtonElement | HTMLAnchorElement>;
		onpointerdown?: PointerEventHandler<HTMLButtonElement | HTMLAnchorElement>;
		onpointerenter?: PointerEventHandler<HTMLButtonElement | HTMLAnchorElement>;
		onpointerleave?: PointerEventHandler<HTMLButtonElement | HTMLAnchorElement>;
		children?: Snippet;
		[key: string]: unknown;
	}

	let {
		variant = 'solid',
		color = 'primary',
		size = 'md',
		full = false,
		pixel = false,
		uppercase = false,
		icon,
		iconPosition = 'left',
		iconSize,
		iconClass = '',
		href,
		target,
		rel,
		disabled = false,
		type = 'button',
		class: className = '',
		onclick,
		onpointerdown,
		onpointerenter,
		onpointerleave,
		children,
		...restProps
	}: Props = $props();

	const isLink = $derived(href !== undefined);

	const particleClasses: Record<Color, string> = {
		primary: 'bg-purple-50 ring-purple-700',
		neutral: 'bg-neutral-50 ring-neutral-700',
		success: 'bg-emerald-50 ring-emerald-700',
		warning: 'bg-amber-50 ring-amber-700',
		danger: 'bg-red-50 ring-red-700',
		info: 'bg-cyan-50 ring-cyan-700'
	};

	const offsetShadowClasses: Record<Color, string> = {
		primary: 'shadow-[3px_3px_0_0_var(--color-purple-700)]',
		neutral: 'shadow-[3px_3px_0_0_var(--color-neutral-700)]',
		success: 'shadow-[3px_3px_0_0_var(--color-emerald-700)]',
		warning: 'shadow-[3px_3px_0_0_var(--color-amber-700)]',
		danger: 'shadow-[3px_3px_0_0_var(--color-red-700)]',
		info: 'shadow-[3px_3px_0_0_var(--color-cyan-700)]'
	};

	const offsetHoverShadowClasses: Record<Color, string> = {
		primary: 'hover:shadow-[5px_5px_0_0_var(--color-purple-700)]',
		neutral: 'hover:shadow-[5px_5px_0_0_var(--color-neutral-700)]',
		success: 'hover:shadow-[5px_5px_0_0_var(--color-emerald-700)]',
		warning: 'hover:shadow-[5px_5px_0_0_var(--color-amber-700)]',
		danger: 'hover:shadow-[5px_5px_0_0_var(--color-red-700)]',
		info: 'hover:shadow-[5px_5px_0_0_var(--color-cyan-700)]'
	};

	const cornerClasses: Record<Color, string> = {
		primary: 'border-purple-100',
		neutral: 'border-neutral-100',
		success: 'border-emerald-100',
		warning: 'border-amber-100',
		danger: 'border-red-100',
		info: 'border-cyan-100'
	};

	let bounceAnimation: Animation | undefined;
	let hoverAnimation: Animation | undefined;

	const cancelBounce = () => {
		bounceAnimation?.cancel();
		bounceAnimation = undefined;
	};

	const cancelHover = () => {
		hoverAnimation?.cancel();
		hoverAnimation = undefined;
	};

	const prefersReducedMotion = () =>
		typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

	const playBounce = (node: HTMLElement) => {
		const isLink = variant === 'link';
		const start = isLink ? '0 0' : '3px 3px';
		const overshoot = isLink ? '0 0' : '-1px -1px';
		const settle = isLink ? '0 0' : '1px 1px';

		cancelHover();
		cancelBounce();

		const animation = node.animate(
			[
				{ translate: start, scale: '0.98', offset: 0 },
				{ translate: overshoot, scale: '1.02', offset: 0.4 },
				{ translate: settle, scale: '0.995', offset: 0.7 },
				{ translate: '0 0', scale: '1', offset: 1 }
			],
			{
				duration: 420,
				easing: 'cubic-bezier(0.34, 1.56, 0.64, 1)'
			}
		);

		bounceAnimation = animation;

		animation.finished
			.then(() => {
				if (bounceAnimation === animation) bounceAnimation = undefined;
			})
			.catch(() => {});
	};

	const playHoverBounce = (node: HTMLElement) => {
		cancelBounce();
		cancelHover();

		const animation = node.animate(
			[
				{ translate: '0 -2px', offset: 0 },
				{ translate: '0 -5px', offset: 0.35 },
				{ translate: '0 -1px', offset: 0.6 },
				{ translate: '0 -3px', offset: 0.8 },
				{ translate: '0 -2px', offset: 1 }
			],
			{
				duration: 480,
				easing: 'cubic-bezier(0.22, 1, 0.36, 1)'
			}
		);

		hoverAnimation = animation;

		animation.finished
			.then(() => {
				if (hoverAnimation === animation) hoverAnimation = undefined;
			})
			.catch(() => {});
	};

	const spawnParticles = (node: HTMLElement, event: MouseEvent) => {
		const rect = node.getBoundingClientRect();
		// Clients without pointer precision (keyboard, assistive tech) burst from the center.
		const isKeyboardClick = event.detail === 0;
		const x = isKeyboardClick ? rect.width / 2 : event.clientX - rect.left;
		const y = isKeyboardClick ? rect.height / 2 : event.clientY - rect.top;

		const startAngle = Math.random() * Math.PI * 2;

		for (let index = 0; index < 10; index += 1) {
			const size = 3 + Math.floor(Math.random() * 3);
			const particle = document.createElement('span');
			particle.setAttribute('aria-hidden', 'true');
			particle.className = `pointer-events-none absolute z-10 ring-1 ${particleClasses[color]}`;
			particle.style.left = `${x - size / 2}px`;
			particle.style.top = `${y - size / 2}px`;
			particle.style.width = `${size}px`;
			particle.style.height = `${size}px`;
			node.appendChild(particle);

			const angle = startAngle + (index / 10) * Math.PI * 2 + (Math.random() - 0.5) * 0.18;
			const distance = 20 + Math.random() * 35;
			const dx = Math.cos(angle) * distance;
			const dy = Math.sin(angle) * distance;

			const animation = particle.animate(
				[
					{ translate: '0 0', scale: '1', opacity: 1, offset: 0 },
					{
						translate: `${dx * 0.6}px ${dy * 0.6}px`,
						scale: '0.85',
						opacity: 1,
						offset: 0.5
					},
					{ translate: `${dx}px ${dy}px`, scale: '0.2', opacity: 0, offset: 1 }
				],
				{
					duration: 480 + Math.random() * 260,
					delay: Math.random() * 30,
					easing: 'cubic-bezier(0.16, 1, 0.3, 1)'
				}
			);

			animation.finished.then(() => particle.remove()).catch(() => particle.remove());
		}
	};

	const handleClick: MouseEventHandler<HTMLButtonElement | HTMLAnchorElement> = (event) => {
		if (disabled) {
			event.preventDefault();
			return;
		}

		if (!prefersReducedMotion()) {
			const node = event.currentTarget;
			playBounce(node);
			spawnParticles(node, event);
		}

		onclick?.(event);
	};

	const handlePointerDown: PointerEventHandler<HTMLButtonElement | HTMLAnchorElement> = (event) => {
		cancelHover();
		cancelBounce();
		onpointerdown?.(event);
	};

	const handlePointerEnter: PointerEventHandler<HTMLButtonElement | HTMLAnchorElement> = (
		event
	) => {
		if (!disabled && variant !== 'link' && !prefersReducedMotion()) {
			playHoverBounce(event.currentTarget);
		}
		onpointerenter?.(event);
	};

	const handlePointerLeave: PointerEventHandler<HTMLButtonElement | HTMLAnchorElement> = (
		event
	) => {
		cancelHover();
		onpointerleave?.(event);
	};

	const baseClasses =
		'inline-flex group relative items-center justify-center overflow-visible rounded-none border font-medium whitespace-nowrap select-none transition-[top,box-shadow,background-color,border-color,color] duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 disabled:pointer-events-none disabled:opacity-40 aria-disabled:pointer-events-none aria-disabled:opacity-40';

	const focusOutlineClasses: Record<Color, string> = {
		primary: 'focus-visible:outline-purple-300',
		neutral: 'focus-visible:outline-neutral-300',
		success: 'focus-visible:outline-emerald-300',
		warning: 'focus-visible:outline-amber-300',
		danger: 'focus-visible:outline-red-300',
		info: 'focus-visible:outline-cyan-300'
	};

	const toneClasses: Record<Color, Record<Variant, string>> = {
		primary: {
			solid:
				'border-purple-300 bg-purple-300 text-neutral-950 hover:border-purple-200 hover:bg-purple-200 active:border-purple-400 active:bg-purple-400',
			soft: 'border-purple-300/25 bg-purple-300/10 text-purple-100 hover:border-purple-300/50 hover:bg-purple-300/20 active:bg-purple-300/25',
			outline:
				'border-purple-300/40 bg-transparent text-purple-100 hover:border-purple-300/70 hover:bg-purple-300/10 active:bg-purple-300/15',
			ghost:
				'border-transparent bg-transparent text-purple-300 hover:bg-purple-300/10 hover:text-purple-200 active:bg-purple-300/15',
			link: 'border-transparent bg-transparent text-purple-300 hover:text-purple-200 active:text-purple-400'
		},
		neutral: {
			solid:
				'border-neutral-200 bg-neutral-200 text-neutral-950 hover:border-neutral-100 hover:bg-neutral-100 active:border-neutral-300 active:bg-neutral-300',
			soft: 'border-neutral-300/25 bg-neutral-300/10 text-neutral-100 hover:border-neutral-300/50 hover:bg-neutral-300/20 active:bg-neutral-300/25',
			outline:
				'border-neutral-300/30 bg-transparent text-neutral-100 hover:border-neutral-300/60 hover:bg-white/5 active:bg-white/10',
			ghost:
				'border-transparent bg-transparent text-neutral-300 hover:bg-white/5 hover:text-neutral-100 active:bg-white/10',
			link: 'border-transparent bg-transparent text-neutral-300 hover:text-neutral-100 active:text-neutral-400'
		},
		success: {
			solid:
				'border-emerald-300 bg-emerald-300 text-neutral-950 hover:border-emerald-200 hover:bg-emerald-200 active:border-emerald-400 active:bg-emerald-400',
			soft: 'border-emerald-300/25 bg-emerald-300/10 text-emerald-100 hover:border-emerald-300/50 hover:bg-emerald-300/20 active:bg-emerald-300/25',
			outline:
				'border-emerald-300/40 bg-transparent text-emerald-100 hover:border-emerald-300/70 hover:bg-emerald-300/10 active:bg-emerald-300/15',
			ghost:
				'border-transparent bg-transparent text-emerald-300 hover:bg-emerald-300/10 hover:text-emerald-200 active:bg-emerald-300/15',
			link: 'border-transparent bg-transparent text-emerald-300 hover:text-emerald-200 active:text-emerald-400'
		},
		warning: {
			solid:
				'border-amber-300 bg-amber-300 text-neutral-950 hover:border-amber-200 hover:bg-amber-200 active:border-amber-400 active:bg-amber-400',
			soft: 'border-amber-300/25 bg-amber-300/10 text-amber-100 hover:border-amber-300/50 hover:bg-amber-300/20 active:bg-amber-300/25',
			outline:
				'border-amber-300/40 bg-transparent text-amber-100 hover:border-amber-300/70 hover:bg-amber-300/10 active:bg-amber-300/15',
			ghost:
				'border-transparent bg-transparent text-amber-300 hover:bg-amber-300/10 hover:text-amber-200 active:bg-amber-300/15',
			link: 'border-transparent bg-transparent text-amber-300 hover:text-amber-200 active:text-amber-400'
		},
		danger: {
			solid:
				'border-red-300 bg-red-300 text-neutral-950 hover:border-red-200 hover:bg-red-200 active:border-red-400 active:bg-red-400',
			soft: 'border-red-300/25 bg-red-300/10 text-red-100 hover:border-red-300/50 hover:bg-red-300/20 active:bg-red-300/25',
			outline:
				'border-red-300/40 bg-transparent text-red-100 hover:border-red-300/70 hover:bg-red-300/10 active:bg-red-300/15',
			ghost:
				'border-transparent bg-transparent text-red-300 hover:bg-red-300/10 hover:text-red-200 active:bg-red-300/15',
			link: 'border-transparent bg-transparent text-red-300 hover:text-red-200 active:text-red-400'
		},
		info: {
			solid:
				'border-cyan-300 bg-cyan-300 text-neutral-950 hover:border-cyan-200 hover:bg-cyan-200 active:border-cyan-400 active:bg-cyan-400',
			soft: 'border-cyan-300/25 bg-cyan-300/10 text-cyan-100 hover:border-cyan-300/50 hover:bg-cyan-300/20 active:bg-cyan-300/25',
			outline:
				'border-cyan-300/40 bg-transparent text-cyan-100 hover:border-cyan-300/70 hover:bg-cyan-300/10 active:bg-cyan-300/15',
			ghost:
				'border-transparent bg-transparent text-cyan-300 hover:bg-cyan-300/10 hover:text-cyan-200 active:bg-cyan-300/15',
			link: 'border-transparent bg-transparent text-cyan-300 hover:text-cyan-200 active:text-cyan-400'
		}
	};

	const sizeClasses: Record<Size, string> = {
		xs: 'h-8 gap-1.5 px-2.5 text-xs',
		sm: 'h-9 gap-2 px-3 text-sm',
		md: 'h-10 gap-2 px-4 text-base',
		lg: 'h-12 gap-2.5 px-5 text-lg'
	};

	const iconSizes: Record<Size, number> = {
		xs: 16,
		sm: 16,
		md: 16,
		lg: 24
	};

	const resolvedIconSize = $derived(iconSize ?? iconSizes[size]);
	const resolvedIconClass = $derived(
		twMerge(
			'transition-transform duration-150 group-active:translate-y-px group-active:scale-90',
			variant === 'solid' && 'brightness-0',
			iconClass
		)
	);
	const resolvedRel = $derived(rel ?? (target === '_blank' ? 'noopener noreferrer' : undefined));

	const computedClass = $derived(
		twMerge(
			baseClasses,
			focusOutlineClasses[color],
			sizeClasses[size],
			toneClasses[color][variant],
			variant !== 'ghost' && variant !== 'link' && offsetShadowClasses[color],
			variant !== 'link' && 'hover:-top-0.5',
			variant !== 'ghost' && variant !== 'link' && offsetHoverShadowClasses[color],
			variant !== 'link' &&
				'active:translate-x-0.75 active:translate-y-0.75 active:scale-[0.98] active:shadow-none',
			variant === 'link' && 'h-auto px-0 py-0 underline-offset-4 hover:underline',
			full && 'w-full',
			pixel && 'font-pixel',
			uppercase && 'uppercase',
			className
		)
	);
</script>

{#snippet corners()}
	<span
		aria-hidden="true"
		class="pointer-events-none absolute -top-1 -left-1 z-10 size-2 scale-75 border-t-2 border-l-2 opacity-0 transition duration-150 group-hover:scale-100 group-hover:opacity-100 {cornerClasses[
			color
		]}"
	></span>
	<span
		aria-hidden="true"
		class="pointer-events-none absolute -top-1 -right-1 z-10 size-2 scale-75 border-t-2 border-r-2 opacity-0 transition duration-150 group-hover:scale-100 group-hover:opacity-100 {cornerClasses[
			color
		]}"
	></span>
	<span
		aria-hidden="true"
		class="pointer-events-none absolute -bottom-1 -left-1 z-10 size-2 scale-75 border-b-2 border-l-2 opacity-0 transition duration-150 group-hover:scale-100 group-hover:opacity-100 {cornerClasses[
			color
		]}"
	></span>
	<span
		aria-hidden="true"
		class="pointer-events-none absolute -right-1 -bottom-1 z-10 size-2 scale-75 border-r-2 border-b-2 opacity-0 transition duration-150 group-hover:scale-100 group-hover:opacity-100 {cornerClasses[
			color
		]}"
	></span>
{/snippet}

{#snippet content()}
	{@render corners()}
	{#if icon && iconPosition === 'left'}
		<Icon name={icon} size={resolvedIconSize} class={resolvedIconClass} alt="" aria-hidden="true" />
	{/if}
	{#if children}
		<span class="inline-block transition-transform duration-150 group-active:translate-x-0.5">
			{@render children()}
		</span>
	{/if}
	{#if icon && iconPosition === 'right'}
		<Icon name={icon} size={resolvedIconSize} class={resolvedIconClass} alt="" aria-hidden="true" />
	{/if}
{/snippet}

<!-- eslint-disable svelte/no-navigation-without-resolve -- Button accepts arbitrary external and internal destinations. -->
{#if isLink}
	<a
		class={computedClass}
		{href}
		{target}
		rel={resolvedRel}
		aria-disabled={disabled || undefined}
		tabindex={disabled ? -1 : undefined}
		onclick={handleClick}
		onpointerdown={handlePointerDown}
		onpointerenter={handlePointerEnter}
		onpointerleave={handlePointerLeave}
		{...restProps}
	>
		{@render content()}
	</a>
{:else}
	<button
		class={computedClass}
		{type}
		{disabled}
		onclick={handleClick}
		onpointerdown={handlePointerDown}
		onpointerenter={handlePointerEnter}
		onpointerleave={handlePointerLeave}
		{...restProps}
	>
		{@render content()}
	</button>
{/if}
<!-- eslint-enable svelte/no-navigation-without-resolve -->
