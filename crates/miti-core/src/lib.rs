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
pub use sunrise::{
    atmospheric_refraction_horizon_deg, geometric_horizon_dip_deg, kathmandu_sunrise_jd_utc,
    kathmandu_topocentric_sunrise, resolve_sankranti_civil_day, sankranti_to_civil_day_jd_utc,
    topocentric_sunrise_zenith_at_jd_deg, topocentric_sunrise_zenith_deg,
    BorderlineSankrantiResolution, TopocentricAtmosphere, TopocentricSunriseMetrics,
    BORDERLINE_CUTOFF_WINDOW_MINUTES, CIVIL_CUTOFF_NST_FRACTION, KATHMANDU_ELEVATION_M,
    KATHMANDU_LAT_DEG, KATHMANDU_LON_DEG, KATHMANDU_PRESSURE_HPA, KATHMANDU_TEMP_CELSIUS,
};
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

    #[test]
    fn test_topocentric_horizon_dip_and_refraction() {
        let dip_ktm = geometric_horizon_dip_deg(KATHMANDU_ELEVATION_M);
        assert!((dip_ktm - 1.096306).abs() < 1e-4);
        assert_eq!(geometric_horizon_dip_deg(0.0), 0.0);

        let refr_sea = atmospheric_refraction_horizon_deg(1010.0, 10.0);
        let refr_ktm =
            atmospheric_refraction_horizon_deg(KATHMANDU_PRESSURE_HPA, KATHMANDU_TEMP_CELSIUS);
        assert!((refr_sea - 34.0 / 60.0).abs() < 1e-6);
        assert!(refr_ktm > 0.45 && refr_ktm < 0.48);
        assert!(refr_ktm < refr_sea);

        let z_ktm = topocentric_sunrise_zenith_deg(
            KATHMANDU_ELEVATION_M,
            KATHMANDU_PRESSURE_HPA,
            KATHMANDU_TEMP_CELSIUS,
        );
        assert!(z_ktm > 91.80 && z_ktm < 91.86);
    }

    #[test]
    fn test_kathmandu_solstice_sunrise_altitude_shift() {
        let jd_summer_solstice = gregorian_to_jd(2026, 6, 21.0);
        let jd_winter_solstice = gregorian_to_jd(2026, 12, 21.0);

        let summer =
            kathmandu_topocentric_sunrise(jd_summer_solstice, TopocentricAtmosphere::kathmandu());
        let winter =
            kathmandu_topocentric_sunrise(jd_winter_solstice, TopocentricAtmosphere::kathmandu());

        // At 1,400 m elevation in Kathmandu, sunrise occurs ~260–315 seconds (~4.4–5.2 min) earlier than at sea level
        assert!(summer.altitude_advance_seconds > 260.0 && summer.altitude_advance_seconds < 320.0);
        assert!(winter.altitude_advance_seconds > 260.0 && winter.altitude_advance_seconds < 320.0);
        assert!(summer.sunrise_jd_utc < summer.sea_level_sunrise_jd_utc);
        assert!(winter.sunrise_jd_utc < winter.sea_level_sunrise_jd_utc);
    }

    #[test]
    fn test_borderline_sankranti_civil_cutoff_resolution() {
        // Construct a synthetic borderline transit ~2.1 minutes before the 23:48:28.8 NST cutoff (nst_fraction = 0.9905)
        let base_jd_nst_midnight = 2_461_346.5 - (5.75 / 24.0);
        let borderline_transit_jd = base_jd_nst_midnight + 0.9905;

        let ktm_res =
            resolve_sankranti_civil_day(borderline_transit_jd, TopocentricAtmosphere::kathmandu());
        let sea_res =
            resolve_sankranti_civil_day(borderline_transit_jd, TopocentricAtmosphere::sea_level());

        assert!(ktm_res.is_borderline);
        assert!(ktm_res.cutoff_margin_minutes.abs() <= BORDERLINE_CUTOFF_WINDOW_MINUTES);
        // At 1,400m Kathmandu (856 hPa), transit is before the cutoff -> Gate 1 stays on current day,
        // whereas flat sea-level horizon rolls Gate 1 to the following day (+1.0 JD).
        assert_eq!(
            sea_res.civil_start_jd_utc - ktm_res.civil_start_jd_utc,
            1.0
        );
    }
}

