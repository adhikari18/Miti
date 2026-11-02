//! Julian Day Number (JDN) and Julian Ephemeris Day (JDE) conversions.
//!
//! Implements Fliegel-Van Flandern / Meeus continuous proleptic Gregorian
//! to Julian Day conversions and polynomial ΔT (TT - UTC) corrections.

/// Represents a proleptic Gregorian calendar date (UTC midnight).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GregorianDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl GregorianDate {
    pub const fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Converts this Gregorian UTC date (00:00:00 UTC) to a continuous Julian Day (JD).
    pub fn to_jd_utc(self) -> f64 {
        gregorian_to_jd(self.year, self.month, self.day as f64)
    }

    /// Creates a `GregorianDate` from a continuous Julian Day at 00:00 UTC.
    pub fn from_jd_utc(jd: f64) -> Self {
        let (year, month, day) = jd_to_gregorian(jd);
        Self { year, month, day }
    }

    /// Returns day of week (0 = Sunday / आइतबार .. 6 = Saturday / शनिबार).
    pub fn weekday(self) -> u8 {
        let jdn = (self.to_jd_utc() + 0.5).floor() as i64;
        ((jdn + 1).rem_euclid(7)) as u8
    }
}

/// Converts a Gregorian date (with fractional day `day_with_fraction`) to Julian Day.
/// Reference: Jean Meeus, *Astronomical Algorithms* (2nd ed., Chapter 7).
pub fn gregorian_to_jd(year: i32, month: u8, day_with_fraction: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year as f64 - 1.0, month as f64 + 12.0)
    } else {
        (year as f64, month as f64)
    };

    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();

    (365.25 * (y + 4716.0)).floor()
        + (30.6001 * (m + 1.0)).floor()
        + day_with_fraction
        + b
        - 1524.5
}

/// Converts a Julian Day (JD) back to `(year, month, day)` in the proleptic Gregorian calendar.
pub fn jd_to_gregorian(jd: f64) -> (i32, u8, u8) {
    let jd_shifted = jd + 0.5;
    let z = jd_shifted.floor();
    let f = jd_shifted - z;

    let a = if z < 2_299_161.0 {
        z
    } else {
        let alpha = ((z - 1_867_216.25) / 36_524.25).floor();
        z + 1.0 + alpha - (alpha / 4.0).floor()
    };

    let b = a + 1524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let d = (365.25 * c).floor();
    let e = ((b - d) / 30.6001).floor();

    let day_float = b - d - (30.6001 * e).floor() + f;
    let day = day_float.floor() as u8;

    let month = if e < 14.0 {
        (e - 1.0) as u8
    } else {
        (e - 13.0) as u8
    };

    let year = if month > 2 {
        (c - 4716.0) as i32
    } else {
        (c - 4715.0) as i32
    };

    (year, month, day)
}

/// Estimates ΔT = TT - UT (in seconds) using NASA Espenak-Meeus polynomial expressions.
pub fn delta_t_seconds(year_decimal: f64) -> f64 {
    if (1900.0..2005.0).contains(&year_decimal) {
        let t = (year_decimal - 2000.0) / 100.0;
        63.86 + 33.45 * t
    } else if (2005.0..=2050.0).contains(&year_decimal) {
        let t = year_decimal - 2000.0;
        62.92 + 0.32217 * t + 0.005589 * t * t
    } else {
        let u = (year_decimal - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

/// Converts a UTC Julian Day to Julian Ephemeris Day (JDE = JD_UTC + ΔT / 86400).
pub fn jd_utc_to_jde(jd_utc: f64) -> f64 {
    let approx_year = 2000.0 + (jd_utc - 2_451_545.0) / 365.25;
    jd_utc + delta_t_seconds(approx_year) / 86_400.0
}
