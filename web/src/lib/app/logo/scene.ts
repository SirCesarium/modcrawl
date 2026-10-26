import * as THREE from 'three';
import {
	createBoxGlowTexture,
	createStreakTexture,
	loadPixelTexture,
	loadPortalTexture
} from './assets';
import {
	BASE_YAW,
	BLOCK_ABSORB_AT,
	BLOCK_DELAY,
	BLOCK_END_X,
	BLOCK_END_Z,
	BLOCK_FADE_IN_END,
	BLOCK_FADE_START,
	BLOCK_START_X,
	BLOCK_START_Z,
	BLOCK_TRAVEL_DURATION,
	BLOCK_Y,
	FIT_MARGIN,
	FRAME_DEPTH,
	FRAME_HEIGHT,
	FRAME_WIDTH,
	GRASS_SIDE_TEXTURE,
	GRASS_TOP_TEXTURE,
	LIGHT_MAX,
	MAX_PITCH,
	MAX_YAW,
	PORTAL_DAMP,
	PORTAL_FRAME_MS,
	RAY_SPIN,
	ROTATION_DAMP
} from './config';
import { createGrassBlock, createPortalLogoParts } from './objects';

export interface PortalLogoOptions {
	canvas: HTMLCanvasElement;
	container: HTMLElement;
	/** Called once the grass block has crossed into the portal. */
	onAbsorb?: () => void;
	/** Called when the pointer leaves and the sequence is rewound. */
	onReset?: () => void;
}

export interface PortalLogo {
	destroy: () => void;
}

const disposeMaterial = (material: THREE.Material): void => {
	for (const value of Object.values(material)) {
		if (value instanceof THREE.Texture) value.dispose();
	}

	material.dispose();
};

const disposeObject = (root: THREE.Object3D): void => {
	root.traverse((object) => {
		if (!(object instanceof THREE.Mesh)) return;

		object.geometry.dispose();

		const materials = Array.isArray(object.material) ? object.material : [object.material];

		materials.forEach(disposeMaterial);
	});
};

export const createPortalLogo = ({
	canvas,
	container,
	onAbsorb,
	onReset
}: PortalLogoOptions): PortalLogo => {
	const scene = new THREE.Scene();
	const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.01, 1000);
	const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });

	renderer.setClearColor(0x000000, 0);
	renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

	scene.add(new THREE.AmbientLight(0x3d333d, 1.4));

	const keyLight = new THREE.DirectionalLight(0xffd3ff, 3.4);

	keyLight.position.set(-4, 2, 3);
	scene.add(keyLight);

	const boxTexture = createBoxGlowTexture();
	const streakTexture = createStreakTexture();
	const logo = createPortalLogoParts(boxTexture, streakTexture);
	const grass = createGrassBlock(
		loadPixelTexture(GRASS_TOP_TEXTURE),
		loadPixelTexture(GRASS_SIDE_TEXTURE)
	);
	const size = new THREE.Vector3(FRAME_WIDTH, FRAME_HEIGHT, FRAME_DEPTH);

	logo.group.rotation.y = BASE_YAW;

	logo.group.add(grass.mesh);
	scene.add(logo.group);

	let portalFrames = 1;
	let portalReady = false;
	let forceRender = true;

	const portalTexture = loadPortalTexture((frames) => {
		portalFrames = frames;
		portalReady = true;
		forceRender = true;
	});

	logo.portalMaterial.map = portalTexture;
	logo.portalMaterial.needsUpdate = true;

	const pointer = { yaw: 0, pitch: 0 };
	let active = false;
	let elapsed = 0;
	let absorbed = false;
	const clock = new THREE.Clock();
	let animationTime = 0;

	const onPointerMove = (event: PointerEvent): void => {
		pointer.yaw = ((event.clientX / window.innerWidth) * 2 - 1) * MAX_YAW;
		pointer.pitch = ((event.clientY / window.innerHeight) * 2 - 1) * MAX_PITCH;
	};

	const onWindowLeave = (): void => {
		pointer.yaw = 0;
		pointer.pitch = 0;
	};

	const onEnter = (): void => {
		active = true;
		elapsed = 0;
		absorbed = false;
		animationTime = 0;
	};

	const onLeave = (): void => {
		active = false;
		elapsed = 0;
		absorbed = false;
		grass.mesh.visible = false;
		grass.materials.forEach((material) => (material.opacity = 1));
		onReset?.();
	};

	window.addEventListener('pointermove', onPointerMove);
	document.documentElement.addEventListener('pointerleave', onWindowLeave);
	container.addEventListener('pointerenter', onEnter);
	container.addEventListener('pointerleave', onLeave);

	const fit = (aspect: number): void => {
		const halfHeight = Math.max(size.y / 2, size.x / 2 / aspect) * FIT_MARGIN;
		const halfWidth = halfHeight * aspect;
		const distance = Math.max(size.x, size.y, size.z) * 2;

		camera.left = -halfWidth;
		camera.right = halfWidth;
		camera.top = halfHeight;
		camera.bottom = -halfHeight;
		camera.near = 0.01;
		camera.far = distance * 2 + size.z;
		camera.position.set(0, 0, distance);
		camera.lookAt(0, 0, 0);
		camera.updateProjectionMatrix();
	};

	const resize = (): void => {
		const { clientWidth: width, clientHeight: height } = container;

		if (width === 0 || height === 0) return;

		renderer.setSize(width, height, false);
		fit(width / height);
		forceRender = true;
	};

	const observer = new ResizeObserver(resize);

	observer.observe(container);
	resize();

	const animate = (): void => {
		const delta = Math.min(clock.getDelta(), 0.05);
		const portalTarget = active ? 1 : 0;

		if (active) elapsed += delta;
		if (active || logo.portalMaterial.opacity > 0) animationTime += delta;

		logo.portalMaterial.opacity = THREE.MathUtils.damp(
			logo.portalMaterial.opacity,
			portalTarget,
			PORTAL_DAMP,
			delta
		);

		if (portalTarget === 0 && logo.portalMaterial.opacity < 0.005) {
			logo.portalMaterial.opacity = 0;
		}

		logo.group.rotation.x = THREE.MathUtils.damp(
			logo.group.rotation.x,
			pointer.pitch,
			ROTATION_DAMP,
			delta
		);
		logo.group.rotation.y = THREE.MathUtils.damp(
			logo.group.rotation.y,
			BASE_YAW + pointer.yaw,
			ROTATION_DAMP,
			delta
		);

		const portalOn = portalTarget === 1 || logo.portalMaterial.opacity > 0;
		const travelProgress = active ? (elapsed - BLOCK_DELAY) / BLOCK_TRAVEL_DURATION : -1;
		const traveling = travelProgress >= 0 && travelProgress < 1;

		if (traveling) {
			const eased = Math.pow(travelProgress, 1.2);
			const fadeIn = THREE.MathUtils.smoothstep(travelProgress, 0, BLOCK_FADE_IN_END);
			const fadeOut = THREE.MathUtils.smoothstep(travelProgress, BLOCK_FADE_START, 0.92);
			const opacity = fadeIn * (1 - fadeOut);

			grass.mesh.visible = opacity > 0.01;
			grass.mesh.position.set(
				THREE.MathUtils.lerp(BLOCK_START_X, BLOCK_END_X, eased),
				BLOCK_Y,
				THREE.MathUtils.lerp(BLOCK_START_Z, BLOCK_END_Z, eased)
			);
			grass.mesh.rotation.set(
				0.18 * travelProgress,
				-0.5 + 2.2 * travelProgress,
				Math.sin(travelProgress * Math.PI) * 0.16
			);
			grass.materials.forEach((material) => (material.opacity = opacity));
		} else {
			grass.mesh.visible = false;
		}

		if (travelProgress >= BLOCK_ABSORB_AT && !absorbed) {
			absorbed = true;
			onAbsorb?.();
		}

		logo.portal.visible = portalOn && portalReady;
		logo.glow.group.visible = portalOn;

		const pulse = 0.86 + 0.14 * Math.sin(animationTime * 3);
		const intensity = logo.portalMaterial.opacity * pulse;

		for (const beam of logo.glow.beams) beam.material.opacity = beam.baseOpacity * intensity;

		for (const ray of logo.glow.rays) {
			ray.material.opacity = ray.baseOpacity * intensity;
			ray.mesh.rotation.z = animationTime * RAY_SPIN * ray.speed;
		}

		logo.light.intensity = LIGHT_MAX * intensity;

		if (portalReady) {
			const frame = Math.floor((animationTime * 1000) / PORTAL_FRAME_MS) % portalFrames;
			portalTexture.offset.y = 1 - (frame + 1) / portalFrames;
		}

		const rotating =
			Math.abs(logo.group.rotation.x - pointer.pitch) > 0.0003 ||
			Math.abs(logo.group.rotation.y - (BASE_YAW + pointer.yaw)) > 0.0003;

		if (!portalOn && !rotating && !traveling && !forceRender) return;

		forceRender = false;
		renderer.render(scene, camera);
	};

	renderer.setAnimationLoop(animate);

	return {
		destroy: () => {
			window.removeEventListener('pointermove', onPointerMove);
			document.documentElement.removeEventListener('pointerleave', onWindowLeave);
			container.removeEventListener('pointerenter', onEnter);
			container.removeEventListener('pointerleave', onLeave);
			observer.disconnect();
			renderer.setAnimationLoop(null);
			disposeObject(scene);
			scene.clear();
			renderer.dispose();
		}
	};
};
