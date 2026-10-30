<script lang="ts">
	import type { Snippet } from 'svelte';
	import { twMerge } from 'tailwind-merge';
	import Icon from './Icon.svelte';

	type Size = 'sm' | 'md' | 'lg';

	interface Props {
		type?: 'text' | 'password' | 'search' | 'email' | 'url';
		password?: boolean;
		search?: boolean;
		value?: string;
		size?: Size;
		full?: boolean;
		pixel?: boolean;
		uppercase?: boolean;
		corners?: boolean;
		id?: string;
		label?: string;
		placeholder?: string;
		hint?: string;
		error?: string;
		disabled?: boolean;
		readonly?: boolean;
		required?: boolean;
		clearable?: boolean;
		shortcut?: string;
		icon?: string;
		iconRight?: string;
		prefix?: Snippet;
		suffix?: Snippet;
		class?: string;
		inputClass?: string;
		[key: string]: unknown;
	}

	let {
		type: typeProp,
		password = false,
		search = false,
		value = $bindable(''),
		size = 'md',
		full = false,
		pixel = false,
		uppercase = false,
		corners = true,
		id,
		label,
		placeholder,
		hint,
		error,
		disabled = false,
		readonly = false,
		required = false,
		clearable = false,
		shortcut,
		icon,
		iconRight,
		prefix,
		suffix,
		class: className = '',
		inputClass = '',
		...restProps
	}: Props = $props();

	let showPassword = $state(false);
	let inputRef = $state<HTMLInputElement | null>(null);

	let autoId = 'text-input-' + Math.random().toString(36).substring(2, 9);
	const inputId = $derived(id ?? autoId);

	const isPassword = $derived(password || typeProp === 'password');
	const isSearch = $derived(search || typeProp === 'search');

	const resolvedType = $derived.by(() => {
		if (isPassword) return showPassword ? 'text' : 'password';
		if (isSearch) return 'search';
		return typeProp ?? 'text';
	});

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

	// Global window shortcut listener (e.g. Ctrl+K, Cmd+K, /)
	const handleWindowKeyDown = (e: KeyboardEvent) => {
		if (!shortcut || disabled || readonly) return;

		const targetTag = (e.target as HTMLElement)?.tagName?.toLowerCase();
		const isTypingInAnotherField =
			(targetTag === 'input' || targetTag === 'textarea') && e.target !== inputRef;

		const normalized = shortcut.toLowerCase().trim();
		const isCtrlOrCmd = e.ctrlKey || e.metaKey;
		const hasModifier =
			normalized.includes('ctrl') || normalized.includes('cmd') || normalized.includes('⌘');

		if (hasModifier) {
			const expectedKey = normalized.split('+').pop()?.trim();
			if (isCtrlOrCmd && expectedKey && e.key.toLowerCase() === expectedKey) {
				e.preventDefault();
				inputRef?.focus();
				inputRef?.select();
			}
		} else if (normalized === '/' && !isTypingInAnotherField && e.key === '/') {
			e.preventDefault();
			inputRef?.focus();
		}
	};

	const handleKeyDown = (e: KeyboardEvent) => {
		if (e.key === 'Escape' && clearable && !disabled && !readonly) {
			e.preventDefault();
			value = '';
		}
	};

	const clearInput = (e: MouseEvent) => {
		e.stopPropagation();
		value = '';
		inputRef?.focus();
	};
</script>

<svelte:window onkeydown={handleWindowKeyDown} />

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
				: ''} {uppercase ? 'uppercase' : ''}"
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
			error &&
				'animate-input-shake border-red-500/80 bg-red-950/10 focus-within:border-red-400 focus-within:ring-red-400/40'
		)}
	>
		{@render cornerMarks()}

		<!-- Prefix slot or icon -->
		{#if prefix}
			<div class="shrink-0 text-neutral-400">
				{@render prefix()}
			</div>
		{:else if icon === 'search' || isSearch}
			<Icon name="search" size={iconSizes[size]} class="shrink-0 opacity-70" alt="Search" />
		{:else if icon}
			<Icon name={icon} size={iconSizes[size]} class="shrink-0 opacity-70" alt="" />
		{/if}

		<!-- The Input Element -->
		<input
			bind:this={inputRef}
			id={inputId}
			type={resolvedType}
			bind:value
			{placeholder}
			{disabled}
			{readonly}
			{required}
			onkeydown={handleKeyDown}
			style="caret-color: #ffffff; caret-shape: block;"
			class={twMerge(
				'w-full bg-transparent text-neutral-100 placeholder:text-neutral-500 placeholder:transition-opacity placeholder:duration-150 focus:outline-none focus:placeholder:opacity-30 [&::-ms-clear]:hidden [&::-ms-reveal]:hidden [&::-webkit-search-cancel-button]:hidden [&::-webkit-search-decoration]:hidden',
				uppercase && 'uppercase',
				inputClass
			)}
			{...restProps}
		/>

		<!-- Shortcut Badge -->
		{#if shortcut}
			<button
				type="button"
				tabindex="-1"
				aria-label={`Focus input with shortcut ${shortcut}`}
				class="shrink-0 cursor-pointer rounded-none border border-white/10 bg-neutral-800/80 px-1.5 py-0.5 font-mono text-[10px] text-neutral-400 transition select-none hover:border-purple-400/50 hover:text-purple-300 focus:outline-none focus-visible:ring-1 focus-visible:ring-purple-400 active:scale-95"
				onclick={() => {
					inputRef?.focus();
					inputRef?.select();
				}}
			>
				{shortcut}
			</button>
		{/if}

		<!-- Clearable Button -->
		{#if clearable && value !== '' && !disabled && !readonly}
			<button
				type="button"
				aria-label="Clear input"
				class="shrink-0 p-1 text-neutral-400 opacity-70 transition-opacity hover:text-neutral-100 hover:opacity-100 focus:outline-none focus-visible:ring-1 focus-visible:ring-purple-400 active:scale-95"
				onclick={clearInput}
			>
				<span aria-hidden="true" class="font-mono text-[11px] leading-none">X</span>
			</button>
		{/if}

		<!-- Password Toggle -->
		{#if isPassword}
			<button
				type="button"
				aria-label={showPassword ? 'Hide password' : 'Show password'}
				class="shrink-0 font-mono text-xs font-semibold text-neutral-400 transition select-none hover:text-purple-300 focus:outline-none focus-visible:ring-1 focus-visible:ring-purple-400 active:scale-95"
				onclick={() => (showPassword = !showPassword)}
			>
				{showPassword ? 'HIDE' : 'SHOW'}
			</button>
		{/if}

		<!-- Warning Icon on Error -->
		{#if error}
			<Icon name="alert" size={iconSizes[size]} class="shrink-0 opacity-90" alt="Error" />
		{/if}

		<!-- Suffix slot or icon -->
		{#if suffix}
			<div class="shrink-0 text-neutral-400">
				{@render suffix()}
			</div>
		{:else if iconRight}
			<Icon name={iconRight} size={iconSizes[size]} class="shrink-0 opacity-70" alt="" />
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
