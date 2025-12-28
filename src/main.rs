use std::io::{self, stdout,  Stdout, Write };
use std::thread::sleep;
use std::{ time::{self, Duration} };
use rand::{Rng, rng};

use crossterm::{
     ExecutableCommand, QueueableCommand, cursor::{self,  Hide, Show}, style::{self, Color, Stylize}, terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, window_size
    }
};


#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct Cell {
    ttl : u8,
    life: u8,
    x: u16,
    y: u16,
}


#[allow(dead_code)]
struct Matrix {
    cols: u16,
    rows: u16,
    foreground: Color,
    background: Color,
    speed: Duration,
    tail: u16,
    max_cells: usize,
    cells: Vec<Cell>,
}

impl Matrix {

    fn new(cols: u16, rows: u16, foreground: Color, background: Color, speed: Duration, tail: u16, max_cells: usize, cells: Vec<Cell>) -> Self {

        Self {
            cols,
            rows,
            foreground,
            background,
            speed, 
            tail, 
            max_cells,
            cells
        }
    }

    fn prepare_background(&self,stdout: &mut Stdout) -> io::Result<()> {
        for y in 0..self.rows {
            for x in 0..self.cols {
                    stdout.queue(cursor::MoveTo(x, y))?
                        .queue(style::PrintStyledContent(" ".with(self.background).on(self.background)))?;
            }
            stdout.flush()?;
        }

        Ok(())

    }

    fn print(&mut self, stdout: &mut Stdout) -> io::Result<()> {

        for cell in self.cells.iter() {
            match cell.life.cmp(&0) {
                std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => {

                    stdout.queue(cursor::MoveTo(cell.x, cell.y))?
                        .queue(style::PrintStyledContent("X".with(self.foreground).on(self.background)))?;
                },
                _ => {
                    stdout.queue(cursor::MoveTo(cell.x, cell.y))?
                        .queue(style::Print(" ".with(self.background).on(self.background)))?;
                }
                
            }
        }
        stdout.flush()?;


        Ok(())
    }

    fn print_tail(&mut self, stdout: &mut Stdout) -> io::Result<()> {

        for cell in self.cells.iter() {
            if self.tail <= u16::from(cell.ttl - cell.life) {
                let v = if cell.y == 0 { 0 } else {self.tail};
                stdout.queue(cursor::MoveTo(cell.x, cell.y - v))?
                    .queue(style::Print(" ".with(self.background).on(self.background).attribute(style::Attribute::Dim)))?;
            }
            
        }
        stdout.flush()?;


        Ok(())
    }

    fn tick(&mut self) {

        for cell in self.cells.iter_mut(){
            if cell.y + 1 >= self.rows {
                cell.life = 0
            }

            if cell.life > 0 {
                cell.life -= 1;
                cell.y =(cell.y + 1).rem_euclid(self.rows);
            }
        }
    }

    fn spawn(&mut self) {

        self.cells = self.cells.clone().into_iter().filter(|c| c.life > 0 ).collect::<Vec<Cell>>();

        let mut rng = rng();
        let ttl :u8 = 10;

        let x = rng.random_range(0..=self.cols);
        let y = rng.random_range(0..=self.rows);

        let cell = Cell {
            ttl,
            life: ttl,
            x,
            y
        };

        if self.cells.len() < self.max_cells {
            self.cells.push(cell);
        }
    }
}


fn main() -> io::Result<()> {

    let window = window_size()?;
    let mut stdout = stdout();

    let tail: u16 = 3;
    let speed = time::Duration::from_millis(100);
    let max_cells: usize = 5;

    let mut matrix = Matrix::new(window.columns, window.rows, 
        Color::DarkGreen, Color::Black, 
        speed, tail, max_cells,
        vec![]);

    // stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(Hide)?;

    matrix.prepare_background(&mut stdout)?;

    loop {
        sleep(matrix.speed);
        matrix.spawn();
        matrix.print(&mut stdout)?;
        matrix.print_tail(&mut stdout)?;
        matrix.tick();
    }

    // stdout.execute(Show)?;
    // stdout.execute(LeaveAlternateScreen)?;
    // Ok(())
}
