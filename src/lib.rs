mod utils;

use cfg_if::cfg_if;
use rand::prelude::*;
use std::fmt;
use web_sys::{window, HtmlElement};
use wasm_bindgen::prelude::*;

const SIZE_FACTOR: u32 = 10;
const FAVOR: u32 = 20;

// A macro to provide `println!(..)`-style syntax for `console.log` logging
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

pub fn now() -> f64 {
    window()
        .expect("should have a window")
        .performance()
        .expect("should have a performance")
        .now()
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    pub fn rand_cells(height: u32, width: u32) -> Vec<Cell> {
        let low = 1;
        let high = 100;
        let mut rng = rand::rng();

        (0..height * width)
            .map(|_| {
                let val = rng.random_range(low..high) as u32;
                if val + FAVOR < (high / 2) {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect()
    }

    pub fn reset_rand(&mut self) {
        self.cells = Universe::rand_cells(self.height, self.width);
    }

    pub fn reset_dead(&mut self) {
        self.cells = (0..self.width * self.height)
            .map(|_| Cell::Dead)
            .collect();
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();

        let window = window().expect("should have a Window");
        let document = window.document().expect("should have a Document");
        let body = document.body().expect("document should have a body");
        let body_element = body.dyn_into::<HtmlElement>().unwrap();

        log!("[{}] beginning render of canvas", now());
        let height = body_element.client_height() as u32 / SIZE_FACTOR;
        let width = body_element.client_width() as u32 / SIZE_FACTOR;
        let cells = Universe::rand_cells(height, width);
        log!("[{}] rendered canvas of size {}x{}", now(), width, height);

        Universe { width, height, cells }
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height)
            .map(|_| Cell::Dead)
            .collect();
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..height * self.width)
            .map(|_| Cell::Dead)
            .collect();
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // rule 1
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // rule 2
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // rule 3
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // rule 4
                    (Cell::Dead, 3) => Cell::Alive,
                    // default
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }
}

impl Universe {
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().clone() {
            let idx = self.get_index(*row, *col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
