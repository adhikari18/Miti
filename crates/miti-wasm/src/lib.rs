//! WebAssembly bindings exposing `miti-core` to JavaScript and TypeScript runtimes.

use miti_core::{
    days_in_bs_month, find_sankranti_transit_jd, gregorian_to_miti, miti_to_gregorian,
    GregorianDate,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmMitiResult {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub weekday: u8,
    pub days_in_month: u8,
}

#[wasm_bindgen]
pub fn from_gregorian_wasm(year: i32, month: u8, day: u8) -> WasmMitiResult {
    let m = gregorian_to_miti(GregorianDate::new(year, month, day));
    WasmMitiResult {
        year: m.year,
        month: m.month,
        day: m.day,
        weekday: m.weekday,
        days_in_month: m.days_in_month,
    }
}

#[wasm_bindgen]
pub fn to_gregorian_wasm(bs_year: i32, bs_month: u8, bs_day: u8) -> Result<Vec<i32>, JsValue> {
    let g = miti_to_gregorian(bs_year, bs_month, bs_day).map_err(|e| JsValue::from_str(e))?;
    Ok(vec![g.year, g.month as i32, g.day as i32])
}

#[wasm_bindgen]
pub fn sankranti_transit_jd_wasm(bs_year: i32, bs_month: u8) -> f64 {
    find_sankranti_transit_jd(bs_year, bs_month).transit_jd_utc
}

#[wasm_bindgen]
pub fn month_lengths_for_bs_year_wasm(bs_year: i32) -> Vec<u8> {
    (1..=12).map(|m| days_in_bs_month(bs_year, m)).collect()
}
