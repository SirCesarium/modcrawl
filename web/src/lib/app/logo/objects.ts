import * as THREE from 'three';
import {
	BEAM_LENGTH,
	BEAM_LAYERS,
	BEAM_OPACITY,
	BEAM_SPREAD,
	BLOCK_SIZE,
	FRAME_DEPTH,
	FRAME_HEIGHT,
	FRAME_HOLE_HEIGHT,
	FRAME_HOLE_WIDTH,
	FRAME_WIDTH,
	GLOW_COLOR,
	PORTAL_COLUMNS,
	PORTAL_HEIGHT,
	PORTAL_ROWS,
	PORTAL_WIDTH,
	RAY_SIZE
} from './config';

export interface GlowLayer {
	material: THREE.MeshBasicMaterial;
	baseOpacity: number;
}

export interface GlowRay extends GlowLayer {
	mesh: THREE.Mesh;
	speed: number;
}

export interface Glow {
	group: THREE.Group;
	beams: GlowLayer[];
	rays: GlowRay[];
}

export interface PortalLogoParts {
	group: THREE.Group;
	portal: THREE.Group;
	portalMaterial: THREE.MeshBasicMaterial;
	glow: Glow;
	light: THREE.PointLight;
}

const createFrame = (): THREE.Mesh => {
	const halfWidth = FRAME_WIDTH / 2;
	const halfHeight = FRAME_HEIGHT / 2;
	const shape = new THREE.Shape();

	shape.moveTo(-halfWidth, -halfHeight);
	shape.lineTo(halfWidth, -halfHeight);
	shape.lineTo(halfWidth, halfHeight);
	shape.lineTo(-halfWidth, halfHeight);
	shape.closePath();

	const hole = new THREE.Path();

	hole.moveTo(-FRAME_HOLE_WIDTH / 2, -FRAME_HOLE_HEIGHT / 2);
	hole.lineTo(FRAME_HOLE_WIDTH / 2, -FRAME_HOLE_HEIGHT / 2);
	hole.lineTo(FRAME_HOLE_WIDTH / 2, FRAME_HOLE_HEIGHT / 2);
	hole.lineTo(-FRAME_HOLE_WIDTH / 2, FRAME_HOLE_HEIGHT / 2);
	hole.closePath();
	shape.holes.push(hole);

	const geometry = new THREE.ExtrudeGeometry(shape, { depth: FRAME_DEPTH, bevelEnabled: false });
	geometry.center();

	const material = new THREE.MeshStandardMaterial({ color: 0xffffff, metalness: 0, roughness: 1 });

	return new THREE.Mesh(geometry, material);
};

const createPortal = (): { group: THREE.Group; material: THREE.MeshBasicMaterial } => {
	const material = new THREE.MeshBasicMaterial({
		transparent: true,
		opacity: 0,
		side: THREE.DoubleSide,
		depthWrite: false
	});

	const tileWidth = PORTAL_WIDTH / PORTAL_COLUMNS;
	const tileHeight = PORTAL_HEIGHT / PORTAL_ROWS;
	const geometry = new THREE.PlaneGeometry(tileWidth, tileHeight);
	const group = new THREE.Group();

	for (let row = 0; row < PORTAL_ROWS; row++) {
		for (let column = 0; column < PORTAL_COLUMNS; column++) {
			const tile = new THREE.Mesh(geometry, material);
			tile.position.x = -PORTAL_WIDTH / 2 + (column + 0.5) * tileWidth;
			tile.position.y = PORTAL_HEIGHT / 2 - (row + 0.5) * tileHeight;
			group.add(tile);
		}
	}

	return { group, material };
};

const glowMaterial = (map: THREE.Texture): THREE.MeshBasicMaterial =>
	new THREE.MeshBasicMaterial({
		map,
		color: GLOW_COLOR,
		transparent: true,
		opacity: 0,
		blending: THREE.AdditiveBlending,
		depthWrite: false,
		side: THREE.DoubleSide
	});

const createGlow = (boxTexture: THREE.Texture, streakTexture: THREE.Texture): Glow => {
	const group = new THREE.Group();
	const geometry = new THREE.PlaneGeometry(1, 1);
	const beams: GlowLayer[] = [];

	for (const side of [1, -1]) {
		for (let layer = 0; layer < BEAM_LAYERS; layer++) {
			const progress = (layer + 1) / BEAM_LAYERS;
			const material = glowMaterial(boxTexture);
			const mesh = new THREE.Mesh(geometry, material);
			const scale = 1 + progress * BEAM_SPREAD;

			mesh.scale.set(PORTAL_WIDTH * scale, PORTAL_HEIGHT * scale, 1);
			mesh.position.z = side * (0.07 + progress * BEAM_LENGTH);
			group.add(mesh);
			beams.push({ material, baseOpacity: BEAM_OPACITY * Math.pow(1 - progress, 1.5) });
		}
	}

	const rayDefinitions = [
		{ z: 0.075, opacity: 0.55, speed: 1, scale: 1 },
		{ z: 0.085, opacity: 0.4, speed: -0.7, scale: 0.8 },
		{ z: -0.075, opacity: 0.45, speed: 0.8, scale: 0.9 }
	];

	const rays: GlowRay[] = rayDefinitions.map(({ z, opacity, speed, scale }) => {
		const material = glowMaterial(streakTexture);
		const mesh = new THREE.Mesh(geometry, material);

		mesh.scale.set(RAY_SIZE * scale, RAY_SIZE * scale, 1);
		mesh.position.z = z;
		group.add(mesh);

		return { mesh, material, baseOpacity: opacity, speed };
	});

	group.visible = false;

	return { group, beams, rays };
};

/** Builds the full portal logo, minus the falling grass block. */
export const createPortalLogoParts = (
	boxTexture: THREE.Texture,
	streakTexture: THREE.Texture
): PortalLogoParts => {
	const group = new THREE.Group();
	const portal = createPortal();
	const glow = createGlow(boxTexture, streakTexture);
	const light = new THREE.PointLight(GLOW_COLOR, 0, 1.2, 2);

	light.position.set(0, 0, 0.12);

	group.add(createFrame());
	group.add(portal.group);
	group.add(glow.group);
	group.add(light);

	return { group, portal: portal.group, portalMaterial: portal.material, glow, light };
};

export const createGrassBlock = (
	topTexture: THREE.Texture,
	sideTexture: THREE.Texture
): { mesh: THREE.Mesh; materials: THREE.MeshStandardMaterial[] } => {
	const side = new THREE.MeshStandardMaterial({
		map: sideTexture,
		roughness: 1,
		transparent: true,
		depthWrite: false
	});
	const top = new THREE.MeshStandardMaterial({
		map: topTexture,
		roughness: 1,
		transparent: true,
		depthWrite: false
	});

	// BoxGeometry face order: +x, -x, +y, -y, +z, -z.
	const materials = [side, side, top, top, side, side];
	const geometry = new THREE.BoxGeometry(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE);
	const mesh = new THREE.Mesh(geometry, materials);

	mesh.visible = false;
	mesh.rotation.order = 'YXZ';

	return { mesh, materials };
};
