import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';
import type { Plugin } from 'vite';
import { playwright } from '@vitest/browser-playwright';
import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { mdsvex, escapeSvelte } from 'mdsvex';
import Prism from 'prismjs';
import 'prismjs/components/prism-json.js';
import 'prismjs/components/prism-java.js';
import 'prismjs/components/prism-bash.js';
import 'prismjs/components/prism-powershell.js';
import 'prismjs/components/prism-markdown.js';
import 'prismjs/components/prism-python.js';
import 'prismjs/components/prism-javascript.js';
import 'prismjs/components/prism-typescript.js';

const docsLayoutPath = fileURLToPath(
	new URL('./src/lib/app/docs/DocsMarkdownLayout.svelte', import.meta.url)
);
const docsDir = fileURLToPath(new URL('../docs', import.meta.url));
const docsIndexPath = fileURLToPath(new URL('../docs/index.json', import.meta.url));

/** Reloads the page when docs files are added, removed, or edited outside Vite's root. */
function docsWatcher(): Plugin {
	return {
		name: 'modcrawl-docs-watch',
		configureServer(server) {
			server.watcher.add(docsDir);

			const reload = (file: string) => {
				if (file === docsIndexPath || file.endsWith('.md')) {
					server.ws.send({ type: 'full-reload' });
				}
			};

			server.watcher.on('add', reload);
			server.watcher.on('unlink', reload);
			server.watcher.on('change', reload);
		}
	};
}

/** Vite and Vitest configuration for the web app. */
export default defineConfig({
	resolve: {
		alias: [
			{
				find: /^#app\/(components|widgets|logo|utils|backdrop|docs)(.*)/,
				replacement: `${fileURLToPath(new URL('./src/lib/app', import.meta.url))}/$1$2`
			},
			{
				find: /^#lib\/(.*)/,
				replacement: `${fileURLToPath(new URL('./src/lib', import.meta.url))}/$1`
			},
			{
				find: /^#core\/(.*)/,
				replacement: `${fileURLToPath(new URL('./src/lib/core', import.meta.url))}/$1`
			}
		]
	},
	server: {
		fs: {
			allow: ['..']
		}
	},
	plugins: [
		docsWatcher(),
		tailwindcss(),
		sveltekit({
			extensions: ['.svelte', '.md'],
			preprocess: [
				mdsvex({
					extensions: ['.md'],
					layout: docsLayoutPath,
					highlight: {
						highlighter(code, lang) {
							const language = (lang || 'text').toLowerCase();
							const grammar = Prism.languages[language] || Prism.languages.text;
							const rawHighlighted = grammar ? Prism.highlight(code, grammar, language) : code;
							const highlighted = escapeSvelte(rawHighlighted);

							return `<div data-code-block class="relative my-4 overflow-hidden border border-purple-500/25 bg-purple-950/40 shadow-lg"><div class="code-toolbar" data-language="${language}"></div><pre class="language-${language} overflow-x-auto p-3 font-mono text-[10px] leading-relaxed sm:p-4 sm:text-sm"><code class="language-${language}">{@html \`${highlighted}\`}</code></pre></div>`;
						}
					}
				})
			],
			compilerOptions: {
				runes: ({ filename }) =>
					filename.endsWith('.md') || filename.split(/[/\\]/).includes('node_modules')
						? undefined
						: true
			},

			adapter: adapter({
				pages: 'build',
				assets: 'build',
				fallback: '404.html',
				precompress: false,
				strict: true
			}),
			prerender: {
				handleUnseenRoutes: 'warn',
				handleHttpError: ({ path, referrer, message }) => {
					// Ignore relative asset or external markdown links in docs examples
					if (path.startsWith('/README') || path.includes('imagen.png')) {
						return;
					}
					throw new Error(`${message} (${path} linked from ${referrer})`);
				}
			}
		})
	],
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				extends: './vite.config.ts',
				test: {
					name: 'client',
					browser: {
						enabled: true,
						provider: playwright(),
						instances: [{ browser: 'chromium', headless: true }]
					},
					include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
					exclude: ['src/lib/server/**']
				}
			},

			{
				extends: './vite.config.ts',
				test: {
					name: 'server',
					environment: 'node',
					include: ['src/**/*.{test,spec}.{js,ts}'],
					exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
				}
			}
		]
	}
});
