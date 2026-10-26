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
	BLOCK_END_Y,
	BLOCK_END_Z,
	BLOCK_FADE_IN_END,
	BLOCK_FADE_START,
	BLOCK_START_X,
	BLOCK_START_Y,
	BLOCK_START_Z,
	BLOCK_TRAVEL_DURATION,
	FIT_MARGIN,
	FRAME_DEPTH,
	FRAME_FADE_DURATION,
	FRAME_HEIGHT,
	FRAME_WIDTH,
	GRASS_SIDE_TEXTURE,
	GRASS_TOP_TEXTURE,
	LIGHT_MAX,
	MAX_PITCH,
	MAX_YAW,
	MOBILE_BASE_YAW,
	OBSIDIAN_TEXTURE,
	PORTAL_DAMP,
	PORTAL_FADE_DELAY,
	PORTAL_FRAME_MS,
	RAY_SPIN,
	ROTATION_DAMP
} from './config';
import { isCoarsePointer } from './device';
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
	const obsidianTexture = loadPixelTexture(OBSIDIAN_TEXTURE);
	const size = new THREE.Vector3(FRAME_WIDTH, FRAME_HEIGHT, FRAME_DEPTH);

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

	const frameMix = { value: 0 };

	logo.frameMaterial.map = obsidianTexture;
	logo.frameMaterial.customProgramCacheKey = () => 'obsidian-frame';
	logo.frameMaterial.onBeforeCompile = (shader) => {
		shader.uniforms.uObsidian = frameMix;
		shader.fragmentShader = shader.fragmentShader
			.replace('#include <common>', '#include <common>\nuniform float uObsidian;')
			.replace(
				'#include <map_fragment>',
				'#ifdef USE_MAP\n\tdiffuseColor.rgb = mix(vec3(1.0), texture2D(map, vMapUv).rgb, uObsidian);\n#endif'
			);
	};

	const pointer = { yaw: 0, pitch: 0 };
	// Touch devices have no reliable hover position; keep the portal at its base angle.
	const coarsePointer = isCoarsePointer();
	const baseYaw = coarsePointer ? MOBILE_BASE_YAW : BASE_YAW;

	logo.group.rotation.y = baseYaw;

	let active = false;
	let elapsed = 0;
	let absorbed = false;
	const clock = new THREE.Clock();
	let animationTime = 0;
	let loopRunning = false;

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

	const animate = (): void => {
		const delta = Math.min(clock.getDelta(), 0.05);

		if (active) elapsed += delta;
		if (active || logo.portalMaterial.opacity > 0) animationTime += delta;

		const portalTarget = active && elapsed >= PORTAL_FADE_DELAY ? 1 : 0;

		logo.portalMaterial.opacity = THREE.MathUtils.damp(
			logo.portalMaterial.opacity,
			portalTarget,
			PORTAL_DAMP,
			delta
		);

		if (portalTarget === 0 && logo.portalMaterial.opacity < 0.005) {
			logo.portalMaterial.opacity = 0;
		}

		if (active) {
			frameMix.value = Math.max(
				frameMix.value,
				THREE.MathUtils.smoothstep(elapsed, 0, FRAME_FADE_DURATION)
			);
		} else {
			frameMix.value = THREE.MathUtils.damp(frameMix.value, 0, PORTAL_DAMP, delta);
		}

		logo.group.rotation.x = THREE.MathUtils.damp(
			logo.group.rotation.x,
			pointer.pitch,
			ROTATION_DAMP,
			delta
		);
		logo.group.rotation.y = THREE.MathUtils.damp(
			logo.group.rotation.y,
			baseYaw + pointer.yaw,
			ROTATION_DAMP,
			delta
		);

		const portalOn = portalTarget === 1 || logo.portalMaterial.opacity > 0;
		const travelProgress = active ? (elapsed - BLOCK_DELAY) / BLOCK_TRAVEL_DURATION : -1;
		const traveling = travelProgress >= 0 && travelProgress < 1;

		if (traveling) {
			const eased = Math.pow(travelProgress, 1.4);
			const fadeIn = THREE.MathUtils.smoothstep(travelProgress, 0, BLOCK_FADE_IN_END);
			const fadeOut = THREE.MathUtils.smoothstep(travelProgress, BLOCK_FADE_START, BLOCK_ABSORB_AT);
			const opacity = fadeIn * (1 - fadeOut);

			grass.mesh.visible = opacity > 0.01;
			grass.mesh.position.set(
				THREE.MathUtils.lerp(BLOCK_START_X, BLOCK_END_X, eased),
				THREE.MathUtils.lerp(BLOCK_START_Y, BLOCK_END_Y, eased),
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
			Math.abs(logo.group.rotation.y - (baseYaw + pointer.yaw)) > 0.0003;
		const frameOn = frameMix.value > 0.005;

		if (!portalOn && !rotating && !traveling && !frameOn && !forceRender) {
			stopLoop();
			return;
		}

		forceRender = false;
		renderer.render(scene, camera);
	};

	function startLoop(): void {
		if (loopRunning) return;

		loopRunning = true;
		clock.start();
		renderer.setAnimationLoop(animate);
	}

	function stopLoop(): void {
		if (!loopRunning) return;

		loopRunning = false;
		renderer.setAnimationLoop(null);
	}

	const onPointerMove = (event: PointerEvent): void => {
		if (coarsePointer || event.pointerType === 'touch') return;

		pointer.yaw = ((event.clientX / window.innerWidth) * 2 - 1) * MAX_YAW;
		pointer.pitch = ((event.clientY / window.innerHeight) * 2 - 1) * MAX_PITCH;
		startLoop();
	};

	const onWindowLeave = (): void => {
		pointer.yaw = 0;
		pointer.pitch = 0;
		startLoop();
	};

	const onEnter = (): void => {
		active = true;
		elapsed = 0;
		absorbed = false;
		animationTime = 0;
		startLoop();
	};

	const onLeave = (): void => {
		active = false;
		elapsed = 0;
		absorbed = false;
		grass.mesh.visible = false;
		grass.materials.forEach((material) => (material.opacity = 1));
		onReset?.();
		startLoop();
	};

	const onPress = (): void => {
		if (active) onLeave();
		else onEnter();
	};

	if (coarsePointer) {
		container.addEventListener('pointerdown', onPress);
	} else {
		window.addEventListener('pointermove', onPointerMove, { passive: true });
		document.documentElement.addEventListener('pointerleave', onWindowLeave);
		container.addEventListener('pointerenter', onEnter);
		container.addEventListener('pointerleave', onLeave);

		if (container.matches(':hover')) onEnter();
	}

	const observer = new ResizeObserver(() => {
		resize();
		startLoop();
	});

	observer.observe(container);
	resize();
	startLoop();

	return {
		destroy: () => {
			window.removeEventListener('pointermove', onPointerMove);
			document.documentElement.removeEventListener('pointerleave', onWindowLeave);
			container.removeEventListener('pointerenter', onEnter);
			container.removeEventListener('pointerleave', onLeave);
			container.removeEventListener('pointerdown', onPress);
			observer.disconnect();
			stopLoop();
			disposeObject(scene);
			obsidianTexture.dispose();
			scene.clear();
			renderer.dispose();
		}
	};
};
