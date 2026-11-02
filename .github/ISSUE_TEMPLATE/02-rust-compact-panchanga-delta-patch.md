---
name: "🦀 [Rust / Core] Bit-Packed Historical Panchanga Delta-Override Layer (no_std)"
about: "Design a compact 16-bit delta patch table for rare historical years where civil Panchanga rulings diverged from pure Drik transit"
title: "feat(core): add bit-packed historical anomaly delta patch layer in miti-core"
labels: ["rust", "math"]
assignees: ""
---

## Context

Pure astronomical ephemeris computation (`VSOP87` + `Chitra Paksha Ayanamsa`) achieves ~99% agreement with published Nepali Bikram Sambat calendars across B.S. 2000–2090 without lookup tables. For the remaining ~1% of borderline historical months where the *Nepal Panchanga Nirnayak Samiti* used traditional *Surya Siddhanta* mean-motion tables instead of modern *Drik* ephemerides (causing a $\pm 1$-day shift on a boundary Sankranti), we want an optional, ultra-compact **Delta Anomaly Patch Layer** (<256 bytes total, `no_std` compatible) rather than a bloated multi-kilobyte lookup table.

## Tasks

1. Define a compact `const ANOMALY_DELTAS: &[(i16, u8, i8)]` sparse array in `crates/miti-core/src/sankranti.rs` storing only `(bs_year, bs_month, day_shift)` for verified historical overrides.
2. Expose `ConversionMode::PureAstronomical` vs. `ConversionMode::CivilHybrid` in both Rust (`miti-core`) and TypeScript (`miti`).
3. Add regression tests comparing both modes across historical boundary dates.
