<script lang="ts">
	import CodeToolbar from '#app/docs/CodeToolbar.svelte';
	import Prism from 'prismjs';
	import 'prismjs/components/prism-json';
	import 'prismjs/components/prism-java';
	import 'prismjs/components/prism-bash';
	import 'prismjs/components/prism-powershell';
	import 'prismjs/components/prism-markdown';
	import 'prismjs/components/prism-python';
	import 'prismjs/components/prism-javascript';
	import 'prismjs/components/prism-typescript';

	type Language =
		'java' | 'json' | 'bash' | 'powershell' | 'markdown' | 'python' | 'typescript' | (string & {});

	interface Props {
		code: string;
		language?: Language;
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

<div
	data-code-block
	class="relative my-4 overflow-hidden border border-purple-500/25 bg-purple-950/40 shadow-lg {className}"
>
	<CodeToolbar {language} />
	<!-- eslint-disable svelte/no-at-html-tags -- Safe internal Prism.js highlighted output -->
	<pre
		class="language-{language} overflow-x-auto font-mono text-[10px] leading-relaxed sm:text-sm"><code
			class="language-{language}">{@html highlighted}</code
		></pre>
</div>
