use std::io::{self, stdout,  Stdout, Write };
use std::{ time::{self, Duration}, cmp::min };
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use rand::{Rng, rng};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor::{self,  Hide, Show}, style::{self, Color, Stylize}, terminal::{
        Clear, ClearType, EnterAlternateScreen,  window_size, LeaveAlternateScreen
    },
    event::{
        poll, read, Event, KeyCode
    }

};


#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct Cell {
    distance: u8,
    alive: bool,
    x: u16,
    y: u16,
}


#[allow(dead_code)]
struct Tail {
    length :u8,
    gradiant_colors: Vec<Color>,
}


#[allow(dead_code)]
struct Matrix<'a> {
    cols: u16,
    rows: u16,
    background: Color,
    speed: Duration,
    max_cells: usize,
    cells: Vec<Cell>,
    tail: Tail,
    spawn_prob: u8,
    charset: &'a[u8; 52],
    pause: bool
}

impl Matrix<'_> {

    fn new(cols: u16, rows: u16, background: Color, speed: Duration, max_cells: usize, cells: Vec<Cell>, tail: Tail, spawn_prob: u8) -> Self {

        Self {
            cols,
            rows,
            background,
            speed, 
            tail, 
            max_cells,
            cells,
            spawn_prob,
            charset: b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
            pause: false
        }
    }

    fn set_gradiant_vector(&mut self,gradiant_colors: Vec<Color>) {
        self.tail.gradiant_colors = gradiant_colors;
    }

    fn clear_background(&self,stdout: &mut Stdout) -> io::Result<()> {
        for y in 0..self.rows {
            for x in 0..self.cols {
                stdout.queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent(self.generate_random_str().with(self.tail.gradiant_colors[0 as usize]).on(self.background)))?;
                }
            stdout.flush()?;
        }

        Ok(())

    }

    fn generate_random_str(&self) -> char {
        let idx = rng().random_range(0..self.charset.len());
        self.charset[idx] as char
    }

    fn print(&mut self, stdout: &mut Stdout) -> io::Result<()> {

        for cell in self.cells.iter() {
            match cell.alive {
                true => {
                    // Print the trail of falling cells.
                    let trail = min(cell.distance, self.tail.length);
                    for cy in (0..trail).rev() {
                        let grad_idx = (trail - 1 - cy) as usize;
                        stdout.queue(cursor::MoveTo(cell.x, cell.y - (cy as u16)))?
                            .queue(style::PrintStyledContent(format!("{}", self.generate_random_str()).with(self.tail.gradiant_colors[grad_idx]).on(self.background)))?;
                        }

                    // clean trail not part of the tail anymore.
                    if self.tail.length < cell.distance {
                        let l: u16 = (self.tail.length as u16) + 1;
                        stdout.queue(cursor::MoveTo(cell.x, cell.y - l))?
                            .queue(style::PrintStyledContent(self.generate_random_str().with(self.tail.gradiant_colors[0 as usize]).on(self.background)))?;
                    }
                },
                false => {
                    // clean the trail of the dead cell
                    let disappearing_trail = cell.y - (self.tail.length as u16) + 1;
                    stdout.queue(cursor::MoveTo(cell.x, disappearing_trail))?
                        .queue(style::PrintStyledContent(self.generate_random_str().with(self.tail.gradiant_colors[0 as usize]).on(self.background)))?;
                    }

            }
        }
        stdout.flush()?;

        self.cells = self.cells.clone().into_iter()
            .filter(|c| c.alive == true || (!c.alive && (c.y - (self.tail.length as u16)) < self.rows))
            .collect::<Vec<Cell>>();


        Ok(())
    }

    fn tick(&mut self) {

        for cell in self.cells.iter_mut(){
            cell.distance += 1;
            cell.y += 1;
            if cell.y > self.rows {
                cell.alive = false;
            }
        }

    }

    fn spawn(&mut self) {

        let mut rng = rng();
        let mut dice = rng.random_range(0..100);

        let mut first = true;
        while dice <= self.spawn_prob || first {
            let x = rng.random_range(0..=self.cols);
            let y = rng.random_range(0..=1);

            let cell = Cell {
                distance: 0,
                alive: true,
                x,
                y
            };

            if self.cells.len() < self.max_cells {
                self.cells.push(cell);
            } else {
            }

            first = false;
            dice = rng.random_range(0..100);
        }
    }
}


fn main() -> io::Result<()> {

    let window = window_size()?;
    let mut stdout = stdout();

    let speed = time::Duration::from_millis(55);
    let max_cells: usize = 200;

    // It means, it only spawns 30% of the time.
    let spawn_prob = 70;

    let length: u8 = (window.rows as u8) / 3 ;
    let mut gradiant_colors = Vec::new();

    for i in 1..=length {
        let r = 0;
        let g = ((255 - 35)/length * i).try_into().unwrap();
        let b = 0;
        let c:Color = Color::Rgb { r: r, g: g, b: b};
        gradiant_colors.push(c);
    }

    let tail = Tail{
        length,
        // gradiant_colors: gradiant_colors.into_iter().rev().collect()
        gradiant_colors
    };

    let mut matrix = Matrix::new(window.columns, window.rows, 
        Color::Black, 
        speed,max_cells,
        vec![], tail,
        spawn_prob);

    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(Hide)?;

    matrix.clear_background(&mut stdout)?;

    loop {
        if poll(matrix.speed)? {
            match read()? {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char(c) => {
                            match c {
                                'b' | 'r' | 'g' => {
                                    let mut gradiant_colors = Vec::new();

                                    for i in 1..=length {
                                        let base= 20;
                                        let rand_color = ((255 - 5)/length * i).try_into().unwrap();

                                        let color:Color; 

                                        if c == 'r' {
                                            color = Color::Rgb { r: rand_color, g: base, b: base};
                                        }else if c ==  'g'  {
                                            color = Color::Rgb { r: base, g: rand_color, b: base};
                                        }else {
                                            color= Color::Rgb { r: base, g: base, b: rand_color};
                                        }

                                        gradiant_colors.push(color);
                                    }
                                    matrix.set_gradiant_vector(gradiant_colors);
                                }
                                'q' => {
                                    break;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if !matrix.pause {
            matrix.print(&mut stdout)?;
            matrix.spawn();
            matrix.tick();
        }
    }

    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
