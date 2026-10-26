<script>
	import AppBar from '#lib/app/widgets/AppBar.svelte';
	import Code from '#lib/components/Code.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Typography from '#lib/components/Typography.svelte';
</script>

<AppBar />

<main class="pt-16">
	<section class="flex min-h-[calc(100vh-4rem)] items-center justify-between gap-4 px-40">
		<div class="flex flex-col items-center gap-4">
			<div class="flex items-center gap-4">
				<Icon name="modcrawl" size={128} />
				<Typography variant="h1" component="span" pixel uppercase>
					<span class="text-pink-300">mod</span>crawl
				</Typography>
			</div>
			<Typography variant="h5" component="span" pixel uppercase>
				Analyze Minecraft mods and plugins!
			</Typography>
		</div>
		<div>
		<!-- TODO: replace this with drop folder/jar(s) -->
			<Code
				language="java"
				code={`
public class HelloWorld {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}`}
			/>
		</div>
	</section>

	<section></section>
</main>
