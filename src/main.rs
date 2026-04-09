use wasm_bindgen::JsValue;

mod puzzle;
mod ui;
mod utils;

fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let rng = utils::random::new_rng()?;
    let (tx, rx) = futures_channel::mpsc::unbounded();
    let puzzle = self::puzzle::Puzzle::new(rng);
    let display = ui::puzzle_display::PuzzleDisplay::new(puzzle, tx.clone(), rx, &document, &body)?;
    display.spawn();

    let button_container = document.create_element("div")?;
    button_container.set_class_name("puzzlerow");
    body.append_child(&button_container)?;

    let local_tx = tx.clone();
    let reset_button = ui::create_button(&document, "Reset", move || {
        local_tx.unbounded_send(ui::puzzle_display::Msg::Reset).ok();
    })?;
    button_container.append_child(&reset_button)?;

    let new_puzzle_button = ui::create_button(&document, "New Puzzle", move || {
        tx.unbounded_send(ui::puzzle_display::Msg::NewPuzzle).ok();
    })?;
    button_container.append_child(&new_puzzle_button)?;

    Ok(())
}
