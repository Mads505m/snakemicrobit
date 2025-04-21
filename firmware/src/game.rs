use heapless::{FnvIndexSet, spsc::Queue};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Coords {
    row: i8,
    col: i8,
}

impl Coords {
    fn random(rng: &mut Prng, exclude: Option<&FnvIndexSet<Coords, 32>>) -> Self {
        let mut coords = Coords {
            row: ((rng.random_u32() as usize) % 5) as i8,
            col: ((rng.random_u32() as usize) % 5) as i8
        };
        while exclude.is_some_and(|exc| exc.contains(&coords)) {
            coords = Coords {
                row: ((rng.random_u32() as usize) % 5) as i8,
                col: ((rng.random_u32() as usize) % 5) as i8
            };
        }
        coords
    }

    fn is_out_of_bounds(&self) -> bool {
        self.row < 0 || self.row >= 5 || self.col < 0 || self.col >= 5
    }
}

struct Prng {
    value: u32
}

impl Prng {
    fn new(seed: u32) -> Self {
        Self { value: seed }
    }
    fn xorshift32(mut input: u32) -> u32 {
        input ^= input << 13;
        input ^= input >> 17;
        input ^= input << 5;
        input
    }
    fn random_u32(&mut self) -> u32 {
        self.value = Self::xorshift32(self.value);
        self.value
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug, Copy, Clone)]
pub enum Turn {
    Left,
    Right,
    None
}

/// Current status of the game.
pub enum GameStatus {
    Won,
    Lost,
    Ongoing
}

enum StepOutcome {
    Full(Coords),
    Collision(Coords),
    Eat(Coords),
    Move(Coords)
}

struct Snake {
    head: Coords,
    tail: Queue<Coords, 32>,
    coord_set: FnvIndexSet<Coords, 32>,
    direction: Direction
}

impl Snake {
    fn new() -> Self {
        let head = Coords { row: 2, col: 2 };
        let initial_tail = Coords { row: 2, col: 1 };
        let mut tail = Queue::new();
        tail.enqueue(initial_tail).unwrap();
        let mut coord_set: FnvIndexSet<Coords, 32> = FnvIndexSet::new();
        coord_set.insert(head).unwrap();
        coord_set.insert(initial_tail).unwrap();
        Self {
            head,
            tail,
            coord_set,
            direction: Direction::Right,
        }
    }

    fn move_snake(&mut self, coords: Coords, extend: bool) {
        // current head becomes part of the tail
        self.tail.enqueue(self.head).unwrap();
        self.head = coords;
        self.coord_set.insert(coords).unwrap();
        if !extend {
            let back = self.tail.dequeue().unwrap();
            self.coord_set.remove(&back);
        }
    }

    fn turn_right(&mut self) {
        self.direction = match self.direction {
            Direction::Up    => Direction::Right,
            Direction::Down  => Direction::Left,
            Direction::Left  => Direction::Up,
            Direction::Right => Direction::Down,
        };
    }

    fn turn_left(&mut self) {
        self.direction = match self.direction {
            Direction::Up    => Direction::Left,
            Direction::Down  => Direction::Right,
            Direction::Left  => Direction::Down,
            Direction::Right => Direction::Up,
        };
    }

    fn turn(&mut self, direction: Turn) {
        match direction {
            Turn::Left  => self.turn_left(),
            Turn::Right => self.turn_right(),
            Turn::None  => ()
        }
    }
}

pub(crate) struct Game {
    rng: Prng,
    snake: Snake,
    food_coords: Coords,
    speed: u8,
    pub(crate) status: GameStatus,
    score: u8
}

impl Game {
    pub(crate) fn new(rng_seed: u32) -> Self {
        let mut rng = Prng::new(rng_seed);
        let snake = Snake::new();
        let food_coords = Coords::random(&mut rng, Some(&snake.coord_set));
        Self {
            rng,
            snake,
            food_coords,
            speed: 1,
            status: GameStatus::Ongoing,
            score: 0
        }
    }

    pub(crate) fn reset(&mut self) {
        self.snake = Snake::new();
        self.place_food();
        self.speed = 1;
        self.status = GameStatus::Ongoing;
        self.score = 0;
    }

    fn place_food(&mut self) -> Coords {
        let coords = Coords::random(&mut self.rng, Some(&self.snake.coord_set));
        self.food_coords = coords;
        coords
    }

    fn wraparound(&self, coords: Coords) -> Coords {
        if coords.row < 0 {
            Coords { row: 4, ..coords }
        } else if coords.row >= 5 {
            Coords { row: 0, ..coords }
        } else if coords.col < 0 {
            Coords { col: 4, ..coords }
        } else {
            Coords { col: 0, ..coords }
        }
    }

    fn get_next_move(&self) -> Coords {
        let head = &self.snake.head;
        let next = match self.snake.direction {
            Direction::Up    => Coords { row: head.row - 1, col: head.col },
            Direction::Down  => Coords { row: head.row + 1, col: head.col },
            Direction::Left  => Coords { row: head.row, col: head.col - 1 },
            Direction::Right => Coords { row: head.row, col: head.col + 1 },
        };
        if next.is_out_of_bounds() {
            self.wraparound(next)
        } else {
            next
        }
    }

    fn get_step_outcome(&self) -> StepOutcome {
        let next = self.get_next_move();
        if self.snake.coord_set.contains(&next) {
            if next != *self.snake.tail.peek().unwrap() {
                StepOutcome::Collision(next)
            } else {
                StepOutcome::Move(next)
            }
        } else if next == self.food_coords {
            if self.snake.tail.len() == 23 {
                StepOutcome::Full(next)
            } else {
                StepOutcome::Eat(next)
            }
        } else {
            StepOutcome::Move(next)
        }
    }

    fn handle_step_outcome(&mut self, outcome: StepOutcome) {
        self.status = match outcome {
            StepOutcome::Collision(_) => GameStatus::Lost,
            StepOutcome::Full(_)      => GameStatus::Won,
            StepOutcome::Eat(c) => {
                self.snake.move_snake(c, true);
                self.place_food();
                self.score += 1;
                if self.score % 5 == 0 {
                    self.speed += 1;
                }
                GameStatus::Ongoing
            },
            StepOutcome::Move(c) => {
                self.snake.move_snake(c, false);
                GameStatus::Ongoing
            }
        }
    }

    pub(crate) fn step(&mut self, turn: Turn) {
        self.snake.turn(turn);
        let outcome = self.get_step_outcome();
        self.handle_step_outcome(outcome);
    }

    pub(crate) fn step_len_ms(&self) -> u32 {
        let result = 1000 - 200 * ((self.speed as i32) - 1);
        if result < 200 { 200 } else { result as u32 }
    }

    pub(crate) fn game_matrix(
        &self,
        head_brightness: u8,
        tail_brightness: u8,
        food_brightness: u8
    ) -> [[u8; 5]; 5] {
        let mut values = [[0u8; 5]; 5];
        // Snake's head
        values[self.snake.head.row as usize][self.snake.head.col as usize] = head_brightness;
        // Snake's tail
        for t in &self.snake.tail {
            values[t.row as usize][t.col as usize] = tail_brightness;
        }
        // Food
        values[self.food_coords.row as usize][self.food_coords.col as usize] = food_brightness;
        values
    }

    pub(crate) fn score_matrix(&self) -> [[u8; 5]; 5] {
        let mut values = [[0u8; 5]; 5];
        let full_rows = (self.score as usize) / 5;
        for r in 0..full_rows {
            values[r] = [1; 5];
        }
        for c in 0..(self.score as usize) % 5 {
            values[full_rows][c] = 1;
        }
        values
    }
}
