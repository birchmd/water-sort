use {
    crate::{
        puzzle::{GenericPuzzle, Vial},
        utils::comp_time_arith::{IsSum, SixEqualsFourPlusTwo},
    },
    core::array,
    futures_channel::mpsc,
    rand::{Rng, rngs::StdRng},
    wasm_bindgen::{JsCast, JsValue, prelude::Closure},
    web_sys::{Document, Element, HtmlElement},
};

const EMPTY_COLOUR: &str = "#f8f9fa";
const BLUE: &str = "#0072B2";
const VERMILION: &str = "#D55E00";
const GREEN: &str = "#009E73";
const PINK: &str = "#CC79A7";

pub type PuzzleDisplay = GenericPuzzleDisplay<5, 6, 4, 2, SixEqualsFourPlusTwo, StdRng>;

pub enum Msg {
    Select(usize),
    Reset,
    NewPuzzle,
}

pub struct GenericPuzzleDisplay<
    const N: usize,
    const T: usize,
    const C: usize,
    const K: usize,
    P,
    R,
> where
    P: IsSum<C, K, T>,
{
    puzzle: GenericPuzzle<N, T, C, K, P, R>,
    cells: [[HtmlElement; N]; T],
    vials: [Element; T],
    rx: mpsc::UnboundedReceiver<Msg>,
    active_index: Option<usize>,
    move_count: usize,
    move_display: Element,
    optimal_display: Element,
    solved_text: HtmlElement,
}

impl<const N: usize, const T: usize, const C: usize, const K: usize, P, R>
    GenericPuzzleDisplay<N, T, C, K, P, R>
where
    P: IsSum<C, K, T> + 'static,
    R: Rng + 'static,
{
    pub fn new(
        puzzle: GenericPuzzle<N, T, C, K, P, R>,
        tx: mpsc::UnboundedSender<Msg>,
        rx: mpsc::UnboundedReceiver<Msg>,
        document: &Document,
        body: &HtmlElement,
    ) -> Result<Self, JsValue> {
        let mut cells = array::from_fn(|_| array::from_fn(|_| body.clone()));
        let mut vials = array::from_fn(|_| body.clone().into());

        let container = document.create_element("div")?;
        container.set_class_name("puzzledisplay");
        body.append_child(&container)?;

        let row = document.create_element("div")?;
        row.set_class_name("puzzlerow");
        container.append_child(&row)?;

        let msgs = document.create_element("div")?;
        msgs.set_class_name("puzzlerow");
        container.append_child(&msgs)?;

        let move_display = document.create_element("p")?;
        msgs.append_child(&move_display)?;

        let optimal_display = document.create_element("p")?;
        msgs.append_child(&optimal_display)?;

        let solved_text: HtmlElement = document.create_element("h3")?.unchecked_into();
        msgs.append_child(&solved_text)?;

        for i in 0..T {
            let (vial, contents) = create_vial(puzzle.get(i), document, &row)?;
            cells[i] = contents;
            vials[i] = vial;
        }

        for (i, vial) in vials.iter().enumerate() {
            let local_tx = tx.clone();
            let on_click = move || {
                local_tx.unbounded_send(Msg::Select(i)).ok();
            };
            let closure = Closure::new::<Box<dyn FnMut()>>(Box::new(on_click));
            vial.unchecked_ref::<HtmlElement>()
                .set_onclick(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        let this = Self {
            puzzle,
            cells,
            vials,
            rx,
            active_index: None,
            move_count: 0,
            move_display,
            optimal_display,
            solved_text,
        };

        this.set_solved_text_visible(false);
        this.set_move_display();
        this.set_optimal_display();
        Ok(this)
    }

    pub fn spawn(mut self) {
        wasm_bindgen_futures::spawn_local(async move {
            while let Ok(msg) = self.rx.recv().await {
                match msg {
                    Msg::Select(idx) => match self.active_index {
                        None => self.activate_index(idx),
                        Some(src) => {
                            if self.puzzle.pour(src, idx) {
                                self.move_count += 1;
                                self.set_move_display();
                            }
                            self.set_solved_text_visible(self.puzzle.is_solved());
                            self.deactivate_index();
                        }
                    },
                    Msg::NewPuzzle => {
                        self.puzzle.regenerate();
                        self.deactivate_index();
                        self.move_count = 0;
                        self.set_solved_text_visible(false);
                        self.set_move_display();
                        self.set_optimal_display();
                    }
                    Msg::Reset => {
                        self.puzzle.reset();
                        self.deactivate_index();
                        self.move_count = 0;
                        self.set_solved_text_visible(false);
                        self.set_move_display();
                    }
                }
            }
        })
    }

    fn set_optimal_display(&self) {
        let text = format!("Minimum Moves: {}", self.puzzle.min_moves());
        self.optimal_display.set_text_content(Some(&text));
    }

    fn set_move_display(&self) {
        let text = format!("Move Count: {}", self.move_count);
        self.move_display.set_text_content(Some(&text));
    }

    fn set_solved_text_visible(&self, is_visible: bool) {
        if is_visible {
            if self.puzzle.min_moves() < self.move_count {
                self.solved_text
                    .set_text_content(Some("You solved it, but there is a more efficient way!"));
            } else {
                self.solved_text.set_text_content(Some("Solved! Nice job!"));
            }
            self.solved_text
                .style()
                .set_property("display", "block")
                .ok();
        } else {
            self.solved_text
                .style()
                .set_property("display", "none")
                .ok();
        }
    }

    fn activate_index(&mut self, idx: usize) {
        self.active_index = Some(idx);
        for (j, vial) in self.vials.iter().enumerate() {
            if j == idx {
                vial.set_class_name("activevial");
            } else {
                vial.set_class_name("vial");
            }
        }
    }

    fn deactivate_index(&mut self) {
        self.active_index = None;
        for i in 0..T {
            self.vials[i].set_class_name("vial");
            for j in 0..N {
                let colour = colour_map(self.puzzle.get(i).get(j));
                self.cells[i][j]
                    .style()
                    .set_property("background-color", colour)
                    .ok();
            }
        }
    }
}

fn create_vial<const N: usize>(
    source: &Vial<N>,
    document: &Document,
    container: &Element,
) -> Result<(Element, [HtmlElement; N]), JsValue> {
    let inner_container = document.create_element("div")?;
    inner_container.set_class_name("vial");
    let table = document.create_element("table")?;
    let mut output = array::from_fn(|_| table.unchecked_ref::<HtmlElement>().clone());
    for (i, out) in output.iter_mut().enumerate() {
        let row = document.create_element("tr")?;
        let cell: HtmlElement = document.create_element("td")?.unchecked_into();
        let colour = colour_map(source.get(i));
        cell.style().set_property("background-color", colour)?;
        row.append_child(&cell)?;
        table.append_child(&row)?;
        *out = cell;
    }
    inner_container.append_child(&table)?;
    container.append_child(&inner_container)?;
    Ok((inner_container, output))
}

const fn colour_map(c: u8) -> &'static str {
    match c {
        1 => BLUE,
        2 => VERMILION,
        3 => GREEN,
        4 => PINK,
        _ => EMPTY_COLOUR,
    }
}
