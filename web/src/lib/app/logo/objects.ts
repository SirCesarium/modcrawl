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
	PORTAL_HEIGHT,
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
	frameMaterial: THREE.MeshStandardMaterial;
	glow: Glow;
	light: THREE.PointLight;
}

const createFrameGeometry = (): THREE.BufferGeometry => {
	const x0 = -FRAME_WIDTH / 2;
	const x1 = -FRAME_HOLE_WIDTH / 2;
	const x2 = FRAME_HOLE_WIDTH / 2;
	const x3 = FRAME_WIDTH / 2;

	const y0 = -FRAME_HEIGHT / 2;
	const y1 = -FRAME_HOLE_HEIGHT / 2;
	const y2 = FRAME_HOLE_HEIGHT / 2;
	const y3 = FRAME_HEIGHT / 2;

	const zf = FRAME_DEPTH / 2;
	const zb = -FRAME_DEPTH / 2;

	const positions: number[] = [];
	const normals: number[] = [];
	const uvs: number[] = [];
	const indices: number[] = [];

	const addQuad = (
		p0: [number, number, number],
		p1: [number, number, number],
		p2: [number, number, number],
		p3: [number, number, number],
		uv0: [number, number],
		uv1: [number, number],
		uv2: [number, number],
		uv3: [number, number],
		norm: [number, number, number]
	): void => {
		const baseIndex = positions.length / 3;
		positions.push(...p0, ...p1, ...p2, ...p3);
		normals.push(...norm, ...norm, ...norm, ...norm);
		uvs.push(...uv0, ...uv1, ...uv2, ...uv3);
		indices.push(baseIndex, baseIndex + 1, baseIndex + 2, baseIndex, baseIndex + 2, baseIndex + 3);
	};

	// 1. FRONT FACE (+Z, norm = [0, 0, 1])
	// Left pillar: x0..x1, y0..y3 (1 x 5 blocks)
	addQuad(
		[x0, y0, zf],
		[x1, y0, zf],
		[x1, y3, zf],
		[x0, y3, zf],
		[0, 0],
		[1, 0],
		[1, 5],
		[0, 5],
		[0, 0, 1]
	);
	// Right pillar: x2..x3, y0..y3 (1 x 5 blocks)
	addQuad(
		[x2, y0, zf],
		[x3, y0, zf],
		[x3, y3, zf],
		[x2, y3, zf],
		[0, 0],
		[1, 0],
		[1, 5],
		[0, 5],
		[0, 0, 1]
	);
	// Top beam: x1..x2, y2..y3 (2 x 1 blocks)
	addQuad(
		[x1, y2, zf],
		[x2, y2, zf],
		[x2, y3, zf],
		[x1, y3, zf],
		[0, 0],
		[2, 0],
		[2, 1],
		[0, 1],
		[0, 0, 1]
	);
	// Bottom beam: x1..x2, y0..y1 (2 x 1 blocks)
	addQuad(
		[x1, y0, zf],
		[x2, y0, zf],
		[x2, y1, zf],
		[x1, y1, zf],
		[0, 0],
		[2, 0],
		[2, 1],
		[0, 1],
		[0, 0, 1]
	);

	// 2. BACK FACE (-Z, norm = [0, 0, -1])
	// Left pillar: x1..x0, y0..y3 (viewed from back, CCW order)
	addQuad(
		[x1, y0, zb],
		[x0, y0, zb],
		[x0, y3, zb],
		[x1, y3, zb],
		[0, 0],
		[1, 0],
		[1, 5],
		[0, 5],
		[0, 0, -1]
	);
	// Right pillar: x3..x2, y0..y3
	addQuad(
		[x3, y0, zb],
		[x2, y0, zb],
		[x2, y3, zb],
		[x3, y3, zb],
		[0, 0],
		[1, 0],
		[1, 5],
		[0, 5],
		[0, 0, -1]
	);
	// Top beam: x2..x1, y2..y3
	addQuad(
		[x2, y2, zb],
		[x1, y2, zb],
		[x1, y3, zb],
		[x2, y3, zb],
		[0, 0],
		[2, 0],
		[2, 1],
		[0, 1],
		[0, 0, -1]
	);
	// Bottom beam: x2..x1, y0..y1
	addQuad(
		[x2, y0, zb],
		[x1, y0, zb],
		[x1, y1, zb],
		[x2, y1, zb],
		[0, 0],
		[2, 0],
		[2, 1],
		[0, 1],
		[0, 0, -1]
	);

	// 3. OUTER WALLS (4 quads)
	// Outer Left (-X, norm = [-1, 0, 0]): y0..y3, zb..zf (1 x 5 blocks)
	addQuad(
		[x0, y0, zb],
		[x0, y0, zf],
		[x0, y3, zf],
		[x0, y3, zb],
		[0, 0],
		[1, 0],
		[1, 5],
		[0, 5],
		[-1, 0, 0]
	);
	// Outer Right (+X, norm = [1, 0, 0]): y0..y3, zf..zb (1 x 5 blocks)
	addQuad(
		[x3, y0, zf],
		[x3, y0, zb],
		[x3, y3, zb],
		[x3, y3, zf],
		[0, 0],
		[1, 0],
		[1, 5],
		[0, 5],
		[1, 0, 0]
	);
	// Outer Top (+Y, norm = [0, 1, 0]): x0..x3, zf..zb (4 x 1 blocks)
	addQuad(
		[x0, y3, zf],
		[x3, y3, zf],
		[x3, y3, zb],
		[x0, y3, zb],
		[0, 0],
		[4, 0],
		[4, 1],
		[0, 1],
		[0, 1, 0]
	);
	// Outer Bottom (-Y, norm = [0, -1, 0]): x0..x3, zb..zf (4 x 1 blocks)
	addQuad(
		[x0, y0, zb],
		[x3, y0, zb],
		[x3, y0, zf],
		[x0, y0, zf],
		[0, 0],
		[4, 0],
		[4, 1],
		[0, 1],
		[0, -1, 0]
	);

	// 4. INNER HOLE WALLS (4 quads)
	// Inner Left wall (facing +X into the hole, norm = [1, 0, 0]): y1..y2, zf..zb (1 x 3 blocks)
	addQuad(
		[x1, y1, zf],
		[x1, y1, zb],
		[x1, y2, zb],
		[x1, y2, zf],
		[0, 0],
		[1, 0],
		[1, 3],
		[0, 3],
		[1, 0, 0]
	);
	// Inner Right wall (facing -X into the hole, norm = [-1, 0, 0]): y1..y2, zb..zf (1 x 3 blocks)
	addQuad(
		[x2, y1, zb],
		[x2, y1, zf],
		[x2, y2, zf],
		[x2, y2, zb],
		[0, 0],
		[1, 0],
		[1, 3],
		[0, 3],
		[-1, 0, 0]
	);
	// Inner Top wall (ceiling of hole, facing -Y, norm = [0, -1, 0]): x1..x2, zb..zf (2 x 1 blocks)
	addQuad(
		[x1, y2, zb],
		[x2, y2, zb],
		[x2, y2, zf],
		[x1, y2, zf],
		[0, 0],
		[2, 0],
		[2, 1],
		[0, 1],
		[0, -1, 0]
	);
	// Inner Bottom wall (floor of hole, facing +Y, norm = [0, 1, 0]): x1..x2, zf..zb (2 x 1 blocks)
	addQuad(
		[x1, y1, zf],
		[x2, y1, zf],
		[x2, y1, zb],
		[x1, y1, zb],
		[0, 0],
		[2, 0],
		[2, 1],
		[0, 1],
		[0, 1, 0]
	);

	const geometry = new THREE.BufferGeometry();
	geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
	geometry.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
	geometry.setAttribute('uv', new THREE.Float32BufferAttribute(uvs, 2));
	geometry.setIndex(indices);

	return geometry;
};

const createFrame = (): { group: THREE.Group; material: THREE.MeshStandardMaterial } => {
	const material = new THREE.MeshStandardMaterial({ color: 0xffffff, metalness: 0, roughness: 1 });
	const geometry = createFrameGeometry();
	const frameMesh = new THREE.Mesh(geometry, material);
	const group = new THREE.Group();

	group.add(frameMesh);

	return { group, material };
};

const createPortal = (): { group: THREE.Group; material: THREE.MeshBasicMaterial } => {
	const material = new THREE.MeshBasicMaterial({
		transparent: true,
		opacity: 0,
		side: THREE.DoubleSide,
		depthWrite: false
	});

	const geometry = new THREE.PlaneGeometry(PORTAL_WIDTH, PORTAL_HEIGHT);
	const mesh = new THREE.Mesh(geometry, material);
	const group = new THREE.Group();

	group.add(mesh);

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

/** Builds the full portal logo, minus the grass block. */
export const createPortalLogoParts = (
	boxTexture: THREE.Texture,
	streakTexture: THREE.Texture
): PortalLogoParts => {
	const group = new THREE.Group();
	const portal = createPortal();
	const frame = createFrame();
	const glow = createGlow(boxTexture, streakTexture);
	const light = new THREE.PointLight(GLOW_COLOR, 0, 1.2, 2);

	light.position.set(0, 0, 0.12);

	group.add(frame.group);
	group.add(portal.group);
	group.add(glow.group);
	group.add(light);

	return {
		group,
		portal: portal.group,
		portalMaterial: portal.material,
		frameMaterial: frame.material,
		glow,
		light
	};
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
