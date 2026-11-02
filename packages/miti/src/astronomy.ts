/**
 * @module astronomy
 * Pure-math astronomical pipeline mirroring `crates/miti-core`:
 * 1. Gregorian UTC <-> Julian Day Number (JDN) & Terrestrial Time (JDE)
 * 2. VSOP87 / Meeus apparent tropical solar longitude
 * 3. Chitra Paksha / Lahiri Ayanamsa sidereal precession
 * 4. Kathmandu topocentric sunrise (27.7172° N, 85.3240° E, UTC+05:45)
 * 5. Newton-Raphson Sankranti (solar Rashi transit) solver
 */

export const KATHMANDU_LAT_DEG = 27.7172;
export const KATHMANDU_LON_DEG = 85.324;
export const KATHMANDU_ELEVATION_M = 1400.0;
export const KATHMANDU_PRESSURE_HPA = 856.0;
export const KATHMANDU_TEMP_CELSIUS = 18.0;
export const NST_OFFSET_DAYS = 5.75 / 24.0;
export const SUNRISE_ZENITH_DEG = 90.8333;
export const HORIZON_DIP_COEFF_DEG = 0.0293;
export const STANDARD_HORIZON_REFRACTION_DEG = 34.0 / 60.0;
export const CIVIL_CUTOFF_NST_FRACTION = 0.992;
export const BORDERLINE_CUTOFF_WINDOW_MINUTES = 4.0;
export const SIDEREAL_YEAR_DAYS = 365.256363004;

/** Canonical apparent solar angular semi-diameter at `1.0 AU` in arcseconds (`959.63"`). */
export const SOLAR_SEMIDIAMETER_1AU_ARCSEC = 959.63;

/** Canonical equatorial horizontal solar parallax at `1.0 AU` in arcseconds (`8.794"`). */
export const SOLAR_PARALLAX_1AU_ARCSEC = 8.794;

export interface GregorianYMD {
  year: number;
  month: number;
  day: number;
}

/**
 * Normalizes any angle in degrees into `[0, 360)`.
 */
export function normalizeDeg(deg: number): number {
  const r = deg % 360;
  return r < 0 ? r + 360 : r;
}

/**
 * Computes the signed shortest angular difference `targetDeg - currentDeg` in `[-180, 180)`.
 */
export function signedAngleDiff(targetDeg: number, currentDeg: number): number {
  let diff = (targetDeg - currentDeg) % 360;
  if (diff < -180) diff += 360;
  else if (diff >= 180) diff -= 360;
  return diff;
}

/**
 * Converts a proleptic Gregorian UTC date to a continuous Julian Day (JD).
 */
export function gregorianToJd(year: number, month: number, dayWithFraction: number): number {
  const y = month <= 2 ? year - 1 : year;
  const m = month <= 2 ? month + 12 : month;

  const a = Math.floor(y / 100);
  const b = 2 - a + Math.floor(a / 4);

  return (
    Math.floor(365.25 * (y + 4716)) +
    Math.floor(30.6001 * (m + 1)) +
    dayWithFraction +
    b -
    1524.5
  );
}

/**
 * Converts a continuous Julian Day (at 00:00 UTC) back to proleptic Gregorian `{ year, month, day }`.
 */
export function jdToGregorian(jd: number): GregorianYMD {
  const jdShifted = jd + 0.5;
  const z = Math.floor(jdShifted);
  const f = jdShifted - z;

  let a = z;
  if (z >= 2299161) {
    const alpha = Math.floor((z - 1867216.25) / 36524.25);
    a = z + 1 + alpha - Math.floor(alpha / 4);
  }

  const b = a + 1524;
  const c = Math.floor((b - 122.1) / 365.25);
  const d = Math.floor(365.25 * c);
  const e = Math.floor((b - d) / 30.6001);

  const day = Math.floor(b - d - Math.floor(30.6001 * e) + f);
  const month = e < 14 ? e - 1 : e - 13;
  const year = month > 2 ? c - 4716 : c - 4715;

  return { year, month, day };
}

/**
 * NASA Espenak-Meeus polynomial approximation for ΔT = TT - UT (in seconds).
 */
export function deltaTSeconds(yearDecimal: number): number {
  if (yearDecimal >= 1900 && yearDecimal < 2005) {
    const t = (yearDecimal - 2000) / 100;
    return 63.86 + 33.45 * t;
  }
  if (yearDecimal >= 2005 && yearDecimal <= 2050) {
    const t = yearDecimal - 2000;
    return 62.92 + 0.32217 * t + 0.005589 * t * t;
  }
  const u = (yearDecimal - 1820) / 100;
  return -20 + 32 * u * u;
}

/**
 * Converts UTC Julian Day to Julian Ephemeris Day (JDE = JD_UTC + ΔT / 86400).
 */
export function jdUtcToJde(jdUtc: number): number {
  const approxYear = 2000 + (jdUtc - 2451545.0) / 365.25;
  return jdUtc + deltaTSeconds(approxYear) / 86400.0;
}

const DEG_TO_RAD = Math.PI / 180;
const RAD_TO_DEG = 180 / Math.PI;

/**
 * Computes the Sun's apparent tropical longitude (in degrees, `[0, 360)`)
 * at `jdUtc` using truncated VSOP87 / Meeus Ch. 25 periodic terms.
 */
export function apparentTropicalSolarLongitudeDeg(jdUtc: number): number {
  const jde = jdUtcToJde(jdUtc);
  const t = (jde - 2451545.0) / 36525.0;
  const t2 = t * t;
  const t3 = t2 * t;

  // 1. Geometric mean longitude L0
  const l0 = normalizeDeg(280.4664567 + 36000.76982779 * t + 0.0003032028 * t2);

  // 2. Mean anomaly M
  const mDeg = normalizeDeg(
    357.5291092 + 35999.0502909 * t - 0.0001536 * t2 + t3 / 24490000.0
  );
  const m = mDeg * DEG_TO_RAD;

  // 3. Equation of the Center C
  const c =
    (1.914602 - 0.004817 * t - 0.000014 * t2) * Math.sin(m) +
    (0.019993 - 0.000101 * t) * Math.sin(2 * m) +
    0.000289 * Math.sin(3 * m);

  // 4. Planetary and lunar perturbations
  const v1 = (125.04 - 1934.136 * t) * DEG_TO_RAD;
  const pVenus1 = (351.52 + 22518.443 * t) * DEG_TO_RAD;
  const pVenus2 = (253.14 + 45036.886 * t) * DEG_TO_RAD;
  const pJupiter = (20.19 + 32964.467 * t) * DEG_TO_RAD;
  const pMoon = (297.85 + 445267.112 * t) * DEG_TO_RAD;

  const dlPerturb =
    0.00134 * Math.cos(pVenus1) +
    0.00154 * Math.cos(pVenus2) +
    0.002 * Math.cos(pJupiter) +
    0.00179 * Math.sin(pMoon);

  const trueLong = l0 + c + dlPerturb;

  // 5. Apparent longitude (nutation + aberration)
  const apparentLong = trueLong - 0.00569 - 0.00478 * Math.sin(v1);
  return normalizeDeg(apparentLong);
}

/**
 * Computes Chitra Paksha / Lahiri Ayanamsa (in degrees) with Nepali Panchanga calibration.
 */
export function lahiriPanchangaAyanamsaDeg(jdUtc: number): number {
  const jde = jdUtcToJde(jdUtc);
  const tCenturies = (jde - 2451545.0) / 36525.0;

  const baseJ2000 = 23.853056;
  const precessionDeg =
    (5028.796195 * tCenturies + 1.105434 * tCenturies * tCenturies) / 3600.0;

  return baseJ2000 + precessionDeg;
}

/**
 * Computes the Sun's Nirayana (Sidereal) longitude in `[0, 360)` degrees at `jdUtc`.
 */
export function siderealSolarLongitudeDeg(jdUtc: number): number {
  const trop = apparentTropicalSolarLongitudeDeg(jdUtc);
  const ayanamsa = lahiriPanchangaAyanamsaDeg(jdUtc);
  return normalizeDeg(trop - ayanamsa);
}

/**
 * Computes solar declination (degrees) and Equation of Time (minutes) at `jdUtc`.
 */
export function solarDeclinationAndEot(jdUtc: number): {
  declinationDeg: number;
  eotMinutes: number;
} {
  const jde = jdUtcToJde(jdUtc);
  const t = (jde - 2451545.0) / 36525.0;

  const lamRad = apparentTropicalSolarLongitudeDeg(jdUtc) * DEG_TO_RAD;
  const eps0Deg = 23.43929111 - 0.013004167 * t - 0.0000001639 * t * t;
  const omega = (125.04 - 1934.136 * t) * DEG_TO_RAD;
  const epsRad = (eps0Deg + 0.00256 * Math.cos(omega)) * DEG_TO_RAD;

  const sinDec = Math.sin(epsRad) * Math.sin(lamRad);
  const declinationDeg = Math.asin(sinDec) * RAD_TO_DEG;

  const l0Rad = normalizeDeg(280.46646 + 36000.76983 * t) * DEG_TO_RAD;
  const mRad = normalizeDeg(357.52911 + 35999.05029 * t) * DEG_TO_RAD;
  const e = 0.016708634 - 0.000042037 * t;
  const y = Math.pow(Math.tan(epsRad / 2), 2);

  const eotRad =
    y * Math.sin(2 * l0Rad) -
    2 * e * Math.sin(mRad) +
    4 * e * y * Math.sin(mRad) * Math.cos(2 * l0Rad) -
    0.5 * y * y * Math.sin(4 * l0Rad) -
    1.25 * e * e * Math.sin(2 * mRad);

  const eotMinutes = eotRad * RAD_TO_DEG * 4.0;
  return { declinationDeg, eotMinutes };
}

/**
 * Computes the Earth–Sun radius vector `R` in Astronomical Units (`AU`) at `jdUtc`
 * using Meeus Ch. 25 elliptic orbit terms plus primary planetary radial perturbations.
 *
 * Ranges from `~0.98329 AU` at perihelion (early January) to `~1.01671 AU` at aphelion (early July).
 */
export function solarRadiusVectorAu(jdUtc: number): number {
  const jde = jdUtcToJde(jdUtc);
  const t = (jde - 2451545.0) / 36525.0;
  const t2 = t * t;
  const t3 = t2 * t;

  const e = 0.016708634 - 0.000042037 * t - 0.0000001267 * t2;

  const mDeg = normalizeDeg(
    357.5291092 + 35999.0502909 * t - 0.0001536 * t2 + t3 / 24490000.0
  );
  const m = mDeg * DEG_TO_RAD;

  const cDeg =
    (1.914602 - 0.004817 * t - 0.000014 * t2) * Math.sin(m) +
    (0.019993 - 0.000101 * t) * Math.sin(2 * m) +
    0.000289 * Math.sin(3 * m);

  const nuRad = (mDeg + cDeg) * DEG_TO_RAD;
  const rKepler = (1.000001018 * (1.0 - e * e)) / (1.0 + e * Math.cos(nuRad));

  const pVenus1 = (351.52 + 22518.443 * t) * DEG_TO_RAD;
  const pVenus2 = (253.14 + 45036.886 * t) * DEG_TO_RAD;
  const pJupiter = (20.19 + 32964.467 * t) * DEG_TO_RAD;
  const pMoon = (297.85 + 445267.112 * t) * DEG_TO_RAD;

  const drPerturb =
    0.00000543 * Math.sin(pVenus1) +
    0.00001575 * Math.sin(pVenus2) +
    0.00001627 * Math.sin(pJupiter) +
    0.00003076 * Math.cos(pMoon);

  return rKepler + drPerturb;
}

/**
 * Computes the apparent angular semi-diameter of the Sun (`s` in degrees) at `jdUtc`:
 * `s = (s_0 / R_AU) / 3600`, where `s_0 = 959.63"` at `1 AU`.
 */
export function solarSemidiameterDeg(jdUtc: number): number {
  const rAu = solarRadiusVectorAu(jdUtc);
  return SOLAR_SEMIDIAMETER_1AU_ARCSEC / 3600.0 / rAu;
}

/**
 * Computes the Sun's equatorial horizontal parallax (`π_☉` in degrees) at `jdUtc`:
 * `π_☉ = (8.794" / R_AU) / 3600`.
 */
export function solarHorizontalParallaxDeg(jdUtc: number): number {
  const rAu = solarRadiusVectorAu(jdUtc);
  return SOLAR_PARALLAX_1AU_ARCSEC / 3600.0 / rAu;
}

/**
 * Computes the elevation-dependent geometric horizon depression angle `Δd` in degrees:
 * `Δd = 0.0293° * √max(0, elevationM)`.
 *
 * At Kathmandu's elevation (`1,400 m`), `Δd ≈ 1.096306°` (`1° 05' 47"`).
 */
export function geometricHorizonDipDeg(elevationM: number): number {
  const clampedH = elevationM > 0 ? elevationM : 0;
  return HORIZON_DIP_COEFF_DEG * Math.sqrt(clampedH);
}

/**
 * Computes topocentric atmospheric refraction `R_atm` (in degrees) at the horizon,
 * scaled for local barometric pressure `pressureHpa` (in hPa) and ambient temperature
 * `tempCelsius` (in °C) per Meeus Ch. 16 / Saemundsson-Bennett relation:
 *
 * `R_atm = (34' / 60) * (P / 1010.0) * (283.0 / (273.0 + T))`
 */
export function atmosphericRefractionHorizonDeg(
  pressureHpa: number,
  tempCelsius: number
): number {
  const pFactor = Math.max(0, pressureHpa) / 1010.0;
  const kelvin = Math.max(150.0, 273.0 + tempCelsius);
  const tFactor = 283.0 / kelvin;
  return STANDARD_HORIZON_REFRACTION_DEG * pFactor * tFactor;
}

/**
 * Computes the topocentric apparent upper-limb sunrise zenith distance (`Z` in degrees)
 * for an observer at `elevationM`, `pressureHpa`, and `tempCelsius` using mean `1 AU`
 * solar semi-diameter (`s_0 = 959.63"`) and horizontal parallax (`π_0 = 8.794"`):
 *
 * `Z = 90° + Δd(h) + R_atm(P, T) + (s_0 - π_0) / 3600`
 */
export function topocentricSunriseZenithDeg(
  elevationM: number = KATHMANDU_ELEVATION_M,
  pressureHpa: number = KATHMANDU_PRESSURE_HPA,
  tempCelsius: number = KATHMANDU_TEMP_CELSIUS
): number {
  const dipDeg = geometricHorizonDipDeg(elevationM);
  const refractionDeg = atmosphericRefractionHorizonDeg(pressureHpa, tempCelsius);
  const meanSdMinusParallaxDeg =
    (SOLAR_SEMIDIAMETER_1AU_ARCSEC - SOLAR_PARALLAX_1AU_ARCSEC) / 3600.0;
  return 90.0 + dipDeg + refractionDeg + meanSdMinusParallaxDeg;
}

/**
 * Computes the date-specific topocentric apparent sunrise zenith distance (`Z` in degrees)
 * at `jdUtc`, incorporating Earth–Sun radius vector variation `s(JD) = s_0 / R_AU(JD)`
 * and solar horizontal parallax `π_☉(JD) = π_0 / R_AU(JD)`:
 *
 * `Z(JD) = 90° + Δd(h) + R_atm(P, T) + s(JD) - π_☉(JD)`
 */
export function topocentricSunriseZenithAtJdDeg(
  jdUtc: number,
  elevationM: number = KATHMANDU_ELEVATION_M,
  pressureHpa: number = KATHMANDU_PRESSURE_HPA,
  tempCelsius: number = KATHMANDU_TEMP_CELSIUS
): number {
  const dipDeg = geometricHorizonDipDeg(elevationM);
  const refractionDeg = atmosphericRefractionHorizonDeg(pressureHpa, tempCelsius);
  const sdDeg = solarSemidiameterDeg(jdUtc);
  const parallaxDeg = solarHorizontalParallaxDeg(jdUtc);
  return 90.0 + dipDeg + refractionDeg + sdDeg - parallaxDeg;
}

/**
 * Topocentric atmospheric and elevation parameters for an observing station.
 */
export interface TopocentricAtmosphere {
  /** Observer elevation above mean sea level in meters (`1,400 m` for Kathmandu). */
  elevationM: number;
  /** Local barometric pressure in hectopascals (`856.0 hPa` for Kathmandu). */
  pressureHpa: number;
  /** Local ambient sunrise temperature in degrees Celsius (`18.0 °C` for Kathmandu). */
  tempCelsius: number;
}

/** Canonical Kathmandu Valley high-altitude atmosphere (`1,400 m`, `856 hPa`, `18 °C`). */
export const KATHMANDU_DEFAULT_ATMOSPHERE: Readonly<TopocentricAtmosphere> = {
  elevationM: KATHMANDU_ELEVATION_M,
  pressureHpa: KATHMANDU_PRESSURE_HPA,
  tempCelsius: KATHMANDU_TEMP_CELSIUS,
};

/** Standard sea-level reference atmosphere (`0 m`, `1,010 hPa`, `10 °C`). */
export const SEA_LEVEL_STANDARD_ATMOSPHERE: Readonly<TopocentricAtmosphere> = {
  elevationM: 0.0,
  pressureHpa: 1010.0,
  tempCelsius: 10.0,
};

/**
 * Detailed diagnostic breakdown of Kathmandu topocentric sunrise vs. standard sea-level sunrise.
 */
export interface TopocentricSunriseMetrics {
  /** Two-pass refined UTC Julian Day of topocentric sunrise (with elevation dip, refraction, and `s_0 / R_AU`). */
  sunriseJdUtc: number;
  /** Two-pass refined UTC Julian Day of standard flat-horizon sea-level sunrise (`Z = 90.8333°`). */
  seaLevelSunriseJdUtc: number;
  /** Effective topocentric sunrise zenith distance in degrees (`~91.83°` at `1,400 m`). */
  zenithDeg: number;
  /** Geometric horizon depression `Δd = 0.0293° √h` in degrees. */
  horizonDipDeg: number;
  /** Pressure- and temperature-compensated horizon refraction `R_atm` in degrees. */
  refractionDeg: number;
  /** Apparent solar semi-diameter `s_0 / R_AU` in degrees at the sunrise instant. */
  solarSemidiameterDeg: number;
  /** Equatorial horizontal solar parallax `π_0 / R_AU` in degrees at the sunrise instant. */
  solarParallaxDeg: number;
  /** Number of seconds by which high-altitude sunrise precedes sea-level sunrise (`(seaLevel - topocentric) * 86400`). */
  altitudeAdvanceSeconds: number;
}

function solveHourAngleSunriseJd(
  jdUtcMidnight: number,
  jdEval: number,
  zenithDeg: number
): number {
  const { declinationDeg, eotMinutes } = solarDeclinationAndEot(jdEval);
  const latRad = KATHMANDU_LAT_DEG * DEG_TO_RAD;
  const decRad = declinationDeg * DEG_TO_RAD;
  const zenithRad = zenithDeg * DEG_TO_RAD;

  const cosH0 =
    (Math.cos(zenithRad) - Math.sin(latRad) * Math.sin(decRad)) /
    (Math.cos(latRad) * Math.cos(decRad));
  const clampedCos = Math.max(-1, Math.min(1, cosH0));
  const h0Deg = Math.acos(clampedCos) * RAD_TO_DEG;

  const solarNoonUtcMin = 720.0 - 4.0 * KATHMANDU_LON_DEG - eotMinutes;
  const sunriseUtcMin = solarNoonUtcMin - 4.0 * h0Deg;

  return jdUtcMidnight + sunriseUtcMin / 1440.0;
}

/**
 * Computes the two-pass refined high-altitude topocentric sunrise and diagnostic breakdown
 * for Kathmandu (`27.7172° N, 85.3240° E`) on the civil date whose UTC midnight is `jdUtcMidnight`.
 */
export function kathmanduTopocentricSunrise(
  jdUtcMidnight: number,
  atmosphere: TopocentricAtmosphere = KATHMANDU_DEFAULT_ATMOSPHERE
): TopocentricSunriseMetrics {
  const jdProbe = jdUtcMidnight + 0.01;

  // Two-pass sea-level reference sunrise (Z = 90.8333°)
  const seaPass1 = solveHourAngleSunriseJd(jdUtcMidnight, jdProbe, SUNRISE_ZENITH_DEG);
  const seaLevelSunriseJdUtc = solveHourAngleSunriseJd(
    jdUtcMidnight,
    seaPass1,
    SUNRISE_ZENITH_DEG
  );

  // Pass 1 topocentric sunrise using initial probe
  const zPass1 = topocentricSunriseZenithAtJdDeg(
    jdProbe,
    atmosphere.elevationM,
    atmosphere.pressureHpa,
    atmosphere.tempCelsius
  );
  const topoPass1 = solveHourAngleSunriseJd(jdUtcMidnight, jdProbe, zPass1);

  // Pass 2 topocentric sunrise evaluated at Pass 1 sunrise instant
  const horizonDipDeg = geometricHorizonDipDeg(atmosphere.elevationM);
  const refractionDeg = atmosphericRefractionHorizonDeg(
    atmosphere.pressureHpa,
    atmosphere.tempCelsius
  );
  const sdDeg = solarSemidiameterDeg(topoPass1);
  const parallaxDeg = solarHorizontalParallaxDeg(topoPass1);
  const zenithDeg = 90.0 + horizonDipDeg + refractionDeg + sdDeg - parallaxDeg;

  const sunriseJdUtc = solveHourAngleSunriseJd(jdUtcMidnight, topoPass1, zenithDeg);
  const altitudeAdvanceSeconds = (seaLevelSunriseJdUtc - sunriseJdUtc) * 86400.0;

  return {
    sunriseJdUtc,
    seaLevelSunriseJdUtc,
    zenithDeg,
    horizonDipDeg,
    refractionDeg,
    solarSemidiameterDeg: sdDeg,
    solarParallaxDeg: parallaxDeg,
    altitudeAdvanceSeconds,
  };
}

/**
 * Computes the UTC Julian Day of local topocentric sunrise in Kathmandu (`1,400 m`, `856 hPa`)
 * for a civil date whose UTC midnight is `jdUtcMidnight`.
 */
export function kathmanduSunriseJdUtc(jdUtcMidnight: number): number {
  return kathmanduTopocentricSunrise(jdUtcMidnight, KATHMANDU_DEFAULT_ATMOSPHERE)
    .sunriseJdUtc;
}

/**
 * Diagnostic resolution for a Sankranti solar transit relative to the Kathmandu civil cutoff,
 * including topocentric horizon dip (`Δd = 0.0293°√h`), barometric refraction (`856 hPa`),
 * and apparent solar semi-diameter (`s_0 / R_AU`) corrections for borderline transits (`±4 min`).
 */
export interface BorderlineSankrantiResolution {
  /** Resolved 00:00 UTC Julian Day of the first civil day (*Gate 1* / १ गते). */
  civilStartJdUtc: number;
  /** Uncorrected sea-level civil start 00:00 UTC Julian Day (without `1,400 m` elevation dip). */
  seaLevelCivilStartJdUtc: number;
  /** Signed time margin in minutes (`transitNst - cutoffNst`); `< 0` stays on current day, `> 0` rolls to next day. */
  cutoffMarginMinutes: number;
  /** `true` if the Sankranti occurs within `±4.0` minutes of the civil transition cutoff. */
  isBorderline: boolean;
  /** Topocentric cutoff shift in seconds induced by elevation dip, pressure refraction, and `s_0 / R_AU`. */
  topocentricCutoffShiftSeconds: number;
  /** High-altitude topocentric sunrise metrics on the resolved *Gate 1* morning. */
  sunriseMetrics: TopocentricSunriseMetrics;
}

/**
 * Resolves an astronomical Sankranti transit (`transitJdUtc`) to its Bikram Sambat
 * civil start day (*Gate 1* / १ गते) using topocentric horizon dip (`Δd = 0.0293°√h`),
 * barometric pressure (`856 hPa`), and apparent solar semi-diameter (`s_0 / R_AU`)
 * calibration for borderline transits within `±4` minutes of the civil cutoff.
 */
export function resolveSankrantiCivilDay(
  transitJdUtc: number,
  atmosphere: TopocentricAtmosphere = KATHMANDU_DEFAULT_ATMOSPHERE
): BorderlineSankrantiResolution {
  const jdNstShifted = transitJdUtc + 0.5 + NST_OFFSET_DAYS;
  const civilDayIndex = Math.floor(jdNstShifted);
  const nstFraction = jdNstShifted - civilDayIndex;

  const zRefKtm = topocentricSunriseZenithDeg(
    KATHMANDU_ELEVATION_M,
    KATHMANDU_PRESSURE_HPA,
    KATHMANDU_TEMP_CELSIUS
  );
  const zActual = topocentricSunriseZenithAtJdDeg(
    transitJdUtc,
    atmosphere.elevationM,
    atmosphere.pressureHpa,
    atmosphere.tempCelsius
  );
  const zSeaLevel = topocentricSunriseZenithAtJdDeg(transitJdUtc, 0.0, 1010.0, 10.0);

  const deltaZFromKtmRef = zActual - zRefKtm;
  const deltaZFromSeaLevel = zActual - zSeaLevel;

  const topocentricShiftDays = deltaZFromKtmRef * (4.0 / 1440.0);
  const seaLevelShiftDays = (zSeaLevel - zRefKtm) * (4.0 / 1440.0);

  const effectiveCutoffFraction = CIVIL_CUTOFF_NST_FRACTION + topocentricShiftDays;
  const seaLevelCutoffFraction = CIVIL_CUTOFF_NST_FRACTION + seaLevelShiftDays;

  const cutoffMarginMinutes = (nstFraction - effectiveCutoffFraction) * 1440.0;
  const isBorderline =
    Math.abs(cutoffMarginMinutes) <= BORDERLINE_CUTOFF_WINDOW_MINUTES;

  const effectiveDayIndex =
    nstFraction > effectiveCutoffFraction ? civilDayIndex + 1 : civilDayIndex;
  const seaLevelDayIndex =
    nstFraction > seaLevelCutoffFraction ? civilDayIndex + 1 : civilDayIndex;

  const civilStartJdUtc = effectiveDayIndex - 0.5;
  const seaLevelCivilStartJdUtc = seaLevelDayIndex - 0.5;
  const sunriseMetrics = kathmanduTopocentricSunrise(civilStartJdUtc, atmosphere);

  return {
    civilStartJdUtc,
    seaLevelCivilStartJdUtc,
    cutoffMarginMinutes,
    isBorderline,
    topocentricCutoffShiftSeconds: deltaZFromSeaLevel * 240.0,
    sunriseMetrics,
  };
}

/**
 * Maps an astronomical Sankranti transit instant (`transitJdUtc`) to the 00:00 UTC
 * Julian Day of the first civil day (Gate 1 / १ गते) of the Bikram Sambat month.
 */
export function sankrantiToCivilDayJdUtc(transitJdUtc: number): number {
  return resolveSankrantiCivilDay(transitJdUtc, KATHMANDU_DEFAULT_ATMOSPHERE)
    .civilStartJdUtc;
}
