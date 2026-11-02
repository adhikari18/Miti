//! Topocentric Sunrise & Civil Day Boundary Calibration for Kathmandu, Nepal.
//!
//! Coordinates:
//! - Latitude:  `27.7172° N`
//! - Longitude: `85.3240° E`
//! - Elevation: `1,400 m` (Kathmandu Valley floor)
//! - Timezone:  Nepal Standard Time (`UTC+05:45`, `20,700` seconds)

use crate::vsop87::solar_declination_and_eot;

/// Canonical geodetic latitude of Kathmandu, Nepal (`27.7172° N`).
pub const KATHMANDU_LAT_DEG: f64 = 27.7172;

/// Canonical geodetic longitude of Kathmandu, Nepal (`85.3240° E`).
pub const KATHMANDU_LON_DEG: f64 = 85.3240;

/// Nepal Standard Time offset from UTC in days (`5.75 hours / 24 hours`).
pub const NST_OFFSET_DAYS: f64 = 5.75 / 24.0;

/// Standard geometric + atmospheric refraction + elevation horizon depression zenith angle (`90.8333°`).
pub const SUNRISE_ZENITH_DEG: f64 = 90.8333;

/// Computes the exact UTC Julian Day of local sunrise in Kathmandu for a civil date
/// whose UTC midnight is `jd_utc_midnight`.
pub fn kathmandu_sunrise_jd_utc(jd_utc_midnight: f64) -> f64 {
    // Evaluate solar declination and Equation of Time near 06:00 NST (00:15 UTC)
    let jd_probe = jd_utc_midnight + 0.01;
    let (dec_deg, eot_minutes) = solar_declination_and_eot(jd_probe);

    let lat_rad = KATHMANDU_LAT_DEG.to_radians();
    let dec_rad = dec_deg.to_radians();
    let zenith_rad = SUNRISE_ZENITH_DEG.to_radians();

    // Hour angle H0 at sunrise: cos(H0) = (cos(Z) - sin(φ)sin(δ)) / (cos(φ)cos(δ))
    let cos_h0 = (zenith_rad.cos() - lat_rad.sin() * dec_rad.sin())
        / (lat_rad.cos() * dec_rad.cos());
    let clamped_cos = cos_h0.clamp(-1.0, 1.0);
    let h0_deg = clamped_cos.acos().to_degrees();

    // Solar noon in UTC minutes from 00:00 UTC:
    // 720 - 4 * longitude - EquationOfTime
    let solar_noon_utc_min = 720.0 - 4.0 * KATHMANDU_LON_DEG - eot_minutes;
    let sunrise_utc_min = solar_noon_utc_min - 4.0 * h0_deg;

    jd_utc_midnight + sunrise_utc_min / 1440.0
}

/// Maps an astronomical Sankranti transit instant (`transit_jd_utc`) to the integer
/// Julian Day (at 00:00 UTC) of the first civil day (*Gate 1* / १ गते) of the new Bikram Sambat month.
///
/// Under the Nepali Panchanga civil transition rule:
/// A Sankranti occurring before the civil transition threshold in Nepal Standard Time
/// inaugurates *Gate 1* on that same civil date; a transit after the threshold inaugurates
/// *Gate 1* on the following civil sunrise.
pub fn sankranti_to_civil_day_jd_utc(transit_jd_utc: f64) -> f64 {
    // Convert transit JD (where .0 is 12:00 UTC) to NST civil day index
    // Adding 0.5 shifts origin from 12:00 UTC to 00:00 UTC; adding NST_OFFSET_DAYS shifts to 00:00 NST.
    let jd_nst_shifted = transit_jd_utc + 0.5 + NST_OFFSET_DAYS;
    let civil_day_index = jd_nst_shifted.floor();
    let nst_fraction = jd_nst_shifted - civil_day_index;

    // Civil midnight / Aparahna-Madhyaratri cutoff in NST:
    // If transit occurs after 23:48 NST (fraction > 0.992), roll to next civil sunrise day.
    let effective_day_index = if nst_fraction > 0.992 {
        civil_day_index + 1.0
    } else {
        civil_day_index
    };

    // Return the 00:00 UTC Julian Day corresponding to that civil date
    effective_day_index - 0.5
}

