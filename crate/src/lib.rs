//! The wedding page's maths core, compiled to WebAssembly.
//!
//! No std, no allocator, no wasm-bindgen: one linear memory, static state and
//! three output buffers the page reads as Float32Array views. The page keeps
//! the DOM and the canvas calls; every number it animates is computed here.
#![no_std]

mod math;
mod motion;
mod petals;
mod rng;
mod tween;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

struct State {
    rng: rng::Rng,
    petals: petals::Petals,
    tweens: tween::Tweens,
    motion: motion::Motion,
}

static mut STATE: State = State {
    rng: rng::Rng::new(0),
    petals: petals::EMPTY,
    tweens: tween::EMPTY,
    motion: motion::EMPTY,
};

fn state() -> &'static mut State {
    unsafe { &mut *core::ptr::addr_of_mut!(STATE) }
}

/// Build marker the page reads back to prove the wasm core is live.
#[no_mangle]
pub extern "C" fn version() -> u32 {
    2
}

// ---------------------------------------------------------------- petal rain

#[no_mangle]
pub extern "C" fn petals_seed(value: u32) {
    state().rng.seed(value);
}

#[no_mangle]
pub extern "C" fn petals_resize(width: f32, height: f32, dpr: f32, count: u32) -> u32 {
    let s = state();
    s.petals.resize(&mut s.rng, width, height, dpr, count)
}

#[no_mangle]
pub extern "C" fn petals_step(dt: f32) -> u32 {
    let s = state();
    s.petals.step(&mut s.rng, dt)
}

#[no_mangle]
pub extern "C" fn petals_ptr() -> u32 {
    state().petals.out.as_ptr() as u32
}

#[no_mangle]
pub extern "C" fn petals_fields() -> u32 {
    petals::FIELDS as u32
}

#[no_mangle]
pub extern "C" fn petals_max() -> u32 {
    petals::MAX as u32
}

// -------------------------------------------------------------------- tweens

#[no_mangle]
pub extern "C" fn tween_add(now: f32, delay: f32, dur: f32, ease_kind: u32) -> i32 {
    state().tweens.add(now, delay, dur, ease_kind)
}

#[no_mangle]
pub extern "C" fn tween_channel(id: u32, ch: u32, from: f32, to: f32) {
    state().tweens.channel(id as usize, ch as usize, from, to);
}

#[no_mangle]
pub extern "C" fn tween_kill(id: u32) {
    state().tweens.kill(id as usize);
}

#[no_mangle]
pub extern "C" fn tween_tick(now: f32) -> u32 {
    state().tweens.tick(now)
}

#[no_mangle]
pub extern "C" fn tween_active() -> u32 {
    state().tweens.active()
}

#[no_mangle]
pub extern "C" fn tween_ptr() -> u32 {
    state().tweens.out.as_ptr() as u32
}

#[no_mangle]
pub extern "C" fn tween_stride() -> u32 {
    tween::STRIDE as u32
}

#[no_mangle]
pub extern "C" fn tween_slots() -> u32 {
    tween::SLOTS as u32
}

// -------------------------------------------------- scroll, tilt, countdown

#[no_mangle]
pub extern "C" fn motion_ptr() -> u32 {
    state().motion.out.as_ptr() as u32
}

#[no_mangle]
pub extern "C" fn motion_len() -> u32 {
    motion::OUT as u32
}

#[no_mangle]
pub extern "C" fn motion_index(what: u32) -> u32 {
    match what {
        1 => motion::TILT_X as u32,
        2 => motion::TILT_Y as u32,
        3 => motion::SCROLL as u32,
        4 => motion::CD_D as u32,
        _ => motion::TRACKS as u32,
    }
}

#[no_mangle]
pub extern "C" fn track_set(i: u32, start: f32, end: f32, from: f32, to: f32, ease_kind: u32, scrub: f32) {
    state().motion.set_track(i as usize, start, end, from, to, ease_kind, scrub);
}

#[no_mangle]
pub extern "C" fn track_clear() {
    state().motion.clear_tracks();
}

#[no_mangle]
pub extern "C" fn scroll_tick(scroll_y: f32, dt: f32) {
    state().motion.scroll_tick(scroll_y, dt);
}

#[no_mangle]
pub extern "C" fn tilt_target(x: f32, y: f32, tau: f32) {
    state().motion.tilt_target(x, y, tau);
}

#[no_mangle]
pub extern "C" fn tilt_tick(dt: f32) -> u32 {
    state().motion.tilt_tick(dt)
}

#[no_mangle]
pub extern "C" fn scroll_bounds(max: f32, tau: f32) {
    state().motion.scroll_bounds(max, tau);
}

#[no_mangle]
pub extern "C" fn scroll_set(y: f32, jump: u32) {
    state().motion.scroll_set(y, jump);
}

#[no_mangle]
pub extern "C" fn scroll_by(dy: f32) {
    state().motion.scroll_by(dy);
}

#[no_mangle]
pub extern "C" fn scroll_step(dt: f32) -> u32 {
    state().motion.scroll_step(dt)
}

#[no_mangle]
pub extern "C" fn scroll_want() -> f32 {
    state().motion.scroll_target()
}

#[no_mangle]
pub extern "C" fn countdown(total_seconds: i32) {
    state().motion.countdown(total_seconds);
}
