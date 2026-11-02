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

/// Canonical NST civil transition cutoff fraction (`0.992` ≈ `23:48:28.8 NST`).
pub const CIVIL_CUTOFF_NST_FRACTION: f64 = 0.992;

/// Borderline Sankranti transit sensitivity window (`±4.0` minutes around the civil cutoff).
pub const BORDERLINE_CUTOFF_WINDOW_MINUTES: f64 = 4.0;

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

/// Topocentric atmospheric and elevation parameters for an observing station.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopocentricAtmosphere {
    /// Observer elevation above mean sea level in meters (`1,400 m` for Kathmandu).
    pub elevation_m: f64,
    /// Local barometric pressure in hectopascals (`856.0 hPa` for Kathmandu).
    pub pressure_hpa: f64,
    /// Local ambient sunrise temperature in degrees Celsius (`18.0 °C` for Kathmandu).
    pub temp_celsius: f64,
}

impl TopocentricAtmosphere {
    /// Canonical Kathmandu Valley high-altitude atmosphere (`1,400 m`, `856 hPa`, `18 °C`).
    #[inline]
    pub const fn kathmandu() -> Self {
        Self {
            elevation_m: KATHMANDU_ELEVATION_M,
            pressure_hpa: KATHMANDU_PRESSURE_HPA,
            temp_celsius: KATHMANDU_TEMP_CELSIUS,
        }
    }

    /// Standard sea-level reference atmosphere (`0 m`, `1,010 hPa`, `10 °C`).
    #[inline]
    pub const fn sea_level() -> Self {
        Self {
            elevation_m: 0.0,
            pressure_hpa: 1_010.0,
            temp_celsius: 10.0,
        }
    }
}

impl Default for TopocentricAtmosphere {
    fn default() -> Self {
        Self::kathmandu()
    }
}

/// Detailed diagnostic breakdown of Kathmandu topocentric sunrise vs. standard sea-level sunrise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopocentricSunriseMetrics {
    /// Two-pass refined UTC Julian Day of topocentric sunrise (with elevation dip, refraction, and `s_0 / R_AU`).
    pub sunrise_jd_utc: f64,
    /// Two-pass refined UTC Julian Day of standard flat-horizon sea-level sunrise (`Z = 90.8333°`).
    pub sea_level_sunrise_jd_utc: f64,
    /// Effective topocentric sunrise zenith distance in degrees (`~91.83°` at `1,400 m`).
    pub zenith_deg: f64,
    /// Geometric horizon depression `Δd = 0.0293° √h` in degrees.
    pub horizon_dip_deg: f64,
    /// Pressure- and temperature-compensated horizon refraction `R_atm` in degrees.
    pub refraction_deg: f64,
    /// Apparent solar semi-diameter `s_0 / R_AU` in degrees at the sunrise instant.
    pub solar_semidiameter_deg: f64,
    /// Equatorial horizontal solar parallax `π_0 / R_AU` in degrees at the sunrise instant.
    pub solar_parallax_deg: f64,
    /// Number of seconds by which high-altitude sunrise precedes sea-level sunrise (`(sea_level - topocentric) * 86400`).
    pub altitude_advance_seconds: f64,
}

fn solve_hour_angle_sunrise_jd(
    jd_utc_midnight: f64,
    jd_eval: f64,
    zenith_deg: f64,
) -> f64 {
    let (dec_deg, eot_minutes) = solar_declination_and_eot(jd_eval);
    let lat_rad = KATHMANDU_LAT_DEG.to_radians();
    let dec_rad = dec_deg.to_radians();
    let zenith_rad = zenith_deg.to_radians();

    let cos_h0 = (zenith_rad.cos() - lat_rad.sin() * dec_rad.sin())
        / (lat_rad.cos() * dec_rad.cos());
    let h0_deg = cos_h0.clamp(-1.0, 1.0).acos().to_degrees();

    let solar_noon_utc_min = 720.0 - 4.0 * KATHMANDU_LON_DEG - eot_minutes;
    let sunrise_utc_min = solar_noon_utc_min - 4.0 * h0_deg;

    jd_utc_midnight + sunrise_utc_min / 1440.0
}

/// Computes the two-pass refined high-altitude topocentric sunrise and diagnostic breakdown
/// for Kathmandu (`27.7172° N, 85.3240° E`) on the civil date whose UTC midnight is `jd_utc_midnight`.
pub fn kathmandu_topocentric_sunrise(
    jd_utc_midnight: f64,
    atmosphere: TopocentricAtmosphere,
) -> TopocentricSunriseMetrics {
    let jd_probe = jd_utc_midnight + 0.01;

    // Two-pass sea-level reference sunrise (Z = 90.8333°)
    let sea_pass1 = solve_hour_angle_sunrise_jd(jd_utc_midnight, jd_probe, SUNRISE_ZENITH_DEG);
    let sea_level_sunrise_jd_utc =
        solve_hour_angle_sunrise_jd(jd_utc_midnight, sea_pass1, SUNRISE_ZENITH_DEG);

    // Pass 1 topocentric sunrise using initial probe
    let z_pass1 = topocentric_sunrise_zenith_at_jd_deg(
        jd_probe,
        atmosphere.elevation_m,
        atmosphere.pressure_hpa,
        atmosphere.temp_celsius,
    );
    let topo_pass1 = solve_hour_angle_sunrise_jd(jd_utc_midnight, jd_probe, z_pass1);

    // Pass 2 topocentric sunrise evaluated at Pass 1 sunrise instant
    let horizon_dip_deg = geometric_horizon_dip_deg(atmosphere.elevation_m);
    let refraction_deg =
        atmospheric_refraction_horizon_deg(atmosphere.pressure_hpa, atmosphere.temp_celsius);
    let solar_semidiameter_deg = solar_semidiameter_deg(topo_pass1);
    let solar_parallax_deg = solar_horizontal_parallax_deg(topo_pass1);
    let zenith_deg =
        90.0 + horizon_dip_deg + refraction_deg + solar_semidiameter_deg - solar_parallax_deg;

    let sunrise_jd_utc = solve_hour_angle_sunrise_jd(jd_utc_midnight, topo_pass1, zenith_deg);
    let altitude_advance_seconds = (sea_level_sunrise_jd_utc - sunrise_jd_utc) * 86_400.0;

    TopocentricSunriseMetrics {
        sunrise_jd_utc,
        sea_level_sunrise_jd_utc,
        zenith_deg,
        horizon_dip_deg,
        refraction_deg,
        solar_semidiameter_deg,
        solar_parallax_deg,
        altitude_advance_seconds,
    }
}

/// Computes the exact UTC Julian Day of local topocentric sunrise in Kathmandu (`1,400 m`, `856 hPa`)
/// for a civil date whose UTC midnight is `jd_utc_midnight`.
pub fn kathmandu_sunrise_jd_utc(jd_utc_midnight: f64) -> f64 {
    kathmandu_topocentric_sunrise(jd_utc_midnight, TopocentricAtmosphere::kathmandu())
        .sunrise_jd_utc
}

/// Diagnostic resolution for a Sankranti solar transit relative to the Kathmandu civil cutoff,
/// including topocentric horizon dip (`Δd = 0.0293°√h`), barometric refraction (`856 hPa`),
/// and apparent solar semi-diameter (`s_0 / R_AU`) corrections for borderline transits (`±4 min`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderlineSankrantiResolution {
    /// Resolved 00:00 UTC Julian Day of the first civil day (*Gate 1* / १ गते).
    pub civil_start_jd_utc: f64,
    /// Uncorrected sea-level civil start 00:00 UTC Julian Day (without `1,400 m` elevation dip).
    pub sea_level_civil_start_jd_utc: f64,
    /// Signed time margin in minutes (`transit_nst - cutoff_nst`); `< 0` stays on current day, `> 0` rolls to next day.
    pub cutoff_margin_minutes: f64,
    /// `true` if the Sankranti occurs within `±4.0` minutes of the civil transition cutoff.
    pub is_borderline: bool,
    /// Topocentric cutoff shift in seconds induced by elevation dip, pressure refraction, and `s_0 / R_AU`.
    pub topocentric_cutoff_shift_seconds: f64,
    /// High-altitude topocentric sunrise metrics on the resolved *Gate 1* morning.
    pub sunrise_metrics: TopocentricSunriseMetrics,
}

/// Resolves an astronomical Sankranti transit (`transit_jd_utc`) to its Bikram Sambat
/// civil start day (*Gate 1* / १ गते) using topocentric horizon dip (`Δd = 0.0293°√h`),
/// barometric pressure (`856 hPa`), and apparent solar semi-diameter (`s_0 / R_AU`)
/// calibration for borderline transits within `±4` minutes of the civil cutoff.
pub fn resolve_sankranti_civil_day(
    transit_jd_utc: f64,
    atmosphere: TopocentricAtmosphere,
) -> BorderlineSankrantiResolution {
    let jd_nst_shifted = transit_jd_utc + 0.5 + NST_OFFSET_DAYS;
    let civil_day_index = jd_nst_shifted.floor();
    let nst_fraction = jd_nst_shifted - civil_day_index;

    // Reference Kathmandu mean topocentric zenith vs. date- and station-specific topocentric zenith
    let z_ref_ktm = topocentric_sunrise_zenith_deg(
        KATHMANDU_ELEVATION_M,
        KATHMANDU_PRESSURE_HPA,
        KATHMANDU_TEMP_CELSIUS,
    );
    let z_actual = topocentric_sunrise_zenith_at_jd_deg(
        transit_jd_utc,
        atmosphere.elevation_m,
        atmosphere.pressure_hpa,
        atmosphere.temp_celsius,
    );
    let z_sea_level = topocentric_sunrise_zenith_at_jd_deg(transit_jd_utc, 0.0, 1_010.0, 10.0);

    // 1° of zenith depression corresponds to 4 minutes (240 s) of diurnal hour-angle rotation
    let delta_z_from_ktm_ref = z_actual - z_ref_ktm;
    let delta_z_from_sea_level = z_actual - z_sea_level;

    let topocentric_shift_days = delta_z_from_ktm_ref * (4.0 / 1_440.0);
    let sea_level_shift_days = (z_sea_level - z_ref_ktm) * (4.0 / 1_440.0);

    let effective_cutoff_fraction = CIVIL_CUTOFF_NST_FRACTION + topocentric_shift_days;
    let sea_level_cutoff_fraction = CIVIL_CUTOFF_NST_FRACTION + sea_level_shift_days;

    let cutoff_margin_minutes = (nst_fraction - effective_cutoff_fraction) * 1_440.0;
    let is_borderline = cutoff_margin_minutes.abs() <= BORDERLINE_CUTOFF_WINDOW_MINUTES;

    let effective_day_index = if nst_fraction > effective_cutoff_fraction {
        civil_day_index + 1.0
    } else {
        civil_day_index
    };

    let sea_level_day_index = if nst_fraction > sea_level_cutoff_fraction {
        civil_day_index + 1.0
    } else {
        civil_day_index
    };

    let civil_start_jd_utc = effective_day_index - 0.5;
    let sea_level_civil_start_jd_utc = sea_level_day_index - 0.5;
    let sunrise_metrics = kathmandu_topocentric_sunrise(civil_start_jd_utc, atmosphere);

    BorderlineSankrantiResolution {
        civil_start_jd_utc,
        sea_level_civil_start_jd_utc,
        cutoff_margin_minutes,
        is_borderline,
        topocentric_cutoff_shift_seconds: delta_z_from_sea_level * 240.0,
        sunrise_metrics,
    }
}

/// Maps an astronomical Sankranti transit instant (`transit_jd_utc`) to the integer
/// Julian Day (at 00:00 UTC) of the first civil day (*Gate 1* / १ गते) of the new Bikram Sambat month.
///
/// Under the Nepali Panchanga civil transition rule:
/// A Sankranti occurring before the civil transition threshold in Nepal Standard Time
/// inaugurates *Gate 1* on that same civil date; a transit after the threshold inaugurates
/// *Gate 1* on the following civil sunrise.
pub fn sankranti_to_civil_day_jd_utc(transit_jd_utc: f64) -> f64 {
    resolve_sankranti_civil_day(transit_jd_utc, TopocentricAtmosphere::kathmandu())
        .civil_start_jd_utc
}

