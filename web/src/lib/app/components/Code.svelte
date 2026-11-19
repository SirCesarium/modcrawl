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
	class="group/code relative my-6 overflow-hidden rounded-md border border-white/10 bg-neutral-900/50 {className}"
>
	<CodeToolbar {language} />
	<!-- eslint-disable svelte/no-at-html-tags -- Safe internal Prism.js highlighted output -->
	<pre
		class="language-{language} overflow-x-auto px-4 pt-1 pb-4 font-mono text-[13px] leading-6 sm:text-sm"><code
			class="language-{language}">{@html highlighted}</code
		></pre>
</div>
