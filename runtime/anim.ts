/**
 * Lớp hoạt hình của trang: mọi con số đều do lõi WebAssembly tính, JS chỉ ghi
 * style. Một vòng requestAnimationFrame duy nhất cho cả trang, tự ngủ khi
 * không còn gì chuyển động. Không có lõi thì set/to nhảy thẳng tới trạng thái
 * cuối, nên nội dung luôn hiện đủ.
 */
import { EASE, getCore } from './core';

export type Ease = keyof typeof EASE;

export interface Props {
  opacity?: number;
  x?: number;
  y?: number;
  xPercent?: number;
  yPercent?: number;
  scale?: number;
  rotate?: number;
  rotateX?: number;
  rotateY?: number;
  perspective?: number;
  clip?: number;
  volume?: number;
}

export interface Opts {
  duration?: number;
  delay?: number;
  ease?: Ease;
  onComplete?: () => void;
}

type State = Record<string, number>;

const DEFAULTS: State = {
  opacity: 1,
  x: 0,
  y: 0,
  xPercent: 0,
  yPercent: 0,
  scale: 1,
  rotate: 0,
  rotateX: 0,
  rotateY: 0,
  perspective: 0,
  clip: 0,
  volume: 0,
};

const TRANSFORM = new Set(['x', 'y', 'xPercent', 'yPercent', 'scale', 'rotate', 'rotateX', 'rotateY', 'perspective']);

interface Entry {
  el: HTMLElement;
  props: string[];
  onComplete?: (() => void) | undefined;
}

const states = new WeakMap<HTMLElement, State>();
const owned = new WeakMap<HTMLElement, Set<number>>();
const entries = new Map<number, Entry>();
const frameCbs = new Set<(now: number, dt: number) => boolean>();
let running = false;
let last = 0;

function stateOf(el: HTMLElement): State {
  let s = states.get(el);
  if (!s) {
    s = {};
    states.set(el, s);
  }
  return s;
}

function read(el: HTMLElement, prop: string): number {
  const s = stateOf(el);
  return s[prop] ?? DEFAULTS[prop] ?? 0;
}

function transformOf(s: State): string {
  let t = '';
  if (s.perspective) t += `perspective(${s.perspective}px) `;
  if (s.x || s.y) t += `translate3d(${s.x ?? 0}px, ${s.y ?? 0}px, 0) `;
  if (s.xPercent || s.yPercent) t += `translate(${s.xPercent ?? 0}%, ${s.yPercent ?? 0}%) `;
  if (s.rotate) t += `rotate(${s.rotate}deg) `;
  if (s.rotateX) t += `rotateX(${s.rotateX}deg) `;
  if (s.rotateY) t += `rotateY(${s.rotateY}deg) `;
  if (s.scale !== undefined && s.scale !== 1) t += `scale(${s.scale}) `;
  return t;
}

function flush(el: HTMLElement, props: string[]): void {
  const s = stateOf(el);
  let moved = false;
  for (const prop of props) {
    if (TRANSFORM.has(prop)) {
      moved = true;
    } else if (prop === 'opacity') {
      el.style.opacity = String(s.opacity);
    } else if (prop === 'clip') {
      const c = s.clip ?? 0;
      el.style.clipPath = c > 0.001 ? `inset(${c}% ${c}% ${c}% ${c}% round 2px)` : '';
    } else if (prop === 'volume') {
      (el as HTMLAudioElement).volume = Math.min(1, Math.max(0, s.volume ?? 0));
    }
  }
  if (moved) {
    const t = transformOf(s);
    el.style.transform = t;
  }
}

/** Ghi thẳng, không hoạt hình. */
export function set(el: HTMLElement, props: Props): void {
  const s = stateOf(el);
  const names = Object.keys(props);
  for (const name of names) s[name] = (props as State)[name]!;
  flush(el, names);
}

function track(el: HTMLElement, id: number): void {
  let ids = owned.get(el);
  if (!ids) {
    ids = new Set();
    owned.set(el, ids);
  }
  ids.add(id);
}

/** Dừng mọi tween đang chạy trên phần tử (giữ nguyên giá trị hiện tại). */
export function kill(el: HTMLElement): void {
  const ids = owned.get(el);
  if (!ids) return;
  const core = getCore();
  for (const id of ids) {
    core?.tween_kill(id);
    entries.delete(id);
  }
  ids.clear();
}

export function fromTo(el: HTMLElement, from: Props, to: Props, opts: Opts = {}): void {
  const core = getCore();
  const props = Array.from(new Set([...Object.keys(from), ...Object.keys(to)]));
  if (!core || props.length > 8) {
    set(el, { ...from, ...to });
    opts.onComplete?.();
    return;
  }
  const now = performance.now();
  const id = core.tween_add(
    now,
    (opts.delay ?? 0) * 1000,
    (opts.duration ?? 0.6) * 1000,
    EASE[opts.ease ?? 'out3'],
  );
  if (id < 0) {
    set(el, { ...from, ...to });
    opts.onComplete?.();
    return;
  }
  const s = stateOf(el);
  for (let i = 0; i < props.length; i += 1) {
    const prop = props[i]!;
    const a = (from as State)[prop] ?? read(el, prop);
    const b = (to as State)[prop] ?? read(el, prop);
    core.tween_channel(id, i, a, b);
    s[prop] = a;
  }
  flush(el, props);
  entries.set(id, { el, props, onComplete: opts.onComplete });
  track(el, id);
  wake();
}

export function to(el: HTMLElement, props: Props, opts: Opts = {}): void {
  fromTo(el, {}, props, opts);
}

export function from(el: HTMLElement, props: Props, opts: Opts = {}): void {
  const target: Props = {};
  for (const prop of Object.keys(props)) (target as State)[prop] = read(el, prop);
  fromTo(el, props, target, opts);
}

/** Chạy cb sau delay giây, theo đúng đồng hồ của vòng hoạt hình. */
export function after(delay: number, cb: () => void): void {
  const core = getCore();
  if (!core) {
    cb();
    return;
  }
  const id = core.tween_add(performance.now(), delay * 1000, 1, 0);
  if (id < 0) {
    cb();
    return;
  }
  entries.set(id, { el: document.documentElement, props: [], onComplete: cb });
  wake();
}

/** Đăng ký việc chạy mỗi khung; trả về true nếu còn cần khung tiếp theo. */
export function onFrame(cb: (now: number, dt: number) => boolean): () => void {
  frameCbs.add(cb);
  wake();
  return () => frameCbs.delete(cb);
}

function loop(now: number): void {
  const dt = Math.min((now - last) / 1000, 0.05);
  last = now;
  const core = getCore();
  let need = false;

  if (core && entries.size) {
    core.tween_tick(now);
    const out = core.tweenOut;
    const stride = core.tweenStride;
    for (const [id, entry] of entries) {
      const base = id * stride;
      const status = out[base + stride - 1]!;
      const s = stateOf(entry.el);
      for (let i = 0; i < entry.props.length; i += 1) s[entry.props[i]!] = out[base + i]!;
      if (entry.props.length) flush(entry.el, entry.props);
      if (status === 3 || status === 0) {
        entries.delete(id);
        owned.get(entry.el)?.delete(id);
        entry.onComplete?.();
      }
    }
    need = entries.size > 0;
  }

  for (const cb of frameCbs) {
    if (cb(now, dt)) need = true;
  }

  if (need) requestAnimationFrame(loop);
  else running = false;
}

/** Đánh thức vòng chạy (sau một sự kiện cuộn, chuột, mở thiệp...). */
export function wake(): void {
  if (running) return;
  running = true;
  last = performance.now();
  requestAnimationFrame(loop);
}
