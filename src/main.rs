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

const WORLD_SIZE: u16 = 20;
const X_OFFSET: u16 = 5;


#[derive(Debug, PartialEq)]
struct Point (u16, u16);

impl Rand for Point {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let x = rng.gen_range(X_OFFSET + 1, WORLD_SIZE as u16);
        let y = rng.gen_range(1, WORLD_SIZE as i32);
        Point(x as u16, y as u16)
    }
}

impl Point {
    // TODO: change out for Result type
    fn new(x: u16, y: u16, limit: u16) -> Option<Point> {
        if x >= limit + X_OFFSET || y >= limit  || x <= X_OFFSET || y == 1 {
            None
        } else {
            Some(Point(x, y))
        }
    }
    fn to_screen_coord(&self) -> (Point, Point){
        let Point(x, y) = *self;
        (Point(x * 2, y), Point(x * 2 + 1, y))
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
    key_reader: termion::input::Keys<R>,
    output_writer: W,
}
impl<R: io::Read, W> Snake<R, W> {
    fn new(stdin: R, stdout: W) -> Snake<R, W> {
        let mut body = VecDeque::new();
        body.push_front(Point(3, 1));
        body.push_front(Point(4, 1));
        body.push_front(Point(5, 1));
        body.push_front(Point(6, 1));
        Snake {
            body: body,
            direction: Direction::South,
            world_size: WORLD_SIZE,
            food: Point::rand(&mut rand::thread_rng()),
            key_reader: stdin.keys(),
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
    
    fn next(&self) -> Option<Point> {
        let &Point(head_x, head_y) = self.body.front().expect("body deque should have content");
        let (offset_x, offset_y) = self.direction.get_offset();
        Point::new(
            (head_x as i32 + offset_x) as u16,
            (head_y as i32 + offset_y) as u16,
            self.world_size
        )
    }
    
    fn step(mut snake: Snake<R, W>) -> Option<Snake<R, W>> {
        if let Some(next) = snake.next() {
            // If the snake eats the food, generate a new one and do not pop the tail off
            if next == snake.food {
                snake.gen_food();
            } else {
                snake.body.pop_back();
            }

            // Check to see if the VecDeque of the snake's body includes the destination point
            if !snake.contains(&next) {
                snake.body.push_front(next);
                return Some(snake)
            } 
        }
        None
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
        use termion::event::Key::*;
        match self.key_reader.next().unwrap().unwrap() {
            Char('j') | Char('s') | Down  => self.turn(Direction::South),
            Char('k') | Char('w') | Up    => self.turn(Direction::North),
            Char('h') | Char('a') | Left  => self.turn(Direction::West),
            Char('l') | Char('d') | Right => self.turn(Direction::East),
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
        let block = ' ';
        let (Point(x1, y1), Point(x2, y2)) = piece.to_screen_coord();
        print!("{}{}{}{}", termion::cursor::Goto(x1, y1), color, block, color::Bg(color::Reset));
        print!("{}{}{}{}", termion::cursor::Goto(x2, y2), color, block, color::Bg(color::Reset)); 
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


fn main() {
    let stdin = std::io::stdin();

    
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdout = stdout.into_raw_mode().unwrap();   
    print!("{}", termion::cursor::Hide);


    let mut snake = Snake::new(stdin.lock(), stdout);
    loop {
        snake = Snake::step(snake).unwrap();
        snake.handle_input();
        snake.draw();
        thread::sleep(Duration::from_millis(100));
    }
}
