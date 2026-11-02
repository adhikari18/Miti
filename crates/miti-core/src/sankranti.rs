//! Core Sankranti (Solar Rashi Transit) Solver & Bikram Sambat Conversion Engine.
//!
//! Finds the exact astronomical moment when the Sun's Nirayana (Sidereal) longitude
//! enters each of the 12 Zodiac signs (*Rashis*: 0° Mesha = Baisakh .. 330° Meena = Chaitra)
//! via Newton-Raphson iteration over VSOP87 solar mechanics and Chitra Paksha Ayanamsa.

use crate::ayanamsa::{lahiri_panchanga_ayanamsa_deg, sidereal_solar_longitude_deg};
use crate::jdn::{gregorian_to_jd, GregorianDate};
use crate::sunrise::{resolve_sankranti_civil_day, TopocentricAtmosphere};
use crate::vsop87::signed_angle_diff;

/// Mean length of the sidereal year in days.
pub const SIDEREAL_YEAR_DAYS: f64 = 365.256_363_004;

/// Mean solar daily motion in degrees/day (`360° / 365.256363`).
pub const MEAN_SOLAR_DAILY_MOTION_DEG: f64 = 0.985_609_11;

/// Canonical Nepali Bikram Sambat month names (English & Devanagari).
pub const BS_MONTH_NAMES_EN: [&str; 12] = [
    "Baisakh", "Jestha", "Asar", "Shrawan", "Bhadra", "Ashwin",
    "Kartik", "Mangsir", "Poush", "Magh", "Falgun", "Chaitra",
];

pub const BS_MONTH_NAMES_NE: [&str; 12] = [
    "बैशाख", "जेठ", "असार", "साउन", "भदौ", "असोज",
    "कार्तिक", "मंसिर", "पुस", "माघ", "फागुन", "चैत",
];

pub const RASHI_NAMES_EN: [&str; 12] = [
    "Mesha (Aries)",
    "Vrishabha (Taurus)",
    "Mithuna (Gemini)",
    "Karkata (Cancer)",
    "Simha (Leo)",
    "Kanya (Virgo)",
    "Tula (Libra)",
    "Vrishchika (Scorpio)",
    "Dhanu (Sagittarius)",
    "Makara (Capricorn)",
    "Kumbha (Aquarius)",
    "Meena (Pisces)",
];

/// Represents an astronomical Sankranti (solar zodiac transit) and its corresponding civil boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct SankrantiEvent {
    /// Bikram Sambat year (e.g., `2083`).
    pub bs_year: i32,
    /// Bikram Sambat month (`1 = Baisakh` .. `12 = Chaitra`).
    pub bs_month: u8,
    /// Target sidereal longitude in degrees (`0.0, 30.0, ..., 330.0`).
    pub target_longitude_deg: f64,
    /// Exact UTC Julian Day of the solar Rashi transit.
    pub transit_jd_utc: f64,
    /// UTC Julian Day of Kathmandu sunrise on the civil *Gate 1* day.
    pub kathmandu_sunrise_jd_utc: f64,
    /// Julian Day (at 00:00 UTC) of the civil *Gate 1* (१ गते) date.
    pub civil_start_jd_utc: f64,
    /// Chitra Paksha / Lahiri Ayanamsa (in degrees) at the transit instant.
    pub ayanamsa_deg: f64,
    /// Signed time margin in minutes (`transit_nst - cutoff_nst`) relative to the civil cutoff.
    pub cutoff_margin_minutes: f64,
    /// `true` if the Sankranti transit occurs within `±4.0` minutes of the civil cutoff.
    pub is_borderline_transit: bool,
}

/// Represents a resolved Bikram Sambat (B.S.) civil date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MitiDate {
    /// Bikram Sambat year (e.g., `2083`).
    pub year: i32,
    /// Bikram Sambat month (`1 = Baisakh` .. `12 = Chaitra`).
    pub month: u8,
    /// Day of the month (*Gate* / गते, `1..=32`).
    pub day: u8,
    /// Day of week (`0 = Sunday / आइतबार` .. `6 = Saturday / शनिबार`).
    pub weekday: u8,
    /// Total number of days in this Bikram Sambat month (`29..=32`).
    pub days_in_month: u8,
}

/// Returns the target Nirayana (sidereal) longitude in degrees for a Bikram Sambat month `1..=12`.
/// - `1 (Baisakh)` -> `0.0°` (Mesha / Aries)
/// - `7 (Kartik)`  -> `180.0°` (Tula / Libra)
/// - `12 (Chaitra)` -> `330.0°` (Meena / Pisces)
#[inline]
pub fn target_longitude_for_bs_month(bs_month: u8) -> f64 {
    (((bs_month.saturating_sub(1)) % 12) as f64) * 30.0
}

/// Computes an initial approximate UTC Julian Day for the Sankranti of `(bs_year, bs_month)`.
///
/// Mesh Sankranti (`bs_month = 1`, `0°` Aries) occurs around April 13–14 of Gregorian year `bs_year - 57`.
fn initial_sankranti_guess_jd(bs_year: i32, bs_month: u8) -> f64 {
    let greg_year = bs_year - 57;
    let mesh_anchor_jd = gregorian_to_jd(greg_year, 4, 13.5);
    let month_offset = (bs_month.saturating_sub(1)) as f64;
    mesh_anchor_jd + month_offset * (SIDEREAL_YEAR_DAYS / 12.0)
}

/// Uses Newton-Raphson root-finding over VSOP87 + Lahiri Ayanamsa to locate the exact
/// UTC Julian Day (`transit_jd_utc`) when `sidereal_solar_longitude_deg(jd) == target_longitude_deg`
/// to sub-millisecond precision (`|Δλ| < 1e-9°`).
pub fn find_sankranti_transit_jd(bs_year: i32, bs_month: u8) -> SankrantiEvent {
    let target_deg = target_longitude_for_bs_month(bs_month);
    let mut jd = initial_sankranti_guess_jd(bs_year, bs_month);

    for _ in 0..16 {
        let current_lon = sidereal_solar_longitude_deg(jd);
        let diff_deg = signed_angle_diff(target_deg, current_lon);
        if diff_deg.abs() < 1e-9 {
            break;
        }
        // Numerical derivative (instantaneous solar angular velocity ~ 0.95° to 1.02°/day)
        let dt = 0.001;
        let lon_plus = sidereal_solar_longitude_deg(jd + dt);
        let velocity = signed_angle_diff(lon_plus, current_lon) / dt;
        let step = diff_deg / velocity.clamp(0.90, 1.05);
        jd += step;
    }

    let resolution = resolve_sankranti_civil_day(jd, TopocentricAtmosphere::kathmandu());
    let ayanamsa_deg = lahiri_panchanga_ayanamsa_deg(jd);

    SankrantiEvent {
        bs_year,
        bs_month,
        target_longitude_deg: target_deg,
        transit_jd_utc: jd,
        kathmandu_sunrise_jd_utc: resolution.sunrise_metrics.sunrise_jd_utc,
        civil_start_jd_utc: resolution.civil_start_jd_utc,
        ayanamsa_deg,
        cutoff_margin_minutes: resolution.cutoff_margin_minutes,
        is_borderline_transit: resolution.is_borderline,
    }
}

/// Computes the exact length in days (`29..=32`) of a given `(bs_year, bs_month)`
/// from the difference between consecutive civil Sankranti start dates.
pub fn days_in_bs_month(bs_year: i32, bs_month: u8) -> u8 {
    let current = find_sankranti_transit_jd(bs_year, bs_month);
    let (next_year, next_month) = if bs_month == 12 {
        (bs_year + 1, 1)
    } else {
        (bs_year, bs_month + 1)
    };
    let next = find_sankranti_transit_jd(next_year, next_month);
    (next.civil_start_jd_utc - current.civil_start_jd_utc).round() as u8
}

/// Converts a `GregorianDate` (UTC) into a `MitiDate` (Bikram Sambat) using pure astronomy.
pub fn gregorian_to_miti(date: GregorianDate) -> MitiDate {
    let target_jd = date.to_jd_utc();

    // Approximate Bikram Sambat year (either date.year + 56 or date.year + 57)
    let mut bs_year = date.year + 57;
    let baisakh_start = find_sankranti_transit_jd(bs_year, 1).civil_start_jd_utc;
    if target_jd < baisakh_start {
        bs_year -= 1;
    }

    // Estimate the month index from elapsed days since Baisakh 1
    let year_start_jd = find_sankranti_transit_jd(bs_year, 1).civil_start_jd_utc;
    let elapsed_days = (target_jd - year_start_jd).max(0.0);
    let mut bs_month = ((elapsed_days / 30.4375).floor() as u8 + 1).clamp(1, 12);

    // Refine month by checking adjacent Sankranti civil start boundaries
    loop {
        let m_start = find_sankranti_transit_jd(bs_year, bs_month).civil_start_jd_utc;
        if target_jd < m_start && bs_month > 1 {
            bs_month -= 1;
            continue;
        }
        let (next_y, next_m) = if bs_month == 12 {
            (bs_year + 1, 1)
        } else {
            (bs_year, bs_month + 1)
        };
        let next_start = find_sankranti_transit_jd(next_y, next_m).civil_start_jd_utc;
        if target_jd >= next_start {
            if bs_month == 12 {
                bs_year += 1;
                bs_month = 1;
            } else {
                bs_month += 1;
            }
            continue;
        }
        let day = (target_jd - m_start).round() as u8 + 1;
        let days_in_month = (next_start - m_start).round() as u8;
        return MitiDate {
            year: bs_year,
            month: bs_month,
            day,
            weekday: date.weekday(),
            days_in_month,
        };
    }
}

/// Converts a Bikram Sambat `(bs_year, bs_month, bs_day)` back to a `GregorianDate` (UTC).
pub fn miti_to_gregorian(bs_year: i32, bs_month: u8, bs_day: u8) -> Result<GregorianDate, &'static str> {
    if !(1..=12).contains(&bs_month) {
        return Err("Bikram Sambat month must be between 1 (Baisakh) and 12 (Chaitra).");
    }
    let max_days = days_in_bs_month(bs_year, bs_month);
    if bs_day < 1 || bs_day > max_days {
        return Err("Bikram Sambat day (Gate) is out of bounds for the specified month.");
    }
    let m_start_jd = find_sankranti_transit_jd(bs_year, bs_month).civil_start_jd_utc;
    let target_jd = m_start_jd + (bs_day as f64 - 1.0);
    Ok(GregorianDate::from_jd_utc(target_jd))
}

