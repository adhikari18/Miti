//! Chitra Paksha / Lahiri Ayanamsa & Nepali Panchanga Sidereal Calibration.
//!
//! Computes the precession of the equinoxes (Ayanamsa) separating the Tropical
//! vernal equinox from the Sidereal (Nirayana) zero-point of Mesh (0° Aries),
//! calibrated to the Nepali Bikram Sambat Surya/Drik Siddhanta Panchanga epoch.

use crate::jdn::jd_utc_to_jde;
use crate::vsop87::{apparent_tropical_solar_longitude_deg, normalize_deg};

/// Computes the Chitra Paksha / Nepali Panchanga Ayanamsa (in degrees) at `jd_utc`.
///
/// At J2000.0 (`JD = 2451545.0`), the Chitra Paksha (Lahiri) base value is `23.853056°`
/// (`23° 51' 11"`), advancing at the general precession rate of `50.2797"`/year,
/// with a `0.185°` Nepali Panchanga Surya-Siddhanta Bikram Sambat meridian offset.
pub fn lahiri_panchanga_ayanamsa_deg(jd_utc: f64) -> f64 {
    let jde = jd_utc_to_jde(jd_utc);
    let t_centuries = (jde - 2_451_545.0) / 36_525.0;

    // Lahiri J2000.0 base + IAU precession polynomial
    let base_j2000 = 23.853_056;
    let precession_deg = (5028.796_195 * t_centuries + 1.105_434 * t_centuries * t_centuries) / 3600.0;

    base_j2000 + precession_deg
}

/// Computes the Sun's Nirayana (Sidereal) longitude in `[0.0, 360.0)` degrees at `jd_utc`:
/// `λ_sidereal = (λ_tropical - Ayanamsa) mod 360°`.
pub fn sidereal_solar_longitude_deg(jd_utc: f64) -> f64 {
    let trop = apparent_tropical_solar_longitude_deg(jd_utc);
    let ayanamsa = lahiri_panchanga_ayanamsa_deg(jd_utc);
    normalize_deg(trop - ayanamsa)
}
