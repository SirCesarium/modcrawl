import * as THREE from 'three';

/** Frame proportions, in world units. */
export const FRAME_WIDTH = 0.625;
export const FRAME_HEIGHT = 0.75;
export const FRAME_DEPTH = 0.125;
export const FRAME_HOLE_WIDTH = 0.375;
export const FRAME_HOLE_HEIGHT = 0.5;

/** Portal surface: a 2x3 grid sampled from a vertical sprite sheet. */
export const PORTAL_WIDTH = 0.385;
export const PORTAL_HEIGHT = 0.51;
export const PORTAL_COLUMNS = 2;
export const PORTAL_ROWS = 3;
export const PORTAL_FRAME_MS = 100;

/** Grass block that slides into the portal. */
export const BLOCK_SIZE = (PORTAL_WIDTH / 2) * 1.2;
export const BLOCK_DELAY = 0.45;
export const BLOCK_TRAVEL_DURATION = 0.9;
export const BLOCK_ABSORB_AT = 0.68;
export const BLOCK_FADE_IN_END = 0.45;
export const BLOCK_FADE_START = 0.6;
export const BLOCK_START_X = -0.6;
export const BLOCK_END_X = 0.22;
export const BLOCK_Y = 0.02;
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
export const MAX_YAW = THREE.MathUtils.degToRad(20);
export const MAX_PITCH = THREE.MathUtils.degToRad(15);
export const ROTATION_DAMP = 5;
export const PORTAL_DAMP = 6;
export const FIT_MARGIN = 1.5;

/** Assets. */
export const PORTAL_TEXTURE = '/nether_portal.png';
export const GRASS_TOP_TEXTURE = '/grass_block_top.png';
export const GRASS_SIDE_TEXTURE = '/grass_block_side.png';
