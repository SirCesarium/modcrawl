import * as THREE from 'three';

/** One Minecraft block in world units. */
export const FRAME_BLOCK = 0.125;

/** Nether frame: two 5-block pillars with a 2-block gap between them. */
export const FRAME_WIDTH = FRAME_BLOCK * 4;
export const FRAME_HEIGHT = FRAME_BLOCK * 5;
export const FRAME_DEPTH = FRAME_BLOCK;
export const FRAME_HOLE_WIDTH = FRAME_BLOCK * 2;
export const FRAME_HOLE_HEIGHT = FRAME_BLOCK * 3;

/** Portal surface: a 2x3 grid sampled from a vertical sprite sheet. */
export const PORTAL_WIDTH = FRAME_HOLE_WIDTH;
export const PORTAL_HEIGHT = FRAME_HOLE_HEIGHT;
export const PORTAL_COLUMNS = 2;
export const PORTAL_ROWS = 3;
export const PORTAL_FRAME_MS = 100;

/** Hover sequence, in seconds. */
export const FRAME_FADE_DURATION = 0.35;
export const PORTAL_FADE_DELAY = 0.55;

/** Grass block that falls into the portal. */
export const BLOCK_SIZE = FRAME_BLOCK;
export const BLOCK_DELAY = 1.1;
export const BLOCK_TRAVEL_DURATION = 1.65;
export const BLOCK_ABSORB_AT = 0.9;
export const BLOCK_FADE_IN_END = 0.35;
export const BLOCK_FADE_START = 0.72;
export const BLOCK_START_X = 0;
export const BLOCK_END_X = 0;
export const BLOCK_START_Y = 0.55;
export const BLOCK_END_Y = 0;
export const BLOCK_START_Z = 0.2;
export const BLOCK_END_Z = 0;

/** Portal glow. */
export const GLOW_COLOR = 0xb35cff;
export const LIGHT_MAX = 0.12;
export const BEAM_LAYERS = 8;
export const BEAM_LENGTH = 0.6;
export const BEAM_SPREAD = 0.9;
export const BEAM_OPACITY = 0.1;
export const RAY_SIZE = 1.1;
export const RAY_SPIN = 0.1;

/** Interaction and camera. */
export const BASE_YAW = THREE.MathUtils.degToRad(-25);
export const MOBILE_BASE_YAW = THREE.MathUtils.degToRad(-35);
export const MAX_YAW = THREE.MathUtils.degToRad(20);
export const MAX_PITCH = THREE.MathUtils.degToRad(15);
export const ROTATION_DAMP = 5;
export const PORTAL_DAMP = 6;
export const FIT_MARGIN = 1.5;

/** Assets. */
export const PORTAL_TEXTURE = '/nether_portal.png';
export const GRASS_TOP_TEXTURE = '/grass_block_top.png';
export const GRASS_SIDE_TEXTURE = '/grass_block_side.png';
export const OBSIDIAN_TEXTURE = '/obsidian.png';
