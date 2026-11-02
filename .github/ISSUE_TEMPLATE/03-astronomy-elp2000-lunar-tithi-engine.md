---
name: "🌙 [Astronomy / Rust] ELP-2000 Lunar Ephemeris, Tithi (12° Elongation) & Adhika Masa Solver"
about: "Implement lunar ecliptic longitude to compute the 30 Tithis (Pratipada to Purnima/Amavasya) and lunar leap months"
title: "feat(lunar): implement ELP-2000/82 truncated lunar longitude and 12-degree Tithi solver"
labels: ["astronomy", "rust"]
assignees: ""
---

## Context

While Bikram Sambat civil dates (*Gate* / गते) are solar, almost all Nepali cultural and public holidays (*Dashain*, *Tihar*, *Buddha Jayanti*, *Maha Shivaratri*, *Janai Purnima*, *Teej*) are governed by the **Lunisolar Panchanga**, specifically the **Tithi** (lunar day), defined by each $12^\circ$ increment in geocentric ecliptic elongation between the Moon and the Sun:

$$\text{Tithi}(\text{JD}) = \left\lfloor \frac{(\lambda_{\text{Moon}}(\text{JD}) - \lambda_{\text{Sun}}(\text{JD})) \bmod 360^\circ}{12^\circ} \right\rfloor + 1 \in \{1 \dots 30\}$$

## Acceptance Criteria

1. Create `crates/miti-core/src/lunar.rs` implementing Meeus Ch. 47 / truncated ELP-2000 periodic terms for apparent lunar ecliptic longitude $\lambda_{\text{Moon}}$ (accurate to $< 10''$).
2. Implement `tithi_at_sunrise(jd_utc: f64) -> TithiInfo` (returning *Paksha*: `Shukla` vs. `Krishna`, Tithi index `1..=30`, and exact Tithi end timestamp).
3. Detect **Adhika Masa** (*Malamasa* / lunar leap month when a lunar synodic month contains zero solar *Sankranti* transits) and **Kshaya Masa** (when a lunar month contains two *Sankrantis*).
