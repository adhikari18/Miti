---
name: "🟦 [TypeScript / Good First Issue] ECMAScript Temporal.CalendarProtocol & Intl.DateTimeFormat Adapter"
about: "Implement a TC39 Temporal-compatible BikramSambatCalendar adapter and richer localized formatting tokens"
title: "feat(miti): add TC39 Temporal calendar adapter and extended Nepali formatting tokens"
labels: ["typescript", "good first issue"]
assignees: ""
---

## Context

`packages/miti/src/index.ts` provides `MitiDate` with `format()`, `addDays()`, `toGregorian()`, and `sankranti()`. To integrate seamlessly with modern JavaScript/TypeScript date ecosystems, we want to add a **TC39 `Temporal` Calendar-compatible adapter** plus relative date helpers (`diffDays`, `startOfMonth`, `endOfMonth`, `isSameMonth`).

## Acceptance Criteria

1. Add `startOfMonth(): MitiDate`, `endOfMonth(): MitiDate`, `addMonths(n: number): MitiDate`, and `diffInDays(other: MitiDate): number` to `MitiDate` in `packages/miti/src/index.ts`.
2. Implement a `BikramSambatTemporalCalendar` class exposing `year(date)`, `month(date)`, `day(date)`, `daysInMonth(date)`, and `daysInYear(date)`.
3. Add unit tests in `packages/miti/test/miti.test.ts` covering month-boundary arithmetic across 29-, 30-, 31-, and 32-day B.S. months.
