# `Miti` (मिति) Post-MVP Engineering Roadmap (115+ Hours Planned)

This document outlines the post-MVP multi-phase engineering and computational astronomy roadmap for **`Miti` (मिति)** beyond the `v0.1.0` core release. Each phase is decomposed into concrete, testable engineering work items (3–6 hours each) across celestial mechanics, Rust/WASM systems programming, lunisolar *Panchanga* algorithms, and TypeScript developer tooling.

---

## Phase 1: Historical Calibration, Topocentric Refraction & Anomaly Patch Layer (~20 hrs)

- [ ] **1.1 High-Altitude Atmospheric Refraction & Horizon Dip Solver (`1,400m` Kathmandu) (~5 hrs):**
  - Enhance `crates/miti-core/src/sunrise.rs` and `packages/miti/src/astronomy.ts` with elevation-dependent geometric horizon depression ($\Delta d = 0.0293^\circ \sqrt{h}$), barometric pressure ($856\text{ hPa}$), and solar semi-diameter ($s_0 / R_{\text{AU}}$) corrections for borderline *Sankranti* transits occurring within $\pm 4$ minutes of the civil cutoff.
- [ ] **1.2 Classical *Surya Siddhanta* vs. Modern *Drik* Ephemeris Mode Switch (~5 hrs):**
  - Implement the traditional *Surya Siddhanta* mean-motion and epicyclic (*Manda Paridhi*) solar equation alongside the modern VSOP87 *Drik* solver so historians can reproduce pre-2000 B.S. *Panchanga* rulings that relied on classical Siddhantic tables.
- [ ] **1.3 Bit-Packed Historical Delta-Override Patch Layer (`no_std`, `<256 bytes`) (~5 hrs):**
  - Build a zero-allocation sparse delta table storing only `(bs_year, bs_month, ±1_day_shift)` for rare historical months where the *Nepal Panchanga Nirnayak Samiti* issued administrative overrides differing from pure astronomical transit.
- [ ] **1.4 Automated 100-Year Verification Harness (`B.S. 1990–2090`) (~5 hrs):**
  - Build a benchmark suite in `crates/miti-core` and `packages/miti` that cross-validates algorithmically generated month lengths against archival gazette records and outputs a per-decade precision report.

---

## Phase 2: Lunar Dynamics, *Tithi* & *Panchanga* Five-Limb Engine (~30 hrs)

While Bikram Sambat civil dates (*Gate* / गते) are purely solar, Nepali festivals and rituals depend on the five limbs (*Panchanga*: *Tithi*, *Vara*, *Nakshatra*, *Yoga*, *Karana*) of the lunisolar calendar:

- [ ] **2.1 Truncated ELP-2000/82 Lunar Ecliptic Longitude Solver (`lunar.rs`) (~6 hrs):**
  - Implement Meeus Ch. 47 / ELP-2000 periodic series (60 primary solar-lunar perturbation terms including Evection, Variation, and Annual Inequality) to compute geocentric lunar longitude $\lambda_{\text{Moon}}(\text{JD})$ to $< 10''$ precision.
- [ ] **2.2 Continuous *Tithi* ($12^\circ$ Lunar-Solar Elongation) & *Paksha* Solver (~6 hrs):**
  - Compute the exact start and end timestamps of all 30 lunar *Tithis* ($\Delta\lambda = (\lambda_{\text{Moon}} - \lambda_{\text{Sun}}) \bmod 360^\circ$, $12^\circ$ per *Tithi*) across *Shukla Paksha* (waxing) and *Krishna Paksha* (waning), including *Kshaya Tithi* (skipped) and *Vriddhi Tithi* (repeated) detection at Kathmandu sunrise.
- [ ] **2.3 27 *Nakshatra* (Lunar Mansions), 27 *Yoga* & 60 *Karana* Solvers (~6 hrs):**
  - Compute the Moon's sidereal *Nakshatra* ($\lfloor \lambda_{\text{Moon, sid}} / 13^\circ 20' \rfloor$), luni-solar *Yoga* ($\lfloor (\lambda_{\text{Moon, sid}} + \lambda_{\text{Sun, sid}}) / 13^\circ 20' \rfloor$), and half-tithi *Karana* at any UTC/NST instant.
- [ ] **2.4 *Adhika Masa* (*Malamasa* / Lunar Leap Month) & *Kshaya Masa* Detector (~6 hrs):**
  - Implement deterministic detection of lunar intercalary leap months (*Adhika Masa*: a lunar *Amanta* cycle containing zero solar *Sankranti* transits) and decayed months (*Kshaya Masa*: a lunar cycle containing two *Sankrantis*).
- [ ] **2.5 Full Daily *Panchanga* Struct & JSON/CLI Formatter (`miti panchanga`) (~6 hrs):**
  - Expose `Miti.getPanchanga(date)` and `miti panchanga <date>` outputting solar *Gate*, *Rashi*, *Tithi* at sunrise, *Nakshatra*, *Yoga*, *Karana*, and Kathmandu sunrise/sunset times.

---

## Phase 3: Deterministic Festival & Public Holiday Predictor (~25 hrs)

- [ ] **3.1 Solar *Sankranti* Festival Resolver (*Maghe Sankranti*, *Bisket / New Year*, *Saune Sankranti*, *Asar 15*) (~5 hrs):**
  - Derive all solar-fixed Nepali observances directly from `findSankrantiTransit()` without static calendar tables.
- [ ] **3.2 *Dashain* (*Navaratri*, *Phulpati*, *Maha Ashtami*, *Maha Navami*, *Vijaya Dashami*) Solver (~6 hrs):**
  - Compute the exact civil Bikram Sambat dates of *Ghatasthapana* (*Ashwin Shukla Pratipada*) through *Vijaya Dashami* (*Ashwin Shukla Dashami*) using sunrise *Tithi* dominance rules.
- [ ] **3.3 *Tihar* (*Yamapanchak*: *Kaag*, *Kukur*, *Gai/Laxmi Puja*, *Mha Puja*, *Bhai Tika*) Solver (~5 hrs):**
  - Resolve *Kartik Krishna Trayodashi* through *Kartik Shukla Dwitiya*, including evening *Pradosh* vs. *Aparahna* *Tithi* rules for *Laxmi Puja* and *Bhai Tika*.
- [ ] **3.4 Major Lunar Festival Suite (*Maha Shivaratri*, *Buddha Jayanti*, *Fagu Purnima / Holi*, *Janai Purnima*, *Teej*) (~5 hrs):**
  - Implement algorithmic resolvers for *Falgun Krishna Chaturdashi* (*Shivaratri*), *Baisakh Shukla Purnima* (*Buddha Jayanti*), *Falgun Shukla Purnima* (*Holi*), and *Bhadra Shukla Tritiya* (*Haritalika Teej*).
- [ ] **3.5 Public Holiday Query API (`Miti.getFestivalsForYear(bsYear)`) (~4 hrs):**
  - Provide a clean TypeScript and Rust query API returning all solar and lunisolar festivals for any past or future Bikram Sambat year.

---

## Phase 4: Developer Ecosystem, WASM Packaging & Cross-Platform Ergonomics (~20 hrs)

- [ ] **4.1 TC39 `Temporal.CalendarProtocol` & `Intl` Adapter (`packages/miti`) (~5 hrs):**
  - Implement a standards-compliant `BikramSambatTemporalCalendar` class (`year()`, `month()`, `day()`, `dateAdd()`, `dateUntil()`, `daysInMonth()`) for seamless integration with the JavaScript `Temporal` API.
- [ ] **4.2 Date Arithmetic, Range Iterators & Relative Formatting (`diff`, `addMonths`, `addYears`, `fromNow`) (~5 hrs):**
  - Expand `MitiDate` with calendar-aware month/year arithmetic, billing-cycle helpers (`startOfMonth`, `endOfMonth`, `isLeapYear`), and localized Nepali relative time strings (`"३ दिन अघि"`, `"भोलि"`).
- [ ] **4.3 Automated `wasm-pack` Dual ESM/CJS Bundle & Benchmark Suite (~5 hrs):**
  - Configure `crates/miti-wasm` build scripts to emit tree-shakeable WebAssembly + inline base64 WASM loaders alongside Vitest microbenchmarks comparing native JS math vs. WASM batch throughput.
- [ ] **4.4 Interactive Web Calendar & Ephemeris Explorer (`packages/miti-web`) (~5 hrs):**
  - Build a lightweight interactive web visualizer displaying the live solar ecliptic wheel (*12 Rashis*), *Sankranti* countdown clock, and printable monthly Nepali *Patro* calendar grid.

---

## Phase 5: Extended Astronomical Precision & Multi-Location Topocentric Support (~20 hrs)

- [ ] **5.1 Expanded 150-Term VSOP87D Earth-Sun Series (~5 hrs):**
  - Add an optional high-precision VSOP87D coefficient module reducing solar longitude residuals below `< 0.5"` across a 4,000-year span (`1000 BCE – 3000 CE`).
- [ ] **5.2 Configurable Topocentric Observer Coordinates (Global Diaspora & Regional Nepal Cities) (~5 hrs):**
  - Allow passing custom `(latitude, longitude, elevation, timezone)` into `MitiConfig` (e.g., Pokhara, Biratnagar, Ilam, or diaspora hubs in London, New York, Sydney) while keeping Kathmandu (`27.7172° N, 85.3240° E`) as the civil default.
- [ ] **5.3 Solar & Lunar Eclipse (*Grahana*) Visibility Predictor for Nepal (~6 hrs):**
  - Use lunar node (*Rahu/Ketu*) proximity at *Purnima* and *Amavasya* to detect solar and lunar eclipses visible from the Himalaya.
- [ ] **5.4 Property-Based Fuzzing (`proptest` / `fast-check`) Across 500 Years (~4 hrs):**
  - Add property-based test suites verifying monotonic date ordering, bijection invariants (`toGregorian(fromGregorian(d)) == d`), and zero panics across `1700 CE – 2300 CE`.
