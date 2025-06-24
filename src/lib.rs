mod utils;

use cfg_if::cfg_if;
use rand::prelude::*;
use std::fmt;
use web_sys::{console, window};
use wasm_bindgen::prelude::*;

// A macro to provide `println!(..)`-style syntax for `console.log` logging
macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!( $( $t )* ).into());
    }
}

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
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

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Alive => write!(f, "Alive"),
            Cell::Dead => write!(f, "Dead"),
        }
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    active: Vec<Cell>,
    back: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32, cell_size: u32) -> Universe {
        utils::set_panic_hook();
        let scaled_width = width / (cell_size + 1);
        let scaled_height = height / (cell_size + 1);

        log!("original width: {}, original height: {}", width, height);
        log!("[{}] beginning render of canvas: width: {}, height: {}, cell_size: {}",
            now(),
            scaled_width,
            scaled_height,
            cell_size
        );
        let active = Universe::set_random(scaled_width, scaled_height);
        let back = Universe::cells_zeroed(scaled_width, scaled_height);
        log!("[{}] rendered canvas of size {}x{}", now(), scaled_width, scaled_height);

        Universe { width: scaled_width, height: scaled_height, active, back }
    }

    pub fn reset_rand(&mut self) {
        self.active = Universe::set_random(self.height, self.width);
    }

    pub fn reset_dead(&mut self) {
        self.reset_cells();
    }
    
    pub fn make_glider(&mut self, row: i32, col: i32) {
        let offset = 1;
        let glider: [(i32, i32); 5] = [(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)];
        let mut first: i32 = 0;
        let mut second: i32 = 0;

        let cells = glider
            .map(|t| {
                first = t.0 + row - offset;
                second = t.1 + col - offset;

                if first < 0 { first += self.height() as i32; }
                if second < 0 { second += self.width() as i32; }
                if first >= self.height() as i32 { first -= self.height() as i32; }
                if second >= self.width() as i32 { second -= self.width() as i32; }

                (first as u32, second as u32)
            });

        self.set_cells(&cells);
    }

    pub fn make_pulsar(&mut self, row: i32, col: i32) {
        let offset = 6;
        let mut first: i32 = 0;
        let mut second: i32 = 0;
        let pulsar = [
            (0, 2), (0, 3), (0, 4), (0, 8), (0, 9), (0, 10),
            (2, 0), (2, 5), (2, 7), (2, 12),
            (3, 0), (3, 5), (3, 7), (3, 12),
            (4, 0), (4, 5), (4, 7), (4, 12),
            (5, 2), (5, 3), (5, 4), (5, 8), (5, 9), (5, 10),
            (7, 2), (7, 3), (7, 4), (7, 8), (7, 9), (7, 10),
            (8, 0), (8, 5), (8, 7), (8, 12),
            (9, 0), (9, 5), (9, 7), (9, 12),
            (10, 0), (10, 5), (10, 7), (10, 12),
            (12, 2), (12, 3), (12, 4), (12, 8), (12, 9), (12, 10),
        ];
        let cells = pulsar
            .map(|t| {
                first = t.0 + row - offset;
                second = t.1 + col - offset;

                if first < 0 { first += self.height() as i32; }
                if second < 0 { second += self.width() as i32; }
                if first >= self.height() as i32 { first -= self.height() as i32; }
                if second >= self.width() as i32 { second -= self.width() as i32; }

                (first as u32, second as u32)
        });
        self.set_cells(&cells);
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.reset_cells();
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.reset_cells();
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.active.as_ptr()
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 {
            self.height - 1
        } else {
            row - 1
        };

        let south = if row == self.height - 1 {
            0
        } else {
            row + 1
        };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.active[nw] as u8;

        let n = self.get_index(north, column);
        count += self.active[n] as u8;

        let ne = self.get_index(north, east);
        count += self.active[ne] as u8;

        let w = self.get_index(row, west);
        count += self.active[w] as u8;

        let e = self.get_index(row, east);
        count += self.active[e] as u8;

        let sw = self.get_index(south, west);
        count += self.active[sw] as u8;

        let s = self.get_index(south, column);
        count += self.active[s] as u8;

        let se = self.get_index(south, east);
        count += self.active[se] as u8;

        count
    }

    pub fn tick(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.active[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // rule 1
                    (Cell::Alive, x) if x < 2 => {
                        Cell::Dead
                    },
                    // rule 2
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // rule 3
                    (Cell::Alive, x) if x > 3 => {
                        Cell::Dead
                    },
                    // rule 4
                    (Cell::Dead, 3) => {
                        Cell::Alive
                    },
                    // default
                    (otherwise, _) => otherwise,
                };

                self.back[idx] = next_cell;
            }
        }

        std::mem::swap(&mut self.active, &mut self.back);
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.active[idx].toggle();
    }
}

impl Universe {
    pub fn get_cells(&self) -> &[Cell] {
        &self.active
    }

    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().clone() {
            let idx = self.get_index(*row, *col);
            self.active[idx] = Cell::Alive;
        }
    }

    pub fn reset_cells(&mut self) {
        self.active = (0..self.width * self.height)
            .map(|_| Cell::Dead)
            .collect();
    }

    pub fn set_random(width: u32, height: u32) -> Vec<Cell> {
        let low = 1;
        let high = 100;
        let mut rng = rand::rng();

        (0..width * height)
            .map(|_| {
                let val = rng.random_range(low..high) as u32;
                if val > (high / 2) {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect()
    }

    pub fn cells_zeroed(width: u32, height: u32) -> Vec<Cell> {
        (0..width * height).map(|_| Cell::Dead).collect()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.active.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
