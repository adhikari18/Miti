//! # `miti-core` (मिति)
//!
//! Pure-math Bikram Sambat (B.S.) astronomical date engine and sidereal calendar solver.
//!
//! Computes Nepali Bikram Sambat dates algorithmically using:
//! 1. **VSOP87 Solar Mechanics** (`vsop87.rs`) for apparent tropical solar longitude.
//! 2. **Chitra Paksha / Lahiri Ayanamsa** (`ayanamsa.rs`) for Nirayana sidereal longitude.
//! 3. **Newton-Raphson Sankranti Root-Finding** (`sankranti.rs`) for exact Rashi ingress times.
//! 4. **Kathmandu Topocentric Sunrise Calibration** (`sunrise.rs`, `27.7172° N, 85.3240° E`) for civil day boundaries.

pub mod ayanamsa;
pub mod jdn;
pub mod sankranti;
pub mod sunrise;
pub mod vsop87;

pub use ayanamsa::{lahiri_panchanga_ayanamsa_deg, sidereal_solar_longitude_deg};
pub use jdn::{gregorian_to_jd, jd_to_gregorian, GregorianDate};
pub use sankranti::{
    days_in_bs_month, find_sankranti_transit_jd, gregorian_to_miti, miti_to_gregorian, MitiDate,
    SankrantiEvent, BS_MONTH_NAMES_EN, BS_MONTH_NAMES_NE, RASHI_NAMES_EN,
};
pub use sunrise::{kathmandu_sunrise_jd_utc, sankranti_to_civil_day_jd_utc};
pub use vsop87::{
    apparent_tropical_solar_longitude_deg, solar_horizontal_parallax_deg, solar_radius_vector_au,
    solar_semidiameter_deg, SOLAR_PARALLAX_1AU_ARCSEC, SOLAR_SEMIDIAMETER_1AU_ARCSEC,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jdn_roundtrip() {
        let g = GregorianDate::new(2026, 11, 2);
        let jd = g.to_jd_utc();
        assert_eq!(jd, 2_461_346.5);
        assert_eq!(GregorianDate::from_jd_utc(jd), g);
    }

    #[test]
    fn test_kartik_2083_conversion() {
        let g = GregorianDate::new(2026, 11, 2);
        let bs = gregorian_to_miti(g);
        assert_eq!(bs.year, 2083);
        assert_eq!(bs.month, 7);
        assert_eq!(bs.day, 17);

        let back = miti_to_gregorian(bs.year, bs.month, bs.day).unwrap();
        assert_eq!(back, g);
    }

    #[test]
    fn test_bs_month_lengths_within_canonical_bounds() {
        for m in 1..=12 {
            let len = days_in_bs_month(2083, m);
            assert!((29..=32).contains(&len), "Month {m} length {len} out of [29, 32]");
        }
    }

    #[test]
    fn test_solar_radius_vector_and_semidiameter() {
        // Perihelion (~Jan 3, 2026) vs Aphelion (~Jul 4, 2026)
        let jd_perihelion = gregorian_to_jd(2026, 1, 3.5);
        let jd_aphelion = gregorian_to_jd(2026, 7, 4.5);

        let r_peri = solar_radius_vector_au(jd_perihelion);
        let r_aph = solar_radius_vector_au(jd_aphelion);

        assert!(r_peri > 0.9830 && r_peri < 0.9836);
        assert!(r_aph > 1.0164 && r_aph < 1.0170);

        let sd_peri_arcsec = solar_semidiameter_deg(jd_perihelion) * 3600.0;
        let sd_aph_arcsec = solar_semidiameter_deg(jd_aphelion) * 3600.0;

        assert!(sd_peri_arcsec > 975.0 && sd_peri_arcsec < 976.5);
        assert!(sd_aph_arcsec > 943.0 && sd_aph_arcsec < 944.5);
        assert!(sd_peri_arcsec > sd_aph_arcsec);
    }
}

