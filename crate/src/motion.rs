//! Scroll-driven tracks, smooth scrolling, the card tilt and the countdown.
//! All of it writes into one small output buffer the page reads each frame:
//!   0..TRACKS-1  track values
//!   TILT_X, TILT_Y, SCROLL, CD_D, CD_H, CD_M, CD_S

use crate::math::{clamp01, damp, ease};

pub const TRACKS: usize = 32;
pub const TILT_X: usize = TRACKS;
pub const TILT_Y: usize = TRACKS + 1;
pub const SCROLL: usize = TRACKS + 2;
pub const CD_D: usize = TRACKS + 3;
pub const OUT: usize = TRACKS + 8;

#[derive(Clone, Copy)]
struct Track {
    start: f32,
    end: f32,
    from: f32,
    to: f32,
    ease: u32,
    scrub: f32,
    value: f32,
    live: bool,
}

const BLANK: Track = Track {
    start: 0.0,
    end: 0.0,
    from: 0.0,
    to: 0.0,
    ease: 0,
    scrub: 0.0,
    value: 0.0,
    live: false,
};

pub struct Motion {
    tracks: [Track; TRACKS],
    pub out: [f32; OUT],
    tilt_tx: f32,
    tilt_ty: f32,
    tilt_tau: f32,
    scroll_target: f32,
    scroll_now: f32,
    scroll_max: f32,
    scroll_tau: f32,
}

pub const EMPTY: Motion = Motion {
    tracks: [BLANK; TRACKS],
    out: [0.0; OUT],
    tilt_tx: 0.0,
    tilt_ty: 0.0,
    tilt_tau: 0.0,
    scroll_target: 0.0,
    scroll_now: 0.0,
    scroll_max: 0.0,
    scroll_tau: 0.0,
};

/// Zero means "never set": every static starts at zero so the module carries
/// no data segment, and the defaults live here instead.
fn or_default(value: f32, fallback: f32) -> f32 {
    if value > 0.0 {
        value
    } else {
        fallback
    }
}

impl Motion {
    /// Map scroll range [start, end] px to [from, to], smoothed over `scrub` s.
    pub fn set_track(&mut self, i: usize, start: f32, end: f32, from: f32, to: f32, ease_kind: u32, scrub: f32) {
        if i >= TRACKS {
            return;
        }
        let keep = self.tracks[i].live;
        self.tracks[i] = Track {
            start,
            end: if end > start { end } else { start + 1.0 },
            from,
            to,
            ease: ease_kind,
            scrub,
            value: if keep { self.tracks[i].value } else { from },
            live: true,
        };
    }

    pub fn clear_tracks(&mut self) {
        for i in 0..TRACKS {
            self.tracks[i].live = false;
        }
    }

    pub fn scroll_tick(&mut self, scroll_y: f32, dt: f32) {
        for i in 0..TRACKS {
            if !self.tracks[i].live {
                continue;
            }
            let t = self.tracks[i];
            let p = clamp01((scroll_y - t.start) / (t.end - t.start));
            let target = t.from + (t.to - t.from) * ease(t.ease, p);
            let v = if t.scrub > 0.0 {
                t.value + (target - t.value) * damp(dt, t.scrub)
            } else {
                target
            };
            self.tracks[i].value = v;
            self.out[i] = v;
        }
    }

    pub fn tilt_target(&mut self, x: f32, y: f32, tau: f32) {
        self.tilt_tx = x;
        self.tilt_ty = y;
        if tau > 0.0 {
            self.tilt_tau = tau;
        }
    }

    /// Returns 1 while the tilt is still moving, 0 once it has settled.
    pub fn tilt_tick(&mut self, dt: f32) -> u32 {
        let k = damp(dt, or_default(self.tilt_tau, 0.12));
        let x = self.out[TILT_X] + (self.tilt_tx - self.out[TILT_X]) * k;
        let y = self.out[TILT_Y] + (self.tilt_ty - self.out[TILT_Y]) * k;
        self.out[TILT_X] = x;
        self.out[TILT_Y] = y;
        let dx = x - self.tilt_tx;
        let dy = y - self.tilt_ty;
        if dx * dx + dy * dy < 1.0e-6 {
            self.out[TILT_X] = self.tilt_tx;
            self.out[TILT_Y] = self.tilt_ty;
            0
        } else {
            1
        }
    }

    pub fn scroll_bounds(&mut self, max: f32, tau: f32) {
        self.scroll_max = if max > 0.0 { max } else { 0.0 };
        if tau > 0.0 {
            self.scroll_tau = tau;
        }
        if self.scroll_target > self.scroll_max {
            self.scroll_target = self.scroll_max;
        }
    }

    pub fn scroll_set(&mut self, y: f32, jump: u32) {
        self.scroll_target = clamp_to(y, self.scroll_max);
        if jump != 0 {
            self.scroll_now = self.scroll_target;
            self.out[SCROLL] = self.scroll_now;
        }
    }

    pub fn scroll_by(&mut self, dy: f32) {
        self.scroll_target = clamp_to(self.scroll_target + dy, self.scroll_max);
    }

    /// Lerp the page towards the wheel target; 0 once it has arrived.
    pub fn scroll_step(&mut self, dt: f32) -> u32 {
        let k = damp(dt, or_default(self.scroll_tau, 0.13));
        let next = self.scroll_now + (self.scroll_target - self.scroll_now) * k;
        self.scroll_now = next;
        self.out[SCROLL] = next;
        let gap = next - self.scroll_target;
        if gap * gap < 0.01 {
            self.scroll_now = self.scroll_target;
            self.out[SCROLL] = self.scroll_target;
            0
        } else {
            1
        }
    }

    pub fn scroll_target(&self) -> f32 {
        self.scroll_target
    }

    /// Split the seconds left into days, hours, minutes, seconds.
    pub fn countdown(&mut self, total_seconds: i32) {
        let t = if total_seconds > 0 { total_seconds } else { 0 };
        self.out[CD_D] = (t / 86_400) as f32;
        self.out[CD_D + 1] = ((t % 86_400) / 3_600) as f32;
        self.out[CD_D + 2] = ((t % 3_600) / 60) as f32;
        self.out[CD_D + 3] = (t % 60) as f32;
    }
}

fn clamp_to(y: f32, max: f32) -> f32 {
    if y < 0.0 {
        0.0
    } else if y > max {
        max
    } else {
        y
    }
}
