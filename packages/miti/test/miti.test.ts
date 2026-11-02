import { describe, expect, it } from 'vitest';
import {
  Miti,
  apparentTropicalSolarLongitudeDeg,
  findSankrantiTransit,
  gregorianToJd,
  jdToGregorian,
  kathmanduSunriseJdUtc,
  lahiriPanchangaAyanamsaDeg,
  siderealSolarLongitudeDeg,
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
