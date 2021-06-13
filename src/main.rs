use grid::Grid;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::convert::TryInto;
use std::time::Duration;

/*
 * Note: I started with
 *
 * struct Cell {
 *  x: i32,
 *  y: i32,
 *  alive: bool
 * }
 *
 * but then realized the grid indices are the cell
 * position, so Grid<bool> could work.
 *
 */

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const CELL_SIZE: u32 = 10;

struct State {
    width: u32,
    height: u32,
    cell_size: u32,
    _grid_size: u32,
    cell_count: u32,
    cells: Grid<bool>,
}

impl State {
    pub fn new(width: u32, height: u32, cell_size: u32) -> Self {

        Self {
            width: width,
            height: height,
            cell_size: cell_size,
            cell_count: 0,
            _grid_size: (height / cell_size ) * (width / cell_size),
            cells: Grid::new((height / cell_size) as usize,
                             (width / cell_size) as usize),
        }
    }

    pub fn init_glider(mut self) ->  Self {
        let center = 10;
        *self.cells.get_mut(center + 0, center + 1).unwrap() = true;
        *self.cells.get_mut(center + 1, center + 2).unwrap() = true;
        *self.cells.get_mut(center + 2, center + 0).unwrap() = true;
        *self.cells.get_mut(center + 2, center + 1).unwrap() = true;
        *self.cells.get_mut(center + 2, center + 2).unwrap() = true;

        self
    }

    pub fn get_cell_rects(&self) -> Box<[Rect]> {
        let mut rects: Vec<Rect> = Vec::with_capacity(self.cell_count as usize);

        for i in 0..self.cells.rows() {
            for j in 0..self.cells.cols() {
                if self.cell_is_alive(i, j) {
                    rects.push(self.cell_to_rect(j, i));
                }
            }
        }
        rects.into_boxed_slice()
    }

    /*
     * Table for new state from old state
     * C   N                 new C
     * 1   0,1             ->  0  # Lonely
     * 1   4,5,6,7,8       ->  0  # Overcrowded
     * 1   2,3             ->  1  # Lives
     * 0   3               ->  1  # It takes three to give birth!
     * 0   0,1,2,4,5,6,7,8 ->  0  # Barren
     */
    pub fn next(&mut self) {
        let mut new_cells = Grid::new((self.height / self.cell_size) as usize,
                                      (self.width / self.cell_size) as usize);

        for i in 0..self.cells.rows() {
            for j in 0..self.cells.cols() {
                let hood = self.get_hood_count(i, j);
                let cell = self.cells.get(i, j).unwrap();

                if *cell {
                    /* Cell is alive, keep alive on 2 and 3 */
                    if hood == 2 || hood == 3 {
                        *new_cells.get_mut(i, j).unwrap() = true;
                        self.cell_count = self.cell_count + 1;
                    }
                } else {

                    /* Cell is dead, only goes alive with three */
                    if hood == 3 {
                        *new_cells.get_mut(i, j).unwrap() = true;
                        self.cell_count = self.cell_count + 1;
                    }
                }
            }
        }
        self.cells = new_cells;
    }

    fn cell_to_rect(&self, x: usize, y: usize) -> Rect {
        Rect::new(((self.cell_size as usize * x)).try_into().unwrap(),
                  ((self.cell_size as usize * y)).try_into().unwrap(),
                  self.cell_size, self.cell_size)
    }

    fn get_hood_count(&self, i: usize, j: usize) -> u32 {
        let mut hood = 0;

        /*
         * Sum the number of live cells
         * around cell at row=i, col=j:
         *
         *   [0,0|0,1|0,2]
         *   [1,0|X,X|1,2]
         *   [2,0|2,1|2,2]
         */
        for off_i in 0..3 {
            for off_j in 0..3 {

                /* (1,1) is the current cell, so skip it */
                if off_i == 1 && off_j == 1 {
                    continue;
                }

                if let Some(ii) = (i + off_i).checked_sub(1) {
                    if let Some(jj) = (j + off_j).checked_sub(1) {
                        if let Some(value) = self.cells.get(ii, jj) {
                            hood += *value as u32;
                        }
                    }
                }
            }
        }
        hood
    }

    fn cell_is_alive(&self, i: usize, j: usize) -> bool {
        *self.cells.get(i, j).unwrap()
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("game of life", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut state = State::new(WIDTH, HEIGHT, CELL_SIZE).init_glider();

    /* Could be changed with some timer event loop */
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.fill_rects(&*(state.get_cell_rects())).unwrap();

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        state.next();
    }
}
