//! Tween engine: every animated number on the page lives here.
//! A slot holds up to `CH` channels (opacity, y, scale ... the page decides
//! what each index means) and writes its current values into `out` each tick.
//! Output per slot: `CH` values then a status float
//!   0 free, 1 waiting out its delay, 2 running, 3 finished on this tick.

use crate::math::{clamp01, ease};

pub const CH: usize = 8;
pub const SLOTS: usize = 768;
pub const STRIDE: usize = CH + 1;

#[derive(Clone, Copy)]
struct Slot {
    from: [f32; CH],
    to: [f32; CH],
    start: f32,
    delay: f32,
    dur: f32,
    ease: u32,
    live: bool,
}

const BLANK: Slot = Slot {
    from: [0.0; CH],
    to: [0.0; CH],
    start: 0.0,
    delay: 0.0,
    dur: 0.0,
    ease: 0,
    live: false,
};

pub struct Tweens {
    slots: [Slot; SLOTS],
    pub out: [f32; SLOTS * STRIDE],
    cursor: usize,
    active: u32,
}

pub const EMPTY: Tweens = Tweens {
    slots: [BLANK; SLOTS],
    out: [0.0; SLOTS * STRIDE],
    cursor: 0,
    active: 0,
};

impl Tweens {
    /// Claim a slot. Returns the id, or -1 when the pool is full.
    pub fn add(&mut self, now: f32, delay: f32, dur: f32, ease_kind: u32) -> i32 {
        for _ in 0..SLOTS {
            let i = self.cursor;
            self.cursor = (self.cursor + 1) % SLOTS;
            if !self.slots[i].live {
                self.slots[i] = Slot {
                    from: [0.0; CH],
                    to: [0.0; CH],
                    start: now,
                    delay,
                    dur: if dur > 0.0 { dur } else { 0.0001 },
                    ease: ease_kind,
                    live: true,
                };
                self.out[i * STRIDE + CH] = 1.0;
                self.active += 1;
                return i as i32;
            }
        }
        -1
    }

    pub fn channel(&mut self, id: usize, ch: usize, from: f32, to: f32) {
        if id >= SLOTS || ch >= CH {
            return;
        }
        self.slots[id].from[ch] = from;
        self.slots[id].to[ch] = to;
        self.out[id * STRIDE + ch] = from;
    }

    pub fn kill(&mut self, id: usize) {
        if id >= SLOTS || !self.slots[id].live {
            return;
        }
        self.slots[id].live = false;
        self.out[id * STRIDE + CH] = 0.0;
        self.active = self.active.saturating_sub(1);
    }

    /// Advance every live slot to `now` (ms) and write its channel values.
    pub fn tick(&mut self, now: f32) -> u32 {
        for i in 0..SLOTS {
            if !self.slots[i].live {
                continue;
            }
            let s = self.slots[i];
            let elapsed = now - s.start - s.delay;
            let o = i * STRIDE;
            if elapsed < 0.0 {
                for c in 0..CH {
                    self.out[o + c] = s.from[c];
                }
                self.out[o + CH] = 1.0;
                continue;
            }
            let t = clamp01(elapsed / s.dur);
            let e = ease(s.ease, t);
            for c in 0..CH {
                self.out[o + c] = s.from[c] + (s.to[c] - s.from[c]) * e;
            }
            if t >= 1.0 {
                self.slots[i].live = false;
                self.out[o + CH] = 3.0;
                self.active = self.active.saturating_sub(1);
            } else {
                self.out[o + CH] = 2.0;
            }
        }
        self.active
    }

    pub fn active(&self) -> u32 {
        self.active
    }
}
