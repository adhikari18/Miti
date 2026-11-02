//! VSOP87 / Meeus high-precision apparent tropical solar longitude engine.
//!
//! Computes the Sun's geometric mean longitude, mean anomaly, equation of center,
//! planetary perturbations (Venus, Jupiter, Moon), nutation in longitude (Δψ),
//! and annual aberration (-20.4898").

use crate::jdn::jd_utc_to_jde;

/// Normalizes any angle in degrees into the canonical interval `[0.0, 360.0)`.
#[inline]
pub fn normalize_deg(deg: f64) -> f64 {
    let r = deg % 360.0;
    if r < 0.0 {
        r + 360.0
    } else {
        r
    }
}

/// Normalizes an angular difference into `[-180.0, 180.0)`.
#[inline]
pub fn signed_angle_diff(target_deg: f64, current_deg: f64) -> f64 {
    let mut diff = (target_deg - current_deg) % 360.0;
    if diff < -180.0 {
        diff += 360.0;
    } else if diff >= 180.0 {
        diff -= 360.0;
    }
    diff
}

/// Computes the apparent tropical solar longitude (in degrees, `[0, 360)`)
/// at a given UTC Julian Day (`jd_utc`) using truncated VSOP87 / Meeus Ch. 25 series.
pub fn apparent_tropical_solar_longitude_deg(jd_utc: f64) -> f64 {
    let jde = jd_utc_to_jde(jd_utc);
    // Julian centuries from J2000.0 (2451545.0 TT)
    let t = (jde - 2_451_545.0) / 36_525.0;
    let t2 = t * t;
    let t3 = t2 * t;

    // 1. Geometric mean longitude of the Sun referred to the mean equinox of date (L0)
    let l0 = normalize_deg(280.466_456_7 + 36_000.769_827_79 * t + 0.000_303_202_8 * t2);

    // 2. Mean anomaly of the Sun (M)
    let m_deg = normalize_deg(
        357.529_109_2 + 35_999.050_290_9 * t - 0.000_153_6 * t2 + t3 / 24_490_000.0,
    );
    let m = m_deg.to_radians();

    // 3. Sun's Equation of the Center (C)
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t2) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();

    // 4. VSOP87 / Meeus planetary and lunar periodic perturbations (in degrees)
    let v1 = (125.04 - 1_934.136 * t).to_radians(); // Moon ascending node Omega
    let p_venus1 = (351.52 + 22_518.443 * t).to_radians();
    let p_venus2 = (253.14 + 45_036.886 * t).to_radians();
    let p_jupiter = (20.19 + 32_964.467 * t).to_radians();
    let p_moon = (297.85 + 445_267.112 * t).to_radians();

    let dl_perturb = 0.00134 * p_venus1.cos()
        + 0.00154 * p_venus2.cos()
        + 0.00200 * p_jupiter.cos()
        + 0.00179 * p_moon.sin();

    // True geometric longitude (Theta)
    let true_long = l0 + c + dl_perturb;

    // 5. Apparent longitude: correct for nutation and annual aberration (-0.00569° - 0.00478° sin(Omega))
    let apparent_long = true_long - 0.00569 - 0.00478 * v1.sin();

    normalize_deg(apparent_long)
}

/// Computes the solar declination (δ in degrees) and Equation of Time (in minutes)
/// for sunrise/sunset calculations at `jd_utc`.
pub fn solar_declination_and_eot(jd_utc: f64) -> (f64, f64) {
    let jde = jd_utc_to_jde(jd_utc);
    let t = (jde - 2_451_545.0) / 36_525.0;

    let lam_rad = apparent_tropical_solar_longitude_deg(jd_utc).to_radians();
    // Mean obliquity of the ecliptic (ε0)
    let eps0_deg = 23.439_291_11 - 0.013_004_167 * t - 0.000_000_163_9 * t * t;
    let omega = (125.04 - 1_934.136 * t).to_radians();
    let eps_rad = (eps0_deg + 0.00256 * omega.cos()).to_radians();

    // Declination δ = arcsin(sin(ε) * sin(λ))
    let sin_dec = eps_rad.sin() * lam_rad.sin();
    let dec_deg = sin_dec.asin().to_degrees();

    // Equation of Time (Meeus Ch. 28 approximation in minutes)
    let l0_rad = normalize_deg(280.466_46 + 36_000.769_83 * t).to_radians();
    let m_rad = normalize_deg(357.529_11 + 35_999.050_29 * t).to_radians();
    let e = 0.016_708_634 - 0.000_042_037 * t;
    let y = (eps_rad / 2.0).tan().powi(2);

    let eot_rad = y * (2.0 * l0_rad).sin() - 2.0 * e * m_rad.sin()
        + 4.0 * e * y * m_rad.sin() * (2.0 * l0_rad).cos()
        - 0.5 * y * y * (4.0 * l0_rad).sin()
        - 1.25 * e * e * (2.0 * m_rad).sin();

    let eot_minutes = eot_rad.to_degrees() * 4.0;
    (dec_deg, eot_minutes)
}

