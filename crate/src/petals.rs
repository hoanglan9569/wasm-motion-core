//! Petal rain: position, spin and the ready-made canvas matrix per petal.
//! Output per petal (`FIELDS` floats), already in device pixels:
//!   0 a, 1 b, 2 c, 3 d, 4 e, 5 f, 6 alpha, 7 colour index

use crate::math::{abs, cos, fract_turns, sin, TAU};
use crate::rng::Rng;

pub const MAX: usize = 96;
pub const FIELDS: usize = 8;
const COLORS: f32 = 4.0;

#[derive(Clone, Copy)]
struct Petal {
    x: f32,
    y: f32,
    size: f32,
    speed: f32,
    drift: f32,
    spin: f32,
    angle: f32,
    alpha: f32,
    hue: f32,
}

const BLANK: Petal = Petal {
    x: 0.0,
    y: 0.0,
    size: 0.0,
    speed: 0.0,
    drift: 0.0,
    spin: 0.0,
    angle: 0.0,
    alpha: 0.0,
    hue: 0.0,
};

pub struct Petals {
    petals: [Petal; MAX],
    pub out: [f32; MAX * FIELDS],
    count: usize,
    w: f32,
    h: f32,
    dpr: f32,
}

pub const EMPTY: Petals = Petals {
    petals: [BLANK; MAX],
    out: [0.0; MAX * FIELDS],
    count: 0,
    w: 0.0,
    h: 0.0,
    dpr: 0.0,
};

/// Petal count per viewport width, the page's own breakpoints.
pub fn count_for(width: f32) -> u32 {
    if width < 620.0 {
        12
    } else if width < 1100.0 {
        18
    } else {
        26
    }
}

impl Petals {
    fn spawn(&mut self, rng: &mut Rng, top: bool) -> Petal {
        let scale = if self.w < 620.0 { 0.72 } else { 1.0 };
        Petal {
            x: rng.next() * self.w,
            y: if top {
                -20.0 - rng.next() * self.h * 0.4
            } else {
                rng.next() * self.h
            },
            size: (4.0 + rng.next() * 8.0) * scale,
            speed: 14.0 + rng.next() * 26.0,
            drift: (rng.next() - 0.5) * 26.0,
            spin: (rng.next() - 0.5) * 1.4,
            angle: rng.next() * TAU,
            alpha: 0.18 + rng.next() * 0.3,
            hue: (rng.next() * COLORS) as i32 as f32,
        }
    }

    pub fn resize(&mut self, rng: &mut Rng, width: f32, height: f32, dpr: f32, count: u32) -> u32 {
        self.w = width;
        self.h = height;
        self.dpr = if dpr > 0.0 { dpr } else { 1.0 };
        let wanted = if count == 0 { count_for(width) } else { count } as usize;
        self.count = if wanted > MAX { MAX } else { wanted };
        for i in 0..self.count {
            self.petals[i] = self.spawn(rng, false);
        }
        self.count as u32
    }

    pub fn step(&mut self, rng: &mut Rng, dt: f32) -> u32 {
        let h = self.h;
        let dpr = if self.dpr > 0.0 { self.dpr } else { 1.0 };
        for i in 0..self.count {
            let mut p = self.petals[i];
            p.y += p.speed * dt;
            p.x += sin(p.angle) * p.drift * dt;
            p.angle += p.spin * dt;
            if p.y - p.size > h {
                p = self.spawn(rng, true);
            }
            if p.angle > TAU || p.angle < -TAU {
                p.angle = fract_turns(p.angle);
            }
            self.petals[i] = p;

            let squash = abs(cos(p.angle * 0.85));
            let squash = if squash < 0.3 { 0.3 } else { squash };
            let c = cos(p.angle);
            let n = sin(p.angle);
            let sx = p.size * squash * dpr;
            let sy = p.size * dpr;
            let o = i * FIELDS;
            self.out[o] = c * sx;
            self.out[o + 1] = n * sx;
            self.out[o + 2] = -n * sy;
            self.out[o + 3] = c * sy;
            self.out[o + 4] = p.x * dpr;
            self.out[o + 5] = p.y * dpr;
            self.out[o + 6] = p.alpha;
            self.out[o + 7] = p.hue;
        }
        self.count as u32
    }
}
