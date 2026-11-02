# Contributing to `Miti` (मिति)

Thank you for contributing to **`Miti`** — the open-source, pure-math Bikram Sambat (B.S.) astronomical date engine and sidereal calendar ecosystem!

We welcome contributions across **computational astronomy**, **Rust (`no_std` / WebAssembly)**, **TypeScript SDK ergonomics**, and **Nepali *Panchanga* calendar research**.

---

## 1. Local Development Quickstart

### Prerequisites
- **Node.js** `>= 20.0.0` & **npm** `>= 10.0.0` (for `packages/miti` and `packages/miti-cli`)
- **Rust / Cargo** `>= 1.75` (optional locally, required when modifying `crates/miti-core` or `crates/miti-wasm`)

### TypeScript SDK & CLI Workflow

```bash
# 1. Install workspace dependencies
npm install

# 2. Build the TypeScript library (`packages/miti`) and CLI (`packages/miti-cli`)
npm run build

# 3. Run the Vitest astronomical & calendar unit test suite
npm test

# 4. Run strict TypeScript typechecking across workspaces
npm run typecheck

# 5. Try the terminal CLI
npm run cli -- conv 2026-11-02
npm run cli -- year 2083
npm run cli -- benchmark
```

### Rust Core & WASM Workflow

```bash
# Run unit tests in crates/miti-core
cargo test --package miti-core
```

---

## 2. Seeded Issues to Get Started

Check out our ready-to-tackle issues in [`.github/ISSUE_TEMPLATE/`](./.github/ISSUE_TEMPLATE/):
1. **`math` / `astronomy`:** [High-Altitude Atmospheric Refraction & Dip Correction for Kathmandu Sunrise (`1,400m`)](./.github/ISSUE_TEMPLATE/01-math-refraction-topocentric-sunrise.md)
2. **`rust` / `math`:** [Bit-Packed Historical Panchanga Delta-Override Layer (`no_std`)](./.github/ISSUE_TEMPLATE/02-rust-compact-panchanga-delta-patch.md)
3. **`astronomy` / `rust`:** [ELP-2000 Lunar Ephemeris, Tithi (`12°` Elongation) & Adhika Masa Solver](./.github/ISSUE_TEMPLATE/03-astronomy-elp2000-lunar-tithi-engine.md)
4. **`typescript` / `good first issue`:** [ECMAScript `Temporal.CalendarProtocol` & `Intl` Adapter](./.github/ISSUE_TEMPLATE/04-typescript-temporal-intl-calendar-adapter.md)

See [`ROADMAP.md`](./ROADMAP.md) for the complete 115+ hour multi-phase engineering plan.

---

## License

By contributing to `Miti`, you agree that your contributions will be licensed under the [MIT License](./LICENSE).

