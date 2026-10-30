<script lang="ts">
	import type { Snippet } from 'svelte';
	import { twMerge } from 'tailwind-merge';
	import Icon from './Icon.svelte';

	type Size = 'sm' | 'md' | 'lg';

	interface Props {
		value?: number;
		min?: number;
		max?: number;
		step?: number;
		controls?: boolean;
		size?: Size;
		full?: boolean;
		pixel?: boolean;
		corners?: boolean;
		id?: string;
		label?: string;
		placeholder?: string;
		hint?: string;
		error?: string;
		disabled?: boolean;
		readonly?: boolean;
		required?: boolean;
		prefix?: Snippet;
		suffix?: Snippet;
		class?: string;
		inputClass?: string;
		[key: string]: unknown;
	}

	let {
		value = $bindable(0),
		min,
		max,
		step = 1,
		controls = true,
		size = 'md',
		full = false,
		pixel = false,
		corners = true,
		id,
		label,
		placeholder,
		hint,
		error,
		disabled = false,
		readonly = false,
		required = false,
		prefix,
		suffix,
		class: className = '',
		inputClass = '',
		...restProps
	}: Props = $props();

	let isLimitHit = $state(false);
	let inputRef = $state<HTMLInputElement | null>(null);

	let autoId = 'num-input-' + Math.random().toString(36).substring(2, 9);
	const inputId = $derived(id ?? autoId);

	const sizeClasses: Record<Size, string> = {
		sm: 'h-9 px-2.5 text-xs gap-1.5',
		md: 'h-10 px-3.5 text-sm gap-2',
		lg: 'h-12 px-4 text-base gap-2.5'
	};

	const iconSizes: Record<Size, number> = {
		sm: 16,
		md: 18,
		lg: 20
	};

	const triggerLimitFeedback = () => {
		isLimitHit = true;
		setTimeout(() => {
			isLimitHit = false;
		}, 200);
	};

	const stepValue = (delta: number) => {
		if (disabled || readonly) return;
		const current = typeof value === 'number' ? value : parseFloat(String(value)) || 0;
		const next = current + delta;
		if (max !== undefined && next > max) {
			triggerLimitFeedback();
			value = max;
			return;
		}
		if (min !== undefined && next < min) {
			triggerLimitFeedback();
			value = min;
			return;
		}
		value = next;
	};

	// Continuous hold stepper
	let stepTimer: ReturnType<typeof setTimeout> | null = null;
	let stepInterval: ReturnType<typeof setInterval> | null = null;

	const startStepping = (direction: 1 | -1, e: MouseEvent | TouchEvent) => {
		if (disabled || readonly) return;
		const isShift = 'shiftKey' in e && (e as MouseEvent).shiftKey;
		const effectiveStep = isShift ? step * 10 : step;
		stepValue(direction * effectiveStep);

		stepTimer = setTimeout(() => {
			stepInterval = setInterval(() => {
				stepValue(direction * effectiveStep);
			}, 80);
		}, 300);
	};

	const stopStepping = () => {
		if (stepTimer) clearTimeout(stepTimer);
		if (stepInterval) clearInterval(stepInterval);
		stepTimer = null;
		stepInterval = null;
	};

	// Mouse wheel support
	const handleWheel = (e: WheelEvent) => {
		if (disabled || readonly) return;
		if (document.activeElement === inputRef) {
			e.preventDefault();
			const isShift = e.shiftKey;
			const effectiveStep = isShift ? step * 10 : step;
			if (e.deltaY < 0) stepValue(effectiveStep);
			else if (e.deltaY > 0) stepValue(-effectiveStep);
		}
	};

	// Keyboard arrows
	const handleKeyDown = (e: KeyboardEvent) => {
		if (disabled || readonly) return;
		if (e.key === 'ArrowUp') {
			e.preventDefault();
			const effectiveStep = e.shiftKey ? step * 10 : step;
			stepValue(effectiveStep);
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			const effectiveStep = e.shiftKey ? step * 10 : step;
			stepValue(-effectiveStep);
		}
	};
</script>

{#snippet cornerMarks()}
	{#if corners}
		<span
			aria-hidden="true"
			class={twMerge(
				'pointer-events-none absolute -top-1 -left-1 z-10 size-2 border-t-2 border-l-2 opacity-0 transition-opacity duration-150 group-focus-within:opacity-100 group-hover:opacity-60 motion-reduce:transition-none',
				error ? 'border-red-400' : 'border-purple-300'
			)}
		></span>
		<span
			aria-hidden="true"
			class={twMerge(
				'pointer-events-none absolute -top-1 -right-1 z-10 size-2 border-t-2 border-r-2 opacity-0 transition-opacity duration-150 group-focus-within:opacity-100 group-hover:opacity-60 motion-reduce:transition-none',
				error ? 'border-red-400' : 'border-purple-300'
			)}
		></span>
		<span
			aria-hidden="true"
			class={twMerge(
				'pointer-events-none absolute -bottom-1 -left-1 z-10 size-2 border-b-2 border-l-2 opacity-0 transition-opacity duration-150 group-focus-within:opacity-100 group-hover:opacity-60 motion-reduce:transition-none',
				error ? 'border-red-400' : 'border-purple-300'
			)}
		></span>
		<span
			aria-hidden="true"
			class={twMerge(
				'pointer-events-none absolute -right-1 -bottom-1 z-10 size-2 border-r-2 border-b-2 opacity-0 transition-opacity duration-150 group-focus-within:opacity-100 group-hover:opacity-60 motion-reduce:transition-none',
				error ? 'border-red-400' : 'border-purple-300'
			)}
		></span>
	{/if}
{/snippet}

<div class="relative flex flex-col gap-1.5 {full ? 'w-full' : 'w-auto'} {className}">
	<!-- Label -->
	{#if label}
		<label
			for={inputId}
			class="flex items-center gap-1.5 text-xs font-semibold text-neutral-300 select-none {pixel
				? 'font-pixel'
				: ''}"
		>
			<span>{label}</span>
			{#if required}
				<span class="text-purple-400">*</span>
			{/if}
		</label>
	{/if}

	<!-- Container -->
	<div
		class={twMerge(
			'group relative inline-flex items-center border border-neutral-700 bg-neutral-900/80 font-medium text-neutral-100 transition duration-150 ease-out focus-within:border-purple-400 focus-within:ring-1 focus-within:ring-purple-400/50 hover:border-neutral-500 active:scale-[0.99] motion-reduce:transform-none motion-reduce:transition-none',
			sizeClasses[size],
			pixel && 'font-pixel',
			full && 'w-full',
			disabled && 'pointer-events-none opacity-40',
			isLimitHit && 'animate-input-shake border-red-500/80',
			error &&
				'animate-input-shake border-red-500/80 bg-red-950/10 focus-within:border-red-400 focus-within:ring-red-400/40'
		)}
	>
		{@render cornerMarks()}

		<!-- Prefix slot -->
		{#if prefix}
			<div class="shrink-0 text-neutral-400">
				{@render prefix()}
			</div>
		{/if}

		<!-- The Number Input -->
		<input
			bind:this={inputRef}
			id={inputId}
			type="number"
			bind:value
			{placeholder}
			{disabled}
			{readonly}
			{required}
			{min}
			{max}
			{step}
			onwheel={handleWheel}
			onkeydown={handleKeyDown}
			style="caret-color: #ffffff; caret-shape: block;"
			class={twMerge(
				'w-full [appearance:textfield] bg-transparent text-neutral-100 placeholder:text-neutral-500 placeholder:transition-opacity placeholder:duration-150 focus:outline-none focus:placeholder:opacity-30 [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none',
				inputClass
			)}
			{...restProps}
		/>

		{#if controls && !readonly}
			<div class="flex items-center gap-1 border-l border-white/10 pl-2">
				<button
					type="button"
					aria-label="Decrement"
					class="flex size-6 items-center justify-center border border-white/10 bg-neutral-900 text-xs font-bold text-neutral-200 transition select-none hover:bg-white/10 focus:outline-none focus-visible:ring-1 focus-visible:ring-purple-400 active:translate-y-px motion-reduce:transform-none"
					onmousedown={(e) => startStepping(-1, e)}
					onmouseup={stopStepping}
					onmouseleave={stopStepping}
					ontouchstart={(e) => startStepping(-1, e)}
					ontouchend={stopStepping}
				>
					-
				</button>
				<button
					type="button"
					aria-label="Increment"
					class="flex size-6 items-center justify-center border border-white/10 bg-neutral-900 text-xs font-bold text-neutral-200 transition select-none hover:bg-white/10 focus:outline-none focus-visible:ring-1 focus-visible:ring-purple-400 active:translate-y-px motion-reduce:transform-none"
					onmousedown={(e) => startStepping(1, e)}
					onmouseup={stopStepping}
					onmouseleave={stopStepping}
					ontouchstart={(e) => startStepping(1, e)}
					ontouchend={stopStepping}
				>
					+
				</button>
			</div>
		{/if}

		<!-- Warning Icon on Error -->
		{#if error}
			<Icon name="alert" size={iconSizes[size]} class="shrink-0 opacity-90" alt="Error" />
		{/if}

		<!-- Suffix slot -->
		{#if suffix}
			<div class="shrink-0 text-neutral-400">
				{@render suffix()}
			</div>
		{/if}
	</div>

	<!-- Hint -->
	{#if error}
		<p
			class="animate-[code-line-in_150ms_ease-out] font-mono text-xs text-red-400 transition duration-150"
		>
			{error}
		</p>
	{:else if hint}
		<p class="font-mono text-xs text-neutral-400">
			{hint}
		</p>
	{/if}
</div>
