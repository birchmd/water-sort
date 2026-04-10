use {
    wasm_bindgen::{JsCast, JsValue, closure::Closure},
    web_sys::{Document, HtmlElement},
};

pub mod puzzle_display;

pub fn create_button<F: FnMut() + 'static>(
    document: &Document,
    text: &str,
    on_click: F,
) -> Result<HtmlElement, JsValue> {
    let button = document.create_element("button")?;
    button.set_text_content(Some(text));
    let closure = Closure::new::<Box<dyn FnMut()>>(Box::new(on_click));
    let elem: HtmlElement = button.unchecked_into();
    elem.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    Ok(elem)
}
