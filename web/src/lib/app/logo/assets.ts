import * as THREE from 'three';
import { PORTAL_COLUMNS, PORTAL_ROWS, PORTAL_TEXTURE } from './config';

const configurePixelTexture = <T extends THREE.Texture>(texture: T): T => {
	texture.colorSpace = THREE.SRGBColorSpace;
	texture.magFilter = THREE.NearestFilter;
	texture.minFilter = THREE.NearestFilter;
	texture.generateMipmaps = false;
	texture.wrapS = THREE.RepeatWrapping;
	texture.wrapT = THREE.RepeatWrapping;

	return texture;
};

const canvasOf = (size: number): [HTMLCanvasElement, CanvasRenderingContext2D] | null => {
	const canvas = document.createElement('canvas');
	canvas.width = size;
	canvas.height = size;

	const context = canvas.getContext('2d');

	return context ? [canvas, context] : null;
};

/** Soft square falloff used for the light beams. */
export const createBoxGlowTexture = (): THREE.CanvasTexture => {
	const size = 64;
	const entry = canvasOf(size);
	const canvas = entry?.[0] ?? document.createElement('canvas');
	const context = entry?.[1];

	if (context) {
		const image = context.createImageData(size, size);

		for (let y = 0; y < size; y++) {
			for (let x = 0; x < size; x++) {
				const u = Math.abs((x + 0.5) / size - 0.5) * 2;
				const v = Math.abs((y + 0.5) / size - 0.5) * 2;
				const falloff = Math.min(1, Math.max(0, (1 - Math.max(u, v)) / 0.65));
				const alpha = falloff * falloff * (3 - 2 * falloff);
				const index = (y * size + x) * 4;

				image.data[index] = 255;
				image.data[index + 1] = 255;
				image.data[index + 2] = 255;
				image.data[index + 3] = Math.round(alpha * 255);
			}
		}

		context.putImageData(image, 0, 0);
	}

	return configurePixelTexture(new THREE.CanvasTexture(canvas));
};

/** Random radial streaks that rotate behind the portal. */
export const createStreakTexture = (): THREE.CanvasTexture => {
	const size = 256;
	const entry = canvasOf(size);
	const canvas = entry?.[0] ?? document.createElement('canvas');
	const context = entry?.[1];

	if (context) {
		context.translate(size / 2, size / 2);
		context.globalCompositeOperation = 'lighter';

		for (let i = 0; i < 70; i++) {
			const angle = Math.random() * Math.PI * 2;
			const inner = size * 0.06;
			const length = size * (0.25 + Math.random() * 0.25);
			const cos = Math.cos(angle);
			const sin = Math.sin(angle);
			const alpha = 0.15 + Math.random() * 0.5;
			const gradient = context.createLinearGradient(
				cos * inner,
				sin * inner,
				cos * length,
				sin * length
			);

			gradient.addColorStop(0, `rgba(255, 255, 255, ${alpha})`);
			gradient.addColorStop(1, 'rgba(255, 255, 255, 0)');
			context.strokeStyle = gradient;
			context.lineWidth = 0.6 + Math.random() * 1.8;
			context.beginPath();
			context.moveTo(cos * inner, sin * inner);
			context.lineTo(cos * length, sin * length);
			context.stroke();
		}
	}

	return configurePixelTexture(new THREE.CanvasTexture(canvas));
};

export const loadPixelTexture = (url: string): THREE.Texture => {
	const texture = new THREE.TextureLoader().load(url, undefined, undefined, () => {
		console.error(`Failed to load texture: ${url}`);
	});

	return configurePixelTexture(texture);
};

/**
 * Loads the portal sprite sheet and reports how many frames it contains.
 * The texture is returned immediately, before the image finishes loading.
 */
export const loadPortalTexture = (onReady: (frames: number) => void): THREE.Texture => {
	const texture = new THREE.TextureLoader().load(
		PORTAL_TEXTURE,
		(loaded) => {
			const image = loaded.image as HTMLImageElement;
			const frames = Math.max(1, Math.round(image.height / image.width));

			loaded.wrapS = THREE.RepeatWrapping;
			loaded.wrapT = THREE.RepeatWrapping;
			loaded.repeat.set(PORTAL_COLUMNS, PORTAL_ROWS / frames);
			onReady(frames);
		},
		undefined,
		() => console.error(`Failed to load texture: ${PORTAL_TEXTURE}`)
	);

	return configurePixelTexture(texture);
};
