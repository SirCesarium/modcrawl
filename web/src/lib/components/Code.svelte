<script lang="ts">
	import Prism from 'prismjs';
	import 'prismjs/components/prism-json';
	import 'prismjs/components/prism-java';
	import 'prismjs/components/prism-bash';
	import 'prism-themes/themes/prism-gruvbox-dark.css';

	interface Props {
		code: string;
		language?: 'java' | 'json' | 'bash';
		class?: string;
	}

	let { code, language = 'java', class: className = '' }: Props = $props();

	const highlighted = $derived.by(() => {
		const targetGrammar = Prism.languages[language] || Prism.languages.text;
		try {
			return Prism.highlight(code.trim(), targetGrammar, language);
		} catch {
			return code.trim();
		}
	});
</script>

<div class="relative my-4 overflow-hidden rounded-xl shadow-lg {className}">
	<div
		class="absolute top-2 right-3 hidden font-mono text-xs tracking-wider uppercase opacity-50 sm:block"
	>
		{language}
	</div>
	<!-- eslint-disable svelte/no-at-html-tags -- Safe internal Prism.js highlighted output -->
	<pre class="overflow-x-auto p-3 font-mono text-[10px] leading-relaxed sm:p-4 sm:text-sm"><code
			class="language-{language}">{@html highlighted}</code
		></pre>
</div>
