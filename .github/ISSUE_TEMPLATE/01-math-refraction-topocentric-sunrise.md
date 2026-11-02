---
name: "📐 [Math / Astronomy] High-Altitude Atmospheric Refraction & Dip Correction for Kathmandu Sunrise (1,400m)"
about: "Refine topocentric sunrise zenith angle in sunrise.rs with barometric pressure, temperature, and 1,400m elevation dip"
title: "feat(sunrise): implement elevation horizon dip and Saemundsson-Bennett atmospheric refraction for Kathmandu (1,400m)"
labels: ["math", "astronomy"]
assignees: ""
---

## Context

`crates/miti-core/src/sunrise.rs` and `packages/miti/src/astronomy.ts` currently use the standard apparent sunrise zenith angle $Z = 90.8333^\circ$ ($90^\circ 50'$). However, the Kathmandu Valley sits at an elevation of $h \approx 1,400\text{ m}$ above mean sea level with lower barometric pressure ($P \approx 856\text{ hPa}$), which alters both the geometric horizon dip ($\Delta d \approx 0.0293^\circ \sqrt{h}$) and standard atmospheric refraction ($R_0 \approx 34'$).

Near critical edge-case *Sankranti* transits (within $\pm 4$ minutes of the civil transition cutoff), this topocentric correction determines whether *Gate 1* (`१ गते`) falls on day $D$ or $D+1$.

## Acceptance Criteria

1. Implement `topocentric_sunrise_zenith_deg(elevation_m: f64, pressure_hpa: f64, temp_celsius: f64) -> f64` in `crates/miti-core/src/sunrise.rs` and `packages/miti/src/astronomy.ts`.
2. Incorporate solar semi-diameter variation ($s = s_0 / R_{\text{AU}}$) based on Earth-Sun radius vector $R$.
3. Add unit tests verifying the sunrise shift (in seconds) at $h = 1,400\text{ m}$ across summer and winter solstices.

