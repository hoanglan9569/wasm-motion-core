/**
 * Loads the WebAssembly core: tweens, scroll tracks, smooth scrolling, tilt
 * damping, a countdown split and a petal-rain particle system.
 *
 * The module is inlined as base64, so a page needs no extra request and the
 * core also runs from file://. If WebAssembly is missing or the module fails
 * to instantiate, `loadCore` resolves to null and the page is expected to
 * skip its animations rather than break.
 */
import { MOTION_CORE_WASM } from './motion-core-wasm';

interface Exports {
  memory: WebAssembly.Memory;
  version(): number;
  petals_seed(v: number): void;
  petals_resize(w: number, h: number, dpr: number, count: number): number;
  petals_step(dt: number): number;
  petals_ptr(): number;
  petals_fields(): number;
  petals_max(): number;
  tween_add(now: number, delay: number, dur: number, ease: number): number;
  tween_channel(id: number, ch: number, from: number, to: number): void;
  tween_kill(id: number): void;
  tween_tick(now: number): number;
  tween_active(): number;
  tween_ptr(): number;
  tween_stride(): number;
  tween_slots(): number;
  motion_ptr(): number;
  motion_len(): number;
  motion_index(what: number): number;
  track_set(i: number, start: number, end: number, from: number, to: number, ease: number, scrub: number): void;
  track_clear(): void;
  scroll_tick(y: number, dt: number): void;
  tilt_target(x: number, y: number, tau: number): void;
  tilt_tick(dt: number): number;
  scroll_bounds(max: number, tau: number): void;
  scroll_set(y: number, jump: number): void;
  scroll_by(dy: number): void;
  scroll_step(dt: number): number;
  scroll_want(): number;
  countdown(totalSeconds: number): void;
}

export interface MotionCore extends Exports {
  /** Tween table: `tweenStride` floats per slot (values, then status). */
  tweenOut: Float32Array;
  tweenStride: number;
  /** Scroll tracks, tilt, smooth scroll position and countdown parts. */
  motionOut: Float32Array;
  motionTiltX: number;
  motionScroll: number;
  motionCountdown: number;
  /** Ready-to-draw canvas matrices for the petal rain. */
  petalsOut: Float32Array;
  petalsFieldCount: number;
}

/** Easing ids understood by the core. */
export const EASE = {
  none: 0,
  in2: 1,
  out2: 2,
  out3: 3,
  out4: 4,
  outSine: 5,
  inOut2: 6,
  back: 7,
} as const;

/** Tween status written in the last float of each slot. */
export const STATUS = { free: 0, waiting: 1, running: 2, done: 3 } as const;

export const CORE_VERSION = 2;

let core: MotionCore | null = null;
let loading: Promise<MotionCore | null> | null = null;

function decode(b64: string): Uint8Array {
  const raw = atob(b64);
  const bytes = new Uint8Array(raw.length);
  for (let i = 0; i < raw.length; i += 1) bytes[i] = raw.charCodeAt(i);
  return bytes;
}

/**
 * A WebAssembly instance's exports object is sealed, so the views are copied
 * onto a fresh object instead of assigned onto it (that throws in strict mode).
 */
function wrap(exports: Exports): MotionCore {
  const buffer = exports.memory.buffer;
  return Object.assign({} as Exports, exports, {
    tweenOut: new Float32Array(buffer, exports.tween_ptr(), exports.tween_slots() * exports.tween_stride()),
    tweenStride: exports.tween_stride(),
    motionOut: new Float32Array(buffer, exports.motion_ptr(), exports.motion_len()),
    motionTiltX: exports.motion_index(1),
    motionScroll: exports.motion_index(3),
    motionCountdown: exports.motion_index(4),
    petalsOut: new Float32Array(buffer, exports.petals_ptr(), exports.petals_max() * exports.petals_fields()),
    petalsFieldCount: exports.petals_fields(),
  });
}

/** Instantiate once; later calls return the same core. */
export function loadCore(options: { disabled?: boolean } = {}): Promise<MotionCore | null> {
  if (loading) return loading;
  loading = (async () => {
    if (options.disabled || typeof WebAssembly !== 'object') return null;
    try {
      const source = (await WebAssembly.instantiate(
        decode(MOTION_CORE_WASM) as BufferSource,
        {},
      )) as WebAssembly.WebAssemblyInstantiatedSource;
      const exports = source.instance.exports as unknown as Exports;
      if (exports.version() !== CORE_VERSION) return null;
      core = wrap(exports);
      return core;
    } catch (error) {
      console.warn('motion-core: falling back to no animation', error);
      return null;
    }
  })();
  return loading;
}

export function getCore(): MotionCore | null {
  return core;
}
