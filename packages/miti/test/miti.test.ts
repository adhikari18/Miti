import { describe, expect, it } from 'vitest';
import {
  KATHMANDU_ELEVATION_M,
  KATHMANDU_PRESSURE_HPA,
  KATHMANDU_TEMP_CELSIUS,
  Miti,
  apparentTropicalSolarLongitudeDeg,
  atmosphericRefractionHorizonDeg,
  findSankrantiTransit,
  geometricHorizonDipDeg,
  gregorianToJd,
  jdToGregorian,
  kathmanduSunriseJdUtc,
  lahiriPanchangaAyanamsaDeg,
  siderealSolarLongitudeDeg,
  solarHorizontalParallaxDeg,
  solarRadiusVectorAu,
  solarSemidiameterDeg,
  topocentricSunriseZenithAtJdDeg,
  topocentricSunriseZenithDeg,
} from '../src/index.js';

describe('Miti Pure-Math Astronomical Bikram Sambat Engine', () => {
  it('converts Gregorian dates to Julian Day Numbers (JDN) and back without loss', () => {
    const jd = gregorianToJd(2026, 11, 2);
    expect(jd).toBe(2461346.5);
    expect(jdToGregorian(jd)).toEqual({ year: 2026, month: 11, day: 2 });
  });

  it('computes VSOP87 tropical longitude and Lahiri Ayanamsa within canonical bounds', () => {
    // J2000.0 epoch (2000-01-01 12:00 UTC -> JD 2451545.0)
    const tropJ2000 = apparentTropicalSolarLongitudeDeg(2451545.0);
    expect(tropJ2000).toBeGreaterThan(280.3);
    expect(tropJ2000).toBeLessThan(280.5);

    const ayanamsaJ2000 = lahiriPanchangaAyanamsaDeg(2451545.0);
    expect(ayanamsaJ2000).toBeGreaterThan(23.85);
    expect(ayanamsaJ2000).toBeLessThan(24.15);
  });

  it('computes Earth-Sun radius vector R_AU and apparent solar semi-diameter (s_0 / R_AU) across perihelion and aphelion', () => {
    const jdPerihelion = gregorianToJd(2026, 1, 3.5);
    const jdAphelion = gregorianToJd(2026, 7, 4.5);

    const rPeri = solarRadiusVectorAu(jdPerihelion);
    const rAph = solarRadiusVectorAu(jdAphelion);

    expect(rPeri).toBeGreaterThan(0.983);
    expect(rPeri).toBeLessThan(0.9836);
    expect(rAph).toBeGreaterThan(1.0164);
    expect(rAph).toBeLessThan(1.017);

    const sdPeriArcsec = solarSemidiameterDeg(jdPerihelion) * 3600;
    const sdAphArcsec = solarSemidiameterDeg(jdAphelion) * 3600;

    expect(sdPeriArcsec).toBeGreaterThan(975.0);
    expect(sdPeriArcsec).toBeLessThan(976.5);
    expect(sdAphArcsec).toBeGreaterThan(943.0);
    expect(sdAphArcsec).toBeLessThan(944.5);
    expect(sdPeriArcsec).toBeGreaterThan(sdAphArcsec);

    const parallaxArcsec = solarHorizontalParallaxDeg(jdPerihelion) * 3600;
    expect(parallaxArcsec).toBeGreaterThan(8.9);
    expect(parallaxArcsec).toBeLessThan(9.0);
  });

  it('computes elevation horizon dip (Δd = 0.0293°√h) and 856 hPa barometric refraction for 1,400m Kathmandu', () => {
    const dipKtm = geometricHorizonDipDeg(KATHMANDU_ELEVATION_M);
    expect(dipKtm).toBeCloseTo(1.096306, 5);
    expect(geometricHorizonDipDeg(0)).toBe(0);

    const refrSea = atmosphericRefractionHorizonDeg(1010.0, 10.0);
    const refrKtm = atmosphericRefractionHorizonDeg(
      KATHMANDU_PRESSURE_HPA,
      KATHMANDU_TEMP_CELSIUS
    );
    expect(refrSea).toBeCloseTo(34.0 / 60.0, 6);
    expect(refrKtm).toBeGreaterThan(0.46);
    expect(refrKtm).toBeLessThan(0.475);
    expect(refrKtm).toBeLessThan(refrSea);

    const zMeanKtm = topocentricSunriseZenithDeg(
      KATHMANDU_ELEVATION_M,
      KATHMANDU_PRESSURE_HPA,
      KATHMANDU_TEMP_CELSIUS
    );
    expect(zMeanKtm).toBeGreaterThan(91.82);
    expect(zMeanKtm).toBeLessThan(91.84);

    // At perihelion (larger solar semi-diameter), topocentric sunrise zenith is larger than at aphelion
    const jdPeri = gregorianToJd(2026, 1, 3.5);
    const jdAph = gregorianToJd(2026, 7, 4.5);
    const zPeri = topocentricSunriseZenithAtJdDeg(jdPeri);
    const zAph = topocentricSunriseZenithAtJdDeg(jdAph);
    expect(zPeri).toBeGreaterThan(zAph);
    expect((zPeri - zAph) * 3600).toBeCloseTo(32.0, 0);
  });

  it('converges to sub-arcsecond accuracy for all 12 Rashi Sankranti transits', () => {
    for (let month = 1; month <= 12; month++) {
      const event = findSankrantiTransit(2083, month);
      const targetLon = (month - 1) * 30;
      const actualLon = siderealSolarLongitudeDeg(event.transitJdUtc);
      const angularError = Math.min(
        Math.abs(actualLon - targetLon),
        360 - Math.abs(actualLon - targetLon)
      );
      expect(angularError).toBeLessThan(1e-7);
    }
  });

  it('computes Kathmandu topocentric sunrise (27.7172° N, 85.3240° E) near 05:15–06:55 NST', () => {
    const jdMidnightUtc = gregorianToJd(2026, 11, 2);
    const sunriseJd = kathmanduSunriseJdUtc(jdMidnightUtc);
    const utcHours = (sunriseJd - jdMidnightUtc) * 24;
    const nstHours = utcHours + 5.75;
    expect(nstHours).toBeGreaterThan(5.0);
    expect(nstHours).toBeLessThan(7.0);
  });

  it('converts 2026-11-02 to 2083-07-17 (२०८३-०७-१७ Kartik 17, 2083 B.S.) and formats tokens', () => {
    const mitiDate = Miti.fromGregorian('2026-11-02');
    expect(mitiDate.year).toBe(2083);
    expect(mitiDate.month).toBe(7);
    expect(mitiDate.day).toBe(17);
    expect(mitiDate.monthNameEn).toBe('Kartik');
    expect(mitiDate.monthNameNe).toBe('कार्तिक');

    expect(mitiDate.format('YYYY-MM-DD')).toBe('2083-07-17');
    expect(mitiDate.format('YYYY-MM-DD', { locale: 'ne' })).toBe('२०८३-०७-१७');
    expect(mitiDate.format('YYYY MMMM DD, dddd', { locale: 'ne' })).toBe(
      '२०८३ कार्तिक १७, सोमबार'
    );

    const roundTrip = mitiDate.toGregorian();
    expect(roundTrip.toISOString().slice(0, 10)).toBe('2026-11-02');
  });

  it('parses Devanagari B.S. strings via Miti.fromBikramSambat and round-trips cleanly', () => {
    const parsed = Miti.fromBikramSambat('२०८३-०७-१७');
    expect(parsed.year).toBe(2083);
    expect(parsed.month).toBe(7);
    expect(parsed.day).toBe(17);
    expect(parsed.toGregorianString()).toBe('2026-11-02');
  });

  it('verifies canonical month-length invariants (29..32 days) and annual totals across B.S. 2000–2090', () => {
    for (const bsYear of [2000, 2020, 2040, 2060, 2080, 2081, 2082, 2083, 2085, 2090]) {
      const lengths = Miti.getMonthLengths(bsYear);
      expect(lengths).toHaveLength(12);
      for (const len of lengths) {
        expect(len).toBeGreaterThanOrEqual(29);
        expect(len).toBeLessThanOrEqual(32);
      }
      const totalDays = lengths.reduce((a, b) => a + b, 0);
      expect(totalDays === 365 || totalDays === 366).toBe(true);
    }
  });
});

