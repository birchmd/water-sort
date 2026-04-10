use {
    rand::{SeedableRng, rngs::StdRng},
    wasm_bindgen::JsValue,
};

pub fn new_rng() -> Result<StdRng, JsValue> {
    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed)
        .map_err(|_| JsValue::from_str("Failed to get random number for rng seed"))?;
    let rng = StdRng::from_seed(seed);
    Ok(rng)
}
