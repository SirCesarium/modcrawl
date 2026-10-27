interface Star {
	x: number;
	y: number;
	size: number;
	vx: number;
	vy: number;
	alpha: number;
	phase: number;
	twinkle: number;
	color: string;
}

const COLORS = ['179, 92, 255', '235, 219, 178', '200, 200, 210', '255, 255, 255'];
const MAX_PIXELS = 2_400_000;
const TARGET_FPS = 24;

export function createStarfield(canvas: HTMLCanvasElement): { destroy: () => void } {
	const context = canvas.getContext('2d');

	if (!context) return { destroy: () => {} };

	const motion = window.matchMedia('(prefers-reduced-motion: reduce)');

	let width = 0;
	let height = 0;
	let stars: Star[] = [];
	let frame = 0;
	let lastTime = 0;
	let running = false;

	const random = (min: number, max: number) => min + Math.random() * (max - min);

	const buildStars = () => {
		const count = Math.max(80, Math.min(380, Math.round((width * height) / 6500)));

		stars = Array.from({ length: count }, () => {
			const roll = Math.random();
			const size = roll > 0.9 ? 3 : roll > 0.65 ? 2 : 1;

			return {
				x: Math.round(Math.random() * width),
				y: Math.round(Math.random() * height),
				size,
				vx: random(-0.04, 0.04),
				vy: random(0.02, 0.07),
				alpha: random(0.4, 0.9),
				phase: random(0, Math.PI * 2),
				twinkle: random(0.004, 0.012),
				color: COLORS[Math.floor(Math.random() * COLORS.length)]
			};
		});
	};

	let entranceAlpha = 0;

	const draw = (delta: number) => {
		if (entranceAlpha < 1) {
			entranceAlpha = Math.min(1, entranceAlpha + delta * 0.018);
		}

		context.clearRect(0, 0, width, height);

		for (const star of stars) {
			const alpha = star.alpha * (0.7 + 0.3 * Math.sin(star.phase)) * entranceAlpha;

			context.fillStyle = `rgba(${star.color}, ${alpha})`;
			context.fillRect(Math.round(star.x), Math.round(star.y), star.size, star.size);

			star.x += star.vx * delta;
			star.y += star.vy * delta;
			star.phase += star.twinkle * delta;

			if (star.x < -2) star.x = width + 2;
			if (star.x > width + 2) star.x = -2;
			if (star.y > height + 2) star.y = -2;
		}
	};

	const resize = () => {
		const cssWidth = canvas.clientWidth || window.innerWidth;
		const cssHeight = canvas.clientHeight || window.innerHeight;
		const scale = Math.min(1, Math.sqrt(MAX_PIXELS / (cssWidth * cssHeight)));

		width = Math.max(1, Math.round(cssWidth * scale));
		height = Math.max(1, Math.round(cssHeight * scale));

		canvas.width = width;
		canvas.height = height;

		buildStars();
		draw(0);
	};

	const tick = (time: number) => {
		if (!running) return;

		if (lastTime === 0) lastTime = time;

		const elapsed = time - lastTime;

		if (elapsed >= 1000 / TARGET_FPS) {
			draw(elapsed / (1000 / 60));
			lastTime = time;
		}

		frame = requestAnimationFrame(tick);
	};

	const start = () => {
		if (running) return;

		running = true;
		lastTime = 0;
		frame = requestAnimationFrame(tick);
	};

	const stop = () => {
		running = false;
		cancelAnimationFrame(frame);
	};

	const syncMotion = () => {
		if (motion.matches || document.hidden) {
			stop();
			draw(0);
			return;
		}

		start();
	};

	const onVisibility = () => {
		if (document.hidden) {
			stop();
			return;
		}

		syncMotion();
	};

	resize();
	window.addEventListener('resize', resize);
	window.addEventListener('visibilitychange', onVisibility);
	motion.addEventListener('change', syncMotion);
	syncMotion();

	return {
		destroy: () => {
			stop();
			window.removeEventListener('resize', resize);
			window.removeEventListener('visibilitychange', onVisibility);
			motion.removeEventListener('change', syncMotion);
		}
	};
}
