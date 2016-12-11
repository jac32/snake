extern crate termion;
extern crate rand;

use termion::color;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use rand::{Rand, Rng};

use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use std::io;
use std::io::Read;

const WORLD_SIZE: u16 = 20;
const X_OFFSET: u16 = 5;

#[derive(PartialEq)]
enum Event {
    Death,
    EatFood,
}

enum Error {
    OutOfBounds,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point (u16, u16);

impl Rand for Point {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let x = rng.gen_range(X_OFFSET + 1, WORLD_SIZE as u16);
        let y = rng.gen_range(2, WORLD_SIZE as i32);
        Point(x as u16, y as u16)
    }
}

impl Point {
    // TODO: change out for Result type
    fn new(x: u16, y: u16, limit: u16) -> Result<Point, Error> {
        if x >= limit + X_OFFSET || y >= limit  || x <= X_OFFSET || y == 1 {
            Err(Error::OutOfBounds)
        } else {
            Ok(Point(x, y))
        }
    }

    fn to_screen_coord(&self) -> Point {
        let Point(x, y) = *self;
        Point(x * 2, y)
    }
}


#[derive(Debug, PartialEq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn get_offset(&self) -> (i32, i32){
        match *self {
            Direction::West => (-1, 0),
            Direction::East => (1, 0),
            Direction::North => (0, -1),
            Direction::South => (0, 1),
        } 
    }
    fn get_opposite(&self) -> Direction {
        match *self {
            Direction::West => Direction::East,
            Direction::East => Direction::West,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
        }
    }
}


struct Snake<R, W> {
    body: VecDeque<Point>,
    direction: Direction,
    world_size: u16,
    food: Point,
    key_reader: io::Bytes<R>,
    output_writer: W,
}
impl<R: io::Read, W> Snake<R, W> {
    fn new(stdin: R, stdout: W) -> Snake<R, W> {
        let mut body = VecDeque::new();
        body.push_front(Point(3, 3));
        body.push_front(Point(4, 3));
        body.push_front(Point(5, 3));
        body.push_front(Point(6, 3));
        Snake {
            body: body,
            direction: Direction::South,
            world_size: WORLD_SIZE,
            food: Point::rand(&mut rand::thread_rng()),
            key_reader: stdin.bytes(),
            output_writer: stdout,
        }
    }

    fn contains(&self, point: &Point) -> bool {
        if self.body.iter().all(|part| { part != point }) {
            false
        } else {
            true
        }
    }
    
    fn next(&self) -> Result<Point, Error> {
        let &Point(head_x, head_y) = self.body.front().expect("body deque should have content");
        let (offset_x, offset_y) = self.direction.get_offset();
        Point::new(
            (head_x as i32 + offset_x) as u16,
            (head_y as i32 + offset_y) as u16,
            self.world_size
        )
    }

    fn step(&mut self) -> Option<Event> {
        self.next().map(|next| {
            
            if self.body.contains(&next) {
                return Some(Event::Death)
            }

            if next != self.food {
                self.body.pop_back();
                self.body.push_front(next);
                None
            } else {
                self.body.push_front(next);
                Some(Event::EatFood)
            }
        }).unwrap_or(Some(Event::Death))
    }

    fn gen_food(&mut self) {
        let food =  Point::rand(&mut rand::thread_rng());
        if self.contains(&food) {
            self.gen_food();
        } else {
            self.food = food;
        }
    }

    fn turn(&mut self, direction: Direction) {
        if self.direction.get_opposite() != direction {
            self.direction = direction
        }
    }

    fn handle_input(&mut self) {
        match self.key_reader.next().unwrap_or(Ok(b'x')).unwrap() {
            b'j' | b's' => self.turn(Direction::South),
            b'k' | b'w' => self.turn(Direction::North),
            b'h' | b'a' => self.turn(Direction::West),
            b'l' | b'd' => self.turn(Direction::East),
            _ => println!("nothing"),
        }
    }

    fn draw_body_piece(piece: &Point) {
        Self::draw_block(piece, color::Bg(color::Green));
    }

    fn draw_food_piece(piece: &Point) {
        Self::draw_block(piece, color::Bg(color::Red));
    }

    fn draw_block<C: std::fmt::Display>(piece: &Point, color: C) {
        let block = "  ";
        let Point(x, y) = piece.to_screen_coord();
        print!("{}{}{}{}", termion::cursor::Goto(x, y), color, block, color::Bg(color::Reset));
    }
    

    fn draw(&self) {
        print!("{}", termion::clear::All);
        for i in 0..WORLD_SIZE {
            Self::draw_block(&Point(X_OFFSET, i+1), color::Bg(color::Blue));
            Self::draw_block(&Point(i + X_OFFSET, 0), color::Bg(color::Blue));
            Self::draw_block(&Point(WORLD_SIZE + X_OFFSET, i+1), color::Bg(color::Blue));
            Self::draw_block(&Point(i + X_OFFSET, WORLD_SIZE), color::Bg(color::Blue));
        }
        for piece in &self.body {
            Self::draw_body_piece(piece);
        }
        Self::draw_food_piece(&self.food);
        println!("");
    }
}

fn clear_screen() {
    print!("{}", termion::clear::All);
    print!("{}", termion::cursor::Goto(1, 1));
}

fn hide_cursor() {
    print!("{}", termion::cursor::Hide);
}

fn main() {
    // Establish lock on stdout and enter raw mode for increased control
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdout = stdout.into_raw_mode().unwrap();   

    hide_cursor();

    let mut snake = Snake::new(termion::async_stdin(), stdout);

    let mut score = 0;
    loop {
        match snake.step() {
            Some(Event::Death) => {
                thread::sleep(Duration::from_millis(600));
                clear_screen();
                break
            },
            safe_event => {
                if safe_event == Some(Event::EatFood) {
                    score += 1;
                    snake.gen_food();
                }
                snake.handle_input();
                snake.draw();
                thread::sleep(Duration::from_millis(100));
            }

        }
    }

    clear_screen();
    println!("Your score was: {}\n", score);
}
