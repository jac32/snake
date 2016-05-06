extern crate rand;
extern crate ncurses;

use ncurses::*;
use rand::{Rand, Rng};

use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

const WORLD_SIZE: usize =  10;
    

#[derive(Debug, PartialEq)]
struct Point (i32, i32);

impl Rand for Point {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let x: i32 = rng.gen_range(0, WORLD_SIZE as i32);
        let y: i32 = rng.gen_range(0, WORLD_SIZE as i32);
        Point(x, y)
    }
}

impl Point {
    fn new(x: i32, y: i32, limit: usize) -> Option<Point> {
        if x >= limit as i32 || y >= limit as i32 || x < 0 || y < 0 {
            None
        } else {
            Some(Point(x, y))
        }
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
            Direction::West => (0, -1),
            Direction::East => (0, 1),
            Direction::North => (-1, 0),
            Direction::South => (1, 0),
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


#[derive(Debug)]
struct Snake {
    body: VecDeque<Point>,
    direction: Direction,
    world_size: usize,
    food: Point,
}
impl Snake {
    fn new() -> Snake {
        let mut body = VecDeque::new();
        body.push_front(Point(0, 1));
        body.push_front(Point(1, 1));
        body.push_front(Point(2, 1));
        body.push_front(Point(3, 1));
        Snake {
            body: body,
            direction: Direction::South,
            world_size: WORLD_SIZE,
            food: Point::rand(&mut rand::thread_rng()),
        }
    }

    fn next(&self) -> Option<Point> {
        let &Point(head_x, head_y) = self.body.front().unwrap();
        let (offset_x, offset_y) = self.direction.get_offset();
        Point::new(head_x + offset_x, head_y + offset_y, self.world_size)
    }
    
    fn step(mut snake: Snake) -> Option<Snake> {
        if let Some(next) = snake.next() {
            // If the snake eats the food, generate a new one and do not pop the tail off
            if next == snake.food {
                snake.food = Point::rand(&mut rand::thread_rng());
            } else {
                snake.body.pop_back();
            }

            // Check to see if the VecDeque of the snake's body includes the destination point
            if snake.body.iter().all(|point| { *point != next }) {
                snake.body.push_front(next);
                return Some(snake)
            } 
        }
        None
    }

    fn turn(&mut self, direction: Direction) {
        if self.direction.get_opposite() != direction {
            self.direction = direction
        }
    }

    fn display(&self) {
        let mut chars = [['-'; WORLD_SIZE]; WORLD_SIZE];
        for &Point(x, y) in &self.body {
            chars[x as usize][y as usize] = 'X';
        }
        let &Point(x, y) = self.body.front().unwrap();
        chars[x as usize][y as usize] = 'O';
        let Point(x, y) = self.food;
        chars[x as usize][y as usize] = '$';
        
        for i in 0..WORLD_SIZE {
            for j in 0..WORLD_SIZE {
                mvprintw(i as i32, (j*2) as i32 ,&format!("{} ", chars[i][j]));
            }
            printw("\n");
        }
    }
}
    
fn main() {
    let mut snake = Snake::new();
    /* Start ncurses. */
    initscr();
    cbreak();
    timeout(250);
    keypad(stdscr, true);
    snake.display();

    
    /* Update the screen. */
    refresh();
    /* Wait for a key press. */
    loop {
        let ch = getch();
        if ch == ncurses::KEY_LEFT { snake.turn(Direction::West); };
        if ch == ncurses::KEY_RIGHT { snake.turn(Direction::East); };
        if ch == ncurses::KEY_UP { snake.turn(Direction::North); };
        if ch == ncurses::KEY_DOWN { snake.turn(Direction::South); };
        
        if let Some(moved_snake) = Snake::step(snake) {
            snake = moved_snake;
            snake.display();
            refresh();    
        } else {
            // Print ASCII GameOver
            printw("  _____               ____              
 / ___/__ ___ _  ___ / __ \\_  _____ ____
/ (_ / _ `/  ' \\/ -_) /_/ / |/ / -_) __/
\\___/\\_,_/_/_/_/\\__/\\____/|___/\\__/_/   ");
            
            refresh();
            thread::sleep(Duration::new(10, 0));
            break;
        }
    }
    
    /* Terminate ncurses. */
    endwin();
}