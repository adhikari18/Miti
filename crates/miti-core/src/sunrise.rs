//! Topocentric Sunrise & Civil Day Boundary Calibration for Kathmandu, Nepal.
//!
//! Coordinates:
//! - Latitude:  `27.7172° N`
//! - Longitude: `85.3240° E`
//! - Elevation: `1,400 m` (Kathmandu Valley floor)
//! - Timezone:  Nepal Standard Time (`UTC+05:45`, `20,700` seconds)

use crate::vsop87::{
    solar_declination_and_eot, solar_horizontal_parallax_deg, solar_semidiameter_deg,
    SOLAR_PARALLAX_1AU_ARCSEC, SOLAR_SEMIDIAMETER_1AU_ARCSEC,
};

/// Canonical geodetic latitude of Kathmandu, Nepal (`27.7172° N`).
pub const KATHMANDU_LAT_DEG: f64 = 27.7172;

/// Canonical geodetic longitude of Kathmandu, Nepal (`85.3240° E`).
pub const KATHMANDU_LON_DEG: f64 = 85.3240;

/// Canonical elevation of the Kathmandu Valley floor above mean sea level (`1,400 m`).
pub const KATHMANDU_ELEVATION_M: f64 = 1_400.0;

/// Canonical barometric pressure at `1,400 m` in Kathmandu (`856.0 hPa`).
pub const KATHMANDU_PRESSURE_HPA: f64 = 856.0;

/// Canonical mean sunrise ambient temperature in Kathmandu (`18.0 °C`).
pub const KATHMANDU_TEMP_CELSIUS: f64 = 18.0;

/// Nepal Standard Time offset from UTC in days (`5.75 hours / 24 hours`).
pub const NST_OFFSET_DAYS: f64 = 5.75 / 24.0;

/// Standard sea-level apparent sunrise zenith angle (`90.8333°` = `90° 50'`).
pub const SUNRISE_ZENITH_DEG: f64 = 90.8333;

/// Geometric horizon depression coefficient (`0.0293° / √m`, accounting for terrestrial refraction).
pub const HORIZON_DIP_COEFF_DEG: f64 = 0.0293;

/// Standard sea-level atmospheric refraction at the horizon (`34'` = `0.566667°` at `1010 hPa`, `10 °C`).
pub const STANDARD_HORIZON_REFRACTION_DEG: f64 = 34.0 / 60.0;

/// Computes the elevation-dependent geometric horizon depression angle `Δd` in degrees:
/// `Δd = 0.0293° * √max(0, elevation_m)`.
///
/// At Kathmandu's elevation (`h = 1,400 m`), `Δd ≈ 1.096306°` (`1° 05' 47"`).
#[inline]
pub fn geometric_horizon_dip_deg(elevation_m: f64) -> f64 {
    let clamped_h = if elevation_m > 0.0 { elevation_m } else { 0.0 };
    HORIZON_DIP_COEFF_DEG * clamped_h.sqrt()
}

/// Computes topocentric atmospheric refraction `R_atm` (in degrees) at the horizon,
/// scaled for local barometric pressure `pressure_hpa` (in hPa) and ambient temperature
/// `temp_celsius` (in °C) per Meeus Ch. 16 / Saemundsson-Bennett relation:
///
/// `R_atm = (34' / 60) * (P / 1010.0) * (283.0 / (273.0 + T))`
#[inline]
pub fn atmospheric_refraction_horizon_deg(pressure_hpa: f64, temp_celsius: f64) -> f64 {
    let p_factor = (pressure_hpa.max(0.0)) / 1010.0;
    let kelvin = (273.0 + temp_celsius).max(150.0);
    let t_factor = 283.0 / kelvin;
    STANDARD_HORIZON_REFRACTION_DEG * p_factor * t_factor
}

/// Computes the topocentric apparent upper-limb sunrise zenith distance (`Z` in degrees)
/// for an observer at `elevation_m`, `pressure_hpa`, and `temp_celsius` using mean `1 AU`
/// solar semi-diameter (`s_0 = 959.63"`) and horizontal parallax (`π_0 = 8.794"`):
///
/// `Z = 90° + Δd(h) + R_atm(P, T) + (s_0 - π_0) / 3600`
#[inline]
pub fn topocentric_sunrise_zenith_deg(
    elevation_m: f64,
    pressure_hpa: f64,
    temp_celsius: f64,
) -> f64 {
    let dip_deg = geometric_horizon_dip_deg(elevation_m);
    let refraction_deg = atmospheric_refraction_horizon_deg(pressure_hpa, temp_celsius);
    let mean_sd_minus_parallax_deg =
        (SOLAR_SEMIDIAMETER_1AU_ARCSEC - SOLAR_PARALLAX_1AU_ARCSEC) / 3600.0;
    90.0 + dip_deg + refraction_deg + mean_sd_minus_parallax_deg
}

/// Computes the date-specific topocentric apparent sunrise zenith distance (`Z` in degrees)
/// at `jd_utc`, incorporating Earth–Sun radius vector variation `s(JD) = s_0 / R_AU(JD)`
/// and solar horizontal parallax `π_☉(JD) = π_0 / R_AU(JD)`:
///
/// `Z(JD) = 90° + Δd(h) + R_atm(P, T) + s(JD) - π_☉(JD)`
#[inline]
pub fn topocentric_sunrise_zenith_at_jd_deg(
    jd_utc: f64,
    elevation_m: f64,
    pressure_hpa: f64,
    temp_celsius: f64,
) -> f64 {
    let dip_deg = geometric_horizon_dip_deg(elevation_m);
    let refraction_deg = atmospheric_refraction_horizon_deg(pressure_hpa, temp_celsius);
    let sd_deg = solar_semidiameter_deg(jd_utc);
    let parallax_deg = solar_horizontal_parallax_deg(jd_utc);
    90.0 + dip_deg + refraction_deg + sd_deg - parallax_deg
}

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

