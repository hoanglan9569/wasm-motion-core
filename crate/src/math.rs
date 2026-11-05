//! Trig and easing without libm: the module must link with no dependency.

pub const PI: f32 = core::f32::consts::PI;
pub const TAU: f32 = PI * 2.0;

pub fn abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub fn clamp01(x: f32) -> f32 {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

/// Wrap into (-TAU, TAU) without a float remainder (no fmodf to link).
pub fn fract_turns(x: f32) -> f32 {
    let turns = x * (1.0 / TAU);
    let whole = turns as i32 as f32;
    (turns - whole) * TAU
}

/// Bhaskara sine with the 0.775/0.225 correction: |err| < 1e-3.
pub fn sin(x: f32) -> f32 {
    let mut a = fract_turns(x);
    if a > PI {
        a -= TAU;
    } else if a < -PI {
        a += TAU;
    }
    let y = (4.0 / PI) * a - (4.0 / (PI * PI)) * a * abs(a);
    0.775 * y + 0.225 * y * abs(y)
}

pub fn cos(x: f32) -> f32 {
    sin(x + PI * 0.5)
}

/// Easing ids shared with the page: 0 linear, 1 power2.in, 2 power2.out,
/// 3 power3.out, 4 power4.out, 5 sine.out, 6 power2.inOut, 7 back.out.
pub fn ease(kind: u32, t: f32) -> f32 {
    let t = clamp01(t);
    let inv = 1.0 - t;
    match kind {
        1 => t * t,
        2 => 1.0 - inv * inv,
        3 => 1.0 - inv * inv * inv,
        4 => 1.0 - inv * inv * inv * inv,
        5 => sin(t * PI * 0.5),
        6 => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - 2.0 * inv * inv
            }
        }
        // back.out: overshoots past the target near the end, never before the start
        7 => {
            let c = 1.70158;
            1.0 - (c + 1.0) * inv * inv * inv + c * inv * inv
        }
        _ => t,
    }
}

/// Frame-rate independent approach factor for a damping time in seconds.
pub fn damp(dt: f32, tau: f32) -> f32 {
    if tau <= 0.0 {
        return 1.0;
    }
    let k = dt / tau;
    let f = 1.0 - 1.0 / (1.0 + k + 0.5 * k * k);
    clamp01(f)
}
