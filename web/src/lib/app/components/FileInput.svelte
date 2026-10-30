<script lang="ts">
	import { twMerge } from 'tailwind-merge';
	import Button from './Button.svelte';
	import Icon from './Icon.svelte';

	interface Props {
		files?: File[];
		accept?: string;
		multiple?: boolean;
		folder?: boolean;
		height?: string;
		label?: string;
		hint?: string;
		error?: string;
		disabled?: boolean;
		required?: boolean;
		pixel?: boolean;
		uppercase?: boolean;
		corners?: boolean;
		class?: string;
		[key: string]: unknown;
	}

	let {
		files = $bindable<File[]>([]),
		accept,
		multiple = true,
		folder = false,
		height = 'h-44',
		label,
		hint,
		error,
		disabled = false,
		required = false,
		pixel = false,
		uppercase = false,
		corners = true,
		class: className = '',
		...restProps
	}: Props = $props();

	let isDragOver = $state(false);
	let isInvalidDrop = $state(false);
	let dropPulse = $state(false);
	let fileInputRef = $state<HTMLInputElement | null>(null);

	const dedupeFiles = (existing: File[], incoming: File[]): File[] => {
		const seen = new Set(existing.map((f) => `${f.name}:${f.size}:${f.lastModified}`));
		const fresh = incoming.filter((f) => !seen.has(`${f.name}:${f.size}:${f.lastModified}`));
		return [...existing, ...fresh];
	};

	const checkAccepted = (dt: DataTransfer): boolean => {
		if (!accept || !dt.items || dt.items.length === 0) return true;
		const allowed = accept.split(',').map((s) => s.trim().toLowerCase());
		for (let i = 0; i < dt.items.length; i++) {
			const item = dt.items[i];
			if (item.kind === 'file') {
				const name = item.type.toLowerCase();
				const matches = allowed.some((ext) => {
					if (ext.startsWith('.')) return true;
					return name === ext || (ext.endsWith('/*') && name.startsWith(ext.slice(0, -2)));
				});
				if (!matches) return false;
			}
		}
		return true;
	};

	const readEntries = (reader: FileSystemDirectoryReader) =>
		new Promise<FileSystemEntry[]>((resolve) => {
			reader.readEntries(
				(entries) => resolve(entries),
				() => resolve([])
			);
		});

	const readDirectory = async (entry: FileSystemDirectoryEntry): Promise<FileSystemEntry[]> => {
		const reader = entry.createReader();
		const entries: FileSystemEntry[] = [];

		while (true) {
			const batch = await readEntries(reader);

			if (!batch.length) return entries;

			entries.push(...batch);
		}
	};

	const collectFiles = async (entry: FileSystemEntry): Promise<File[]> => {
		if (entry.isFile) {
			const file = await new Promise<File | null>((resolve) => {
				(entry as FileSystemFileEntry).file(resolve, () => resolve(null));
			});

			return file ? [file] : [];
		}

		if (entry.isDirectory) {
			const entries = await readDirectory(entry as FileSystemDirectoryEntry);
			const nested = await Promise.all(entries.map(collectFiles));

			return nested.flat();
		}

		return [];
	};

	const applyFiles = (incoming: File[]) => {
		if (folder) {
			files = incoming;
		} else if (multiple) {
			files = dedupeFiles(files, incoming);
		} else {
			files = [incoming[0]];
		}
	};

	const pulseDropZone = () => {
		dropPulse = true;
		setTimeout(() => {
			dropPulse = false;
		}, 300);
	};

	const firstEntry = (dt: DataTransfer | null): FileSystemEntry | null => {
		if (!dt || dt.items.length === 0) return null;
		const item = dt.items[0];
		return typeof item.webkitGetAsEntry === 'function' ? item.webkitGetAsEntry() : null;
	};

	const openPicker = () => {
		if (disabled || files.length > 0) return;
		fileInputRef?.click();
	};

	// The shared button's hover bounce reads as a jitter at this compact size.
	const keepRaised = (event: PointerEvent & { currentTarget: HTMLElement }) => {
		event.currentTarget
			.getAnimations()
			.filter((animation) => !(animation instanceof CSSTransition))
			.forEach((animation) => animation.cancel());
	};

	const handleDragOver = (e: DragEvent) => {
		e.preventDefault();
		if (disabled) return;
		if (folder) {
			const entry = firstEntry(e.dataTransfer);
			isInvalidDrop = !!entry && entry.isFile;
		} else if (e.dataTransfer) {
			isInvalidDrop = !checkAccepted(e.dataTransfer);
		}
		isDragOver = true;
	};

	const handleDragLeave = (e: DragEvent) => {
		e.preventDefault();
		const current = e.currentTarget as HTMLElement | null;
		const related = e.relatedTarget as Node | null;
		if (current && related && current.contains(related)) {
			return;
		}
		isDragOver = false;
		isInvalidDrop = false;
	};

	const handleDrop = async (e: DragEvent) => {
		e.preventDefault();
		isDragOver = false;
		isInvalidDrop = false;
		if (disabled) return;

		const dt = e.dataTransfer;
		if (!dt) return;

		let droppedFiles: File[];

		if (folder) {
			const directoryEntries = Array.from(dt.items)
				.map((item) =>
					typeof item.webkitGetAsEntry === 'function' ? item.webkitGetAsEntry() : null
				)
				.filter((entry): entry is FileSystemEntry => !!entry && entry.isDirectory);

			if (directoryEntries.length === 0) return;

			const nested = await Promise.all(directoryEntries.map(collectFiles));
			droppedFiles = nested.flat();
		} else {
			droppedFiles = Array.from(dt.files);
		}

		if (droppedFiles.length === 0) return;

		applyFiles(droppedFiles);
		pulseDropZone();
	};

	const handleFileInputChange = (e: Event) => {
		const target = e.target as HTMLInputElement;
		if (!target.files) return;

		const selected = Array.from(target.files);
		if (selected.length === 0) return;

		applyFiles(selected);
		pulseDropZone();
	};

	const removeFile = (index: number, e: MouseEvent) => {
		e.stopPropagation();
		files = files.filter((_, i) => i !== index);
	};

	const clearAll = (e: MouseEvent) => {
		e.stopPropagation();
		files = [];
	};

	const emptyTitle = $derived(
		folder
			? 'Drop your mod folder here, or click to browse'
			: multiple
				? 'Drop your file(s) here, or click to browse'
				: 'Drop your file here, or click to browse'
	);

	const emptyHint = $derived(
		folder
			? 'Select or drag a folder'
			: accept
				? `Supported files: ${accept}`
				: 'Supports .jar, .zip files'
	);

	const cornerState = $derived(
		twMerge(
			'transition duration-150 motion-reduce:transition-none',
			error ? 'border-red-400' : isDragOver ? 'border-purple-300' : 'border-purple-100',
			isDragOver
				? 'scale-100 opacity-100'
				: files.length > 0
					? 'scale-75 opacity-0'
					: 'scale-75 opacity-0 group-hover/zone:scale-100 group-hover/zone:opacity-100 group-focus-visible/zone:scale-100 group-focus-visible/zone:opacity-100'
		)
	);

	const zoneClass = $derived(
		twMerge(
			'group/zone relative flex flex-col items-center justify-center rounded-none border border-dashed bg-neutral-900/50 p-4 text-center select-none transition-[top,background-color,border-color,color] duration-150 ease-out motion-reduce:transform-none motion-reduce:transition-none',
			height,
			files.length === 0 &&
				'hover:-top-0.5 hover:border-solid hover:border-purple-300/60 hover:bg-neutral-900/70 active:translate-x-0.75 active:translate-y-0.75 active:scale-[0.98]',
			isDragOver
				? '-top-0.5 border-solid border-purple-300 bg-purple-900/25'
				: 'border-neutral-500',
			isInvalidDrop && 'cursor-not-allowed border-solid border-red-500 bg-red-950/20',
			dropPulse && 'animate-drop-pulse border-purple-400',
			disabled && 'pointer-events-none opacity-40',
			error && 'animate-input-shake border-solid border-red-500/80 bg-red-950/10'
		)
	);
</script>

{#snippet cornerMarks()}
	{#if corners}
		<span
			aria-hidden="true"
			class="pointer-events-none absolute -top-1 -left-1 z-30 size-2 border-t-2 border-l-2 {cornerState}"
		></span>
		<span
			aria-hidden="true"
			class="pointer-events-none absolute -top-1 -right-1 z-30 size-2 border-t-2 border-r-2 {cornerState}"
		></span>
		<span
			aria-hidden="true"
			class="pointer-events-none absolute -bottom-1 -left-1 z-30 size-2 border-b-2 border-l-2 {cornerState}"
		></span>
		<span
			aria-hidden="true"
			class="pointer-events-none absolute -right-1 -bottom-1 z-30 size-2 border-r-2 border-b-2 {cornerState}"
		></span>
	{/if}
{/snippet}

<div class="relative flex w-full flex-col gap-1.5 {className}">
	<!-- Label -->
	{#if label}
		<span
			class="flex items-center gap-1.5 text-xs font-semibold text-neutral-300 select-none {pixel
				? 'font-pixel'
				: ''} {uppercase ? 'uppercase' : ''}"
		>
			<span>{label}</span>
			{#if required}
				<span class="text-purple-400">*</span>
			{/if}
		</span>
	{/if}

	<!-- Drop Zone -->
	<div
		role="button"
		tabindex={disabled || files.length > 0 ? -1 : 0}
		aria-label={label ?? (folder ? 'Upload Folder' : 'Upload File')}
		onclick={openPicker}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				openPicker();
			}
		}}
		ondragover={handleDragOver}
		ondragleave={handleDragLeave}
		ondrop={handleDrop}
		class={zoneClass}
		{...restProps}
	>
		{@render cornerMarks()}

		<!-- Hidden native input -->
		<input
			bind:this={fileInputRef}
			type="file"
			class="hidden"
			accept={folder ? undefined : accept}
			multiple={!folder && multiple}
			webkitdirectory={folder ? true : undefined}
			{disabled}
			onchange={handleFileInputChange}
		/>

		{#if isDragOver && !isInvalidDrop}
			<!-- Marquee "DROP IT" ticker when dragging -->
			<div
				class="pointer-events-none absolute inset-0 z-20 flex items-center justify-center overflow-hidden"
			>
				<div
					class="font-pixel flex animate-[marquee_4s_linear_infinite] text-base font-bold tracking-widest whitespace-nowrap text-purple-200 uppercase"
				>
					<span>DROP IT • DROP IT • DROP IT • DROP IT • DROP IT • DROP IT •&nbsp;</span>
					<span>DROP IT • DROP IT • DROP IT • DROP IT • DROP IT • DROP IT •&nbsp;</span>
				</div>
			</div>
		{/if}

		<div
			class="flex size-full flex-col items-center justify-center transition-opacity duration-150 {isDragOver &&
			!isInvalidDrop
				? 'pointer-events-none opacity-0'
				: ''}"
		>
			{#if files.length > 0}
				<!-- Display Selected Files -->
				<div class="flex size-full flex-col justify-between gap-2">
					<div
						class="flex shrink-0 items-center justify-between text-xs font-medium text-purple-300 uppercase"
					>
						<span class={pixel ? 'font-pixel' : ''}>
							{files.length} file{files.length === 1 ? '' : 's'}
						</span>
						<Button
							size="xs"
							variant="ghost"
							color="danger"
							class="h-6 px-1.5 text-[11px] normal-case"
							onclick={clearAll}
							onpointerenter={keepRaised}
						>
							Clear all
						</Button>
					</div>

					<div class="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto pr-1">
						{#each files as f, idx (f.name + idx)}
							<div
								class="flex shrink-0 items-center justify-between border border-neutral-700 bg-neutral-900/80 px-2.5 py-1.5 text-xs transition-colors duration-150 hover:border-purple-300/40 hover:bg-neutral-900"
							>
								<div class="flex items-center gap-2 truncate">
									<Icon name="file" size={16} class="opacity-70" alt="File" />
									<span class="truncate text-neutral-200" title={f.name}>{f.name}</span>
								</div>
								<button
									type="button"
									aria-label="Remove file"
									class="font-pixel ml-1 flex size-6 shrink-0 items-center justify-center border border-transparent text-[10px] leading-none text-neutral-400 transition-colors duration-150 hover:border-red-300/25 hover:bg-red-300/10 hover:text-red-300 focus:outline-none focus-visible:ring-1 focus-visible:ring-purple-400 active:translate-y-px"
									onclick={(e) => removeFile(idx, e)}
								>
									<span aria-hidden="true">X</span>
								</button>
							</div>
						{/each}
					</div>
				</div>
			{:else}
				<!-- Empty Drop Placeholder -->
				<div class="flex flex-col items-center justify-center gap-3">
					<div
						class="flex size-12 items-center justify-center border border-purple-300 bg-purple-300 text-neutral-950 transition-[box-shadow,background-color,border-color] duration-150 ease-out group-hover/zone:shadow-[3px_3px_0_0_var(--color-purple-700)]"
					>
						<Icon name={folder ? 'folder' : 'upload'} size={24} class="brightness-0" alt="" />
					</div>
					<div class="flex flex-col gap-1">
						<p
							class="text-sm font-semibold text-neutral-200 transition-colors duration-150 group-hover/zone:text-purple-200 {pixel
								? 'font-pixel'
								: ''} {uppercase ? 'uppercase' : ''}"
						>
							{emptyTitle}
						</p>
						<p class="font-mono text-xs text-neutral-400">
							{emptyHint}
						</p>
					</div>
				</div>
			{/if}
		</div>
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
