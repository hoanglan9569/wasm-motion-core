# wasm-motion-core

Every animated number of a web page, computed in WebAssembly. The page keeps
the DOM and the canvas; the core keeps the maths.

- **5 KB module** (`crate/` in Rust, `no_std`, no allocator, no wasm-bindgen),
  inlined as base64 so a page costs no extra request and works over `file://`.
- **Tween engine**: 768 slots, 8 channels each, 8 easing curves, one table the
  page reads as a `Float32Array`.
- **Scroll tracks**: map a scroll range to a value with optional scrub damping
  (parallax, progress lines).
- **Smooth scroll**: Lenis-style exponential approach, wheel target and bounds.
- **Tilt damping** for pointer-driven card tilts.
- **Countdown**: seconds split into days / hours / minutes / seconds.
- **Petal rain**: particle simulation that outputs ready canvas matrices in
  device pixels, so drawing is `setTransform` + `fill` and nothing else.

## Why not an existing library

Checked on 2026-11-05 before writing a line:

| Candidate | State | Why not |
| --- | --- | --- |
| [tween-axis](https://www.npmjs.com/package/tween-axis) | 2.2.0, 78 downloads in the last month, 209 KB unpacked | AssemblyScript core built for react-voodoo's additive axis model; no scroll, particles or DOM writer |
| [spanda](https://github.com/aarambh-darshan/spanda) | archived 2026-05-07, 17 stars | unmaintained, superseded |
| [animato](https://github.com/AarambhDevHub/animato) | 1.7.3, published 2026-10-21, 2,214 downloads total | young single-author crate; its `wasm-dom` path wants a Rust-side app with wasm-bindgen + web-sys, which costs far more than 5 KB |
| [Rive](https://rive.app) web runtime | mature | ~200 KB gzipped: a whole renderer for a page that needs arithmetic |

## Build

```sh
rustup target add wasm32-unknown-unknown   # once
bun run build                              # cargo build + base64 emit
bun run typecheck
```

`bun run build` writes `runtime/motion-core.wasm` and the generated
`runtime/motion-core-wasm.ts`. Both are committed, so a consumer never needs
the Rust toolchain.

## Use

```ts
import { loadCore } from './runtime/core';
import * as anim from './runtime/anim';

await loadCore();                       // null when WebAssembly is unavailable
anim.from(el, { opacity: 0, y: 40 }, { duration: 1.1, ease: 'out3' });
```

`anim` owns one `requestAnimationFrame` loop for the whole page and sleeps as
soon as nothing moves. With no core, `set`/`to`/`from` jump straight to the end
state, so content is always visible.

## Layout

```
crate/src/lib.rs      exported functions, static state
crate/src/tween.rs    tween slots and the tick
crate/src/motion.rs   scroll tracks, smooth scroll, tilt, countdown
crate/src/petals.rs   petal simulation and canvas matrices
crate/src/math.rs     sin/cos/easing without libm
runtime/core.ts       loader and typed view over linear memory
runtime/anim.ts       DOM writer: style strings, one rAF loop
```

Every static starts at zero so the module carries no data segment: that alone
took the build from 104 KB to 4.6 KB.

## Pitfalls

- Sequential tweens on one element: both write during their delay, pinning the element at the later tween's from-value unless the `hold` flag is set (do not touch the element until delay is over).

## Consumers

- [wedding-lan-thanh](https://github.com/hoanglan9569/wedding-lan-thanh) vendors
  `runtime/` into `src/vendor/motion-core/` via its `bun run sync:core`.

MIT.
