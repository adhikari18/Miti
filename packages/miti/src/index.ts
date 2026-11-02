import {
  gregorianToJd,
  jdToGregorian,
  KATHMANDU_DEFAULT_ATMOSPHERE,
  lahiriPanchangaAyanamsaDeg,
  resolveSankrantiCivilDay,
  SIDEREAL_YEAR_DAYS,
  siderealSolarLongitudeDeg,
  signedAngleDiff,
  type BorderlineSankrantiResolution,
  type GregorianYMD,
  type TopocentricAtmosphere,
} from './astronomy.js';

export * from './astronomy.js';

export const BS_MONTH_NAMES_EN = [
  'Baisakh',
  'Jestha',
  'Asar',
  'Shrawan',
  'Bhadra',
  'Ashwin',
  'Kartik',
  'Mangsir',
  'Poush',
  'Magh',
  'Falgun',
  'Chaitra',
] as const;

export const BS_MONTH_NAMES_NE = [
  'बैशाख',
  'जेठ',
  'असार',
  'साउन',
  'भदौ',
  'असोज',
  'कार्तिक',
  'मंसिर',
  'पुस',
  'माघ',
  'फागुन',
  'चैत',
] as const;

export const WEEKDAY_NAMES_EN = [
  'Sunday',
  'Monday',
  'Tuesday',
  'Wednesday',
  'Thursday',
  'Friday',
  'Saturday',
] as const;

export const WEEKDAY_NAMES_NE = [
  'आइतबार',
  'सोमबार',
  'मंगलबार',
  'बुधबार',
  'बिहिबार',
  'शुक्रबार',
  'शनिबार',
] as const;

export const RASHI_NAMES_EN = [
  'Mesha (Aries)',
  'Vrishabha (Taurus)',
  'Mithuna (Gemini)',
  'Karkata (Cancer)',
  'Simha (Leo)',
  'Kanya (Virgo)',
  'Tula (Libra)',
  'Vrishchika (Scorpio)',
  'Dhanu (Sagittarius)',
  'Makara (Capricorn)',
  'Kumbha (Aquarius)',
  'Meena (Pisces)',
] as const;

export const RASHI_NAMES_NE = [
  'मेष',
  'वृष',
  'मिथुन',
  'कर्कट',
  'सिंह',
  'कन्या',
  'तुला',
  'वृश्चिक',
  'धनु',
  'मकर',
  'कुम्भ',
  'मीन',
] as const;

const DEVANAGARI_DIGITS = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'] as const;

/**
 * Converts ASCII digits (`0-9`) in a string or number into Nepali Devanagari digits (`०-९`).
 */
export function toDevanagariDigits(value: string | number): string {
  return String(value).replace(/[0-9]/g, (d) => DEVANAGARI_DIGITS[Number(d)]);
}

/**
 * Converts Nepali Devanagari digits (`०-९`) in a string into ASCII digits (`0-9`).
 */
export function fromDevanagariDigits(value: string): string {
  return value.replace(/[०-९]/g, (ch) => String(ch.charCodeAt(0) - 0x0966));
}

/**
 * Astronomical Sankranti (solar zodiac transit) metadata for a Bikram Sambat month.
 */
export interface SankrantiEvent {
  /** Bikram Sambat year (e.g., `2083`). */
  bsYear: number;
  /** Bikram Sambat month (`1 = Baisakh` .. `12 = Chaitra`). */
  bsMonth: number;
  /** Sidereal Rashi name in English (e.g., `"Tula (Libra)"`). */
  rashiNameEn: string;
  /** Sidereal Rashi name in Nepali (e.g., `"तुला"`). */
  rashiNameNe: string;
  /** Target Nirayana solar longitude (`0, 30, ..., 330` degrees). */
  targetLongitudeDeg: number;
  /** Exact UTC Julian Day of the solar Rashi transit. */
  transitJdUtc: number;
  /** Exact UTC ISO-8601 timestamp of the solar Rashi transit. */
  transitIsoUtc: string;
  /** UTC Julian Day of Kathmandu local sunrise on Gate 1. */
  kathmanduSunriseJdUtc: number;
  /** Julian Day (at 00:00 UTC) of civil Gate 1 (१ गते). */
  civilStartJdUtc: number;
  /** Gregorian civil start date (`YYYY-MM-DD`) of Gate 1. */
  civilStartGregorian: GregorianYMD;
  /** Chitra Paksha / Lahiri Ayanamsa in degrees at the transit instant. */
  ayanamsaDeg: number;
  /** Signed time margin in minutes (`transitNst - cutoffNst`) relative to the Kathmandu civil cutoff. */
  cutoffMarginMinutes: number;
  /** `true` if the Sankranti transit occurs within `±4.0` minutes of the civil transition cutoff. */
  isBorderlineTransit: boolean;
}

/**
 * Formatting options for `MitiDate.format(pattern, options)`.
 */
export interface MitiFormatOptions {
  /**
   * Locale for month/weekday names and default digit system:
   * - `'en'`: English names and ASCII numerals (`2083-07-17`)
   * - `'ne'`: Nepali names and Devanagari numerals (`२०८३-०७-१७`)
   */
  locale?: 'en' | 'ne';
  /**
   * Override digit script independently of `locale`:
   * - `'latn'`: `0-9`
   * - `'deva'`: `०-९`
   */
  numeralSystem?: 'latn' | 'deva';
}

const sankrantiCache = new Map<string, SankrantiEvent>();

function jdToIsoTimestampUtc(jdUtc: number): string {
  const unixMs = Math.round((jdUtc - 2440587.5) * 86400000);
  return new Date(unixMs).toISOString();
}

/**
 * Locates the exact astronomical Sankranti transit for `(bsYear, bsMonth)`
 * using Newton-Raphson root-finding over VSOP87 + Chitra Paksha Ayanamsa.
 */
export function findSankrantiTransit(bsYear: number, bsMonth: number): SankrantiEvent {
  const cacheKey = `${bsYear}-${bsMonth}`;
  const cached = sankrantiCache.get(cacheKey);
  if (cached) return cached;

  const normalizedMonth = (((bsMonth - 1) % 12) + 12) % 12;
  const targetLongitudeDeg = normalizedMonth * 30.0;
  const gregYear = bsYear - 57;
  const meshAnchorJd = gregorianToJd(gregYear, 4, 13.5);
  let jd = meshAnchorJd + normalizedMonth * (SIDEREAL_YEAR_DAYS / 12.0);

  for (let iter = 0; iter < 16; iter++) {
    const currentLon = siderealSolarLongitudeDeg(jd);
    const diffDeg = signedAngleDiff(targetLongitudeDeg, currentLon);
    if (Math.abs(diffDeg) < 1e-9) break;

    const dt = 0.001;
    const lonPlus = siderealSolarLongitudeDeg(jd + dt);
    const velocity = Math.max(0.9, Math.min(1.05, signedAngleDiff(lonPlus, currentLon) / dt));
    jd += diffDeg / velocity;
  }

  const resolution = resolveSankrantiCivilDay(jd, KATHMANDU_DEFAULT_ATMOSPHERE);
  const civilStartJdUtc = resolution.civilStartJdUtc;
  const kathmanduSunriseJd = resolution.sunriseMetrics.sunriseJdUtc;
  const ayanamsaDeg = lahiriPanchangaAyanamsaDeg(jd);
  const civilStartGregorian = jdToGregorian(civilStartJdUtc);

  const event: SankrantiEvent = {
    bsYear,
    bsMonth: normalizedMonth + 1,
    rashiNameEn: RASHI_NAMES_EN[normalizedMonth],
    rashiNameNe: RASHI_NAMES_NE[normalizedMonth],
    targetLongitudeDeg,
    transitJdUtc: jd,
    transitIsoUtc: jdToIsoTimestampUtc(jd),
    kathmanduSunriseJdUtc: kathmanduSunriseJd,
    civilStartJdUtc,
    civilStartGregorian,
    ayanamsaDeg: Math.round(ayanamsaDeg * 1e6) / 1e6,
    cutoffMarginMinutes: Math.round(resolution.cutoffMarginMinutes * 1e4) / 1e4,
    isBorderlineTransit: resolution.isBorderline,
  };

  sankrantiCache.set(cacheKey, event);
  return event;
}

/**
 * Computes the exact number of civil days (`29..32`) in `(bsYear, bsMonth)`.
 */
export function daysInBsMonth(bsYear: number, bsMonth: number): number {
  const current = findSankrantiTransit(bsYear, bsMonth);
  const nextYear = bsMonth === 12 ? bsYear + 1 : bsYear;
  const nextMonth = bsMonth === 12 ? 1 : bsMonth + 1;
  const next = findSankrantiTransit(nextYear, nextMonth);
  return Math.round(next.civilStartJdUtc - current.civilStartJdUtc);
}

/**
 * Immutable Bikram Sambat (B.S.) civil date representation with Temporal/Day.js-style ergonomics.
 */
export class MitiDate {
  /** Bikram Sambat year (e.g., `2083`). */
  public readonly year: number;
  /** Bikram Sambat month (`1 = Baisakh` .. `12 = Chaitra`). */
  public readonly month: number;
  /** Bikram Sambat day of month (*Gate* / गते, `1..32`). */
  public readonly day: number;
  /** Day of week (`0 = Sunday / आइतबार` .. `6 = Saturday / शनिबार`). */
  public readonly weekday: number;
  /** Total days in this Bikram Sambat month (`29..32`). */
  public readonly daysInMonth: number;
  /** Underlying UTC midnight Julian Day for this civil date. */
  public readonly jdUtc: number;

  constructor(
    year: number,
    month: number,
    day: number,
    weekday: number,
    daysInMonth: number,
    jdUtc: number
  ) {
    this.year = year;
    this.month = month;
    this.day = day;
    this.weekday = weekday;
    this.daysInMonth = daysInMonth;
    this.jdUtc = jdUtc;
  }

  /** English month name (e.g., `"Kartik"`). */
  public get monthNameEn(): string {
    return BS_MONTH_NAMES_EN[this.month - 1] ?? 'Unknown';
  }

  /** Nepali Devanagari month name (e.g., `"कार्तिक"`). */
  public get monthNameNe(): string {
    return BS_MONTH_NAMES_NE[this.month - 1] ?? '';
  }

  /** English weekday name (e.g., `"Monday"`). */
  public get weekdayNameEn(): string {
    return WEEKDAY_NAMES_EN[this.weekday] ?? '';
  }

  /** Nepali Devanagari weekday name (e.g., `"सोमबार"`). */
  public get weekdayNameNe(): string {
    return WEEKDAY_NAMES_NE[this.weekday] ?? '';
  }

  /** Sidereal Rashi (zodiac sign) governing this solar month. */
  public get rashiNameEn(): string {
    return RASHI_NAMES_EN[this.month - 1] ?? '';
  }

  /** Converts this Bikram Sambat date back to a JavaScript UTC `Date` object. */
  public toGregorian(): Date {
    const { year, month, day } = jdToGregorian(this.jdUtc);
    return new Date(Date.UTC(year, month - 1, day, 0, 0, 0, 0));
  }

  /** Returns the Gregorian `YYYY-MM-DD` string corresponding to this Bikram Sambat date. */
  public toGregorianString(): string {
    const { year, month, day } = jdToGregorian(this.jdUtc);
    return `${String(year).padStart(4, '0')}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  }

  /** Returns the canonical ISO-style Bikram Sambat string (`YYYY-MM-DD`). */
  public toString(): string {
    return this.format('YYYY-MM-DD', { locale: 'en' });
  }

  /** Returns the astronomical `SankrantiEvent` that inaugurated this date's month. */
  public sankranti(): SankrantiEvent {
    return findSankrantiTransit(this.year, this.month);
  }

  /** Returns a new `MitiDate` shifted by `deltaDays` civil days. */
  public addDays(deltaDays: number): MitiDate {
    const nextGreg = jdToGregorian(this.jdUtc + deltaDays);
    return Miti.fromGregorian(
      new Date(Date.UTC(nextGreg.year, nextGreg.month - 1, nextGreg.day))
    );
  }

  /**
   * Formats this Bikram Sambat date using pattern tokens:
   * - `YYYY`: 4-digit year (`2083` or `२०८३`)
   * - `YY`: 2-digit year (`83` or `८३`)
   * - `MMMM`: Full month name (`Kartik` or `कार्तिक`)
   * - `MMM`: Short month name (`Kar` or `कार्तिक`)
   * - `MM`: 2-digit padded month (`07` or `०७`)
   * - `M`: Unpadded month (`7` or `७`)
   * - `DD`: 2-digit padded day (`17` or `१७`)
   * - `D`: Unpadded day (`17` or `१७`)
   * - `dddd`: Full weekday name (`Monday` or `सोमबार`)
   * - `ddd`: Short weekday name (`Mon` or `सोम`)
   */
  public format(pattern = 'YYYY-MM-DD', options: MitiFormatOptions = {}): string {
    const locale = options.locale ?? 'en';
    const useDeva =
      options.numeralSystem === 'deva' ||
      (options.numeralSystem === undefined && locale === 'ne');

    const num = (val: string | number) =>
      useDeva ? toDevanagariDigits(val) : String(val);

    const yyyy = num(String(this.year).padStart(4, '0'));
    const yy = num(String(this.year).slice(-2));
    const mmmm = locale === 'ne' ? this.monthNameNe : this.monthNameEn;
    const mmm = locale === 'ne' ? this.monthNameNe : this.monthNameEn.slice(0, 3);
    const mm = num(String(this.month).padStart(2, '0'));
    const m = num(this.month);
    const dd = num(String(this.day).padStart(2, '0'));
    const d = num(this.day);
    const dddd = locale === 'ne' ? this.weekdayNameNe : this.weekdayNameEn;
    const ddd =
      locale === 'ne'
        ? this.weekdayNameNe.replace(/बार$/, '')
        : this.weekdayNameEn.slice(0, 3);

    const tokenMap: Record<string, string> = {
      YYYY: yyyy,
      YY: yy,
      MMMM: mmmm,
      MMM: mmm,
      MM: mm,
      M: m,
      DD: dd,
      D: d,
      dddd: dddd,
      ddd: ddd,
    };

    return pattern.replace(
      /YYYY|YY|MMMM|MMM|MM|M|DD|D|dddd|ddd/g,
      (match) => tokenMap[match] ?? match
    );
  }
}

function parseGregorianInput(input: Date | string): GregorianYMD {
  if (typeof input === 'string') {
    const normalized = fromDevanagariDigits(input.trim());
    const isoMatch = /^(\d{4})-(\d{1,2})-(\d{1,2})/.exec(normalized);
    if (isoMatch) {
      return {
        year: Number(isoMatch[1]),
        month: Number(isoMatch[2]),
        day: Number(isoMatch[3]),
      };
    }
    const parsed = new Date(normalized);
    if (Number.isNaN(parsed.getTime())) {
      throw new Error(`Invalid Gregorian date string: '${input}'`);
    }
    return {
      year: parsed.getUTCFullYear(),
      month: parsed.getUTCMonth() + 1,
      day: parsed.getUTCDate(),
    };
  }

  if (!(input instanceof Date) || Number.isNaN(input.getTime())) {
    throw new Error('Invalid Date object passed to Miti.fromGregorian().');
  }
  return {
    year: input.getUTCFullYear(),
    month: input.getUTCMonth() + 1,
    day: input.getUTCDate(),
  };
}

/**
 * Primary entry point for the `Miti` Bikram Sambat astronomical calendar engine.
 */
export const Miti = {
  /**
   * Converts a Gregorian `Date` or `YYYY-MM-DD` string into a Bikram Sambat `MitiDate`
   * using pure VSOP87 solar mechanics, Chitra Paksha Ayanamsa, and Kathmandu sunrise calibration.
   */
  fromGregorian(date: Date | string): MitiDate {
    const g = parseGregorianInput(date);
    const targetJd = gregorianToJd(g.year, g.month, g.day);
    const weekday = (((Math.floor(targetJd + 0.5) + 1) % 7) + 7) % 7;

    let bsYear = g.year + 57;
    const baisakhStart = findSankrantiTransit(bsYear, 1).civilStartJdUtc;
    if (targetJd < baisakhStart) {
      bsYear -= 1;
    }

    const yearStartJd = findSankrantiTransit(bsYear, 1).civilStartJdUtc;
    const elapsedDays = Math.max(0, targetJd - yearStartJd);
    let bsMonth = Math.max(1, Math.min(12, Math.floor(elapsedDays / 30.4375) + 1));

    while (true) {
      const mStart = findSankrantiTransit(bsYear, bsMonth).civilStartJdUtc;
      if (targetJd < mStart && bsMonth > 1) {
        bsMonth -= 1;
        continue;
      }
      const nextY = bsMonth === 12 ? bsYear + 1 : bsYear;
      const nextM = bsMonth === 12 ? 1 : bsMonth + 1;
      const nextStart = findSankrantiTransit(nextY, nextM).civilStartJdUtc;
      if (targetJd >= nextStart) {
        if (bsMonth === 12) {
          bsYear += 1;
          bsMonth = 1;
        } else {
          bsMonth += 1;
        }
        continue;
      }

      const day = Math.round(targetJd - mStart) + 1;
      const daysInMonth = Math.round(nextStart - mStart);
      return new MitiDate(bsYear, bsMonth, day, weekday, daysInMonth, targetJd);
    }
  },

  /**
   * Constructs a `MitiDate` directly from a Bikram Sambat `(year, month, day)` tuple
   * or `"YYYY-MM-DD"` / `"२०८३-०७-१७"` string.
   */
  fromBikramSambat(yearOrIso: number | string, month?: number, day?: number): MitiDate {
    let y: number;
    let m: number;
    let d: number;

    if (typeof yearOrIso === 'string') {
      const normalized = fromDevanagariDigits(yearOrIso.trim());
      const parts = /^(\d{4})-(\d{1,2})-(\d{1,2})$/.exec(normalized);
      if (!parts) {
        throw new Error(`Invalid Bikram Sambat date string: '${yearOrIso}'`);
      }
      y = Number(parts[1]);
      m = Number(parts[2]);
      d = Number(parts[3]);
    } else {
      y = yearOrIso;
      m = month ?? 1;
      d = day ?? 1;
    }

    if (m < 1 || m > 12) {
      throw new Error(`Bikram Sambat month must be between 1 and 12 (received ${m}).`);
    }

    const maxDays = daysInBsMonth(y, m);
    if (d < 1 || d > maxDays) {
      throw new Error(
        `Bikram Sambat day ${d} is out of bounds for ${BS_MONTH_NAMES_EN[m - 1]} ${y} (has ${maxDays} days).`
      );
    }

    const mStartJd = findSankrantiTransit(y, m).civilStartJdUtc;
    const targetJd = mStartJd + (d - 1);
    const weekday = (((Math.floor(targetJd + 0.5) + 1) % 7) + 7) % 7;

    return new MitiDate(y, m, d, weekday, maxDays, targetJd);
  },

  /**
   * Returns today's Bikram Sambat `MitiDate` (evaluated in Nepal Standard Time `UTC+05:45`).
   */
  today(): MitiDate {
    const nowUtcMs = Date.now();
    const nstMs = nowUtcMs + (5 * 60 + 45) * 60 * 1000;
    const nstDate = new Date(nstMs);
    return Miti.fromGregorian(
      `${nstDate.getUTCFullYear()}-${String(nstDate.getUTCMonth() + 1).padStart(2, '0')}-${String(nstDate.getUTCDate()).padStart(2, '0')}`
    );
  },

  /**
   * Returns the astronomical `SankrantiEvent` for `(bsYear, bsMonth)`.
   */
  getSankranti(bsYear: number, bsMonth: number): SankrantiEvent {
    return findSankrantiTransit(bsYear, bsMonth);
  },

  /**
   * Dynamically computes the 12 month lengths (`29..32` days) of `bsYear` from solar transits.
   */
  getMonthLengths(bsYear: number): number[] {
    return Array.from({ length: 12 }, (_, idx) => daysInBsMonth(bsYear, idx + 1));
  },

  /**
   * Resolves a Sankranti transit instant (`transitJdUtc`) with high-altitude topocentric
   * refraction (`856 hPa`), horizon dip (`1,400m`), and solar semi-diameter (`s_0 / R_AU`)
   * calibration for borderline transits occurring within `±4` minutes of the civil cutoff.
   */
  resolveBorderlineSankranti(
    transitJdUtc: number,
    atmosphere: TopocentricAtmosphere = KATHMANDU_DEFAULT_ATMOSPHERE
  ): BorderlineSankrantiResolution {
    return resolveSankrantiCivilDay(transitJdUtc, atmosphere);
  },

  toDevanagariDigits,
  fromDevanagariDigits,
};

