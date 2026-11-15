<script lang="ts">
	interface Props {
		class?: string;
	}

	let { class: className = '' }: Props = $props();
</script>

<hr class="docs-rule {className}" />

<style>
	.docs-rule {
		border: none;
		height: 1px;
		position: relative;
		background: linear-gradient(
			90deg,
			transparent 0%,
			color-mix(in srgb, var(--color-purple-500) 30%, transparent) 10%,
			color-mix(in srgb, var(--color-purple-400) 55%, transparent) 50%,
			color-mix(in srgb, var(--color-purple-500) 30%, transparent) 90%,
			transparent 100%
		);
	}

	.docs-rule::before,
	.docs-rule::after {
		content: '';
		position: absolute;
		top: -2px;
		width: 5px;
		height: 5px;
		background: color-mix(in srgb, var(--color-purple-400) 65%, transparent);
	}

	.docs-rule::before {
		left: calc(10% - 2px);
	}

	.docs-rule::after {
		right: calc(10% - 2px);
	}
</style>
