use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Print;
use crossterm::terminal::ClearType;
use crossterm::{cursor, terminal, QueueableCommand};
use rand::{random, Rng};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::time::Duration;

const VELOCITY_INCREASE: f64 = 1.003;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    x: f64,
    y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }
}

impl Position {
    pub fn to_discrete(&self) -> DiscretePosition {
        let x = self.x.round() as usize;
        let y = self.y.round() as usize;

        DiscretePosition::new(x, y)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DiscretePosition {
    x: usize,
    y: usize,
}

impl DiscretePosition {
    pub fn new(x: usize, y: usize) -> Self {
        DiscretePosition { x, y }
    }

    pub fn to_continuous(&self) -> Position {
        Position::new(self.x as f64, self.y as f64)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity {
    vx: f64,
    vy: f64,
}

impl Velocity {
    pub fn new(vx: f64, vy: f64) -> Self {
        Velocity { vx, vy }
    }
}

pub struct Player {
    extend_up: usize,
    extend_down: usize,
    key_up: KeyCode,
    key_down: KeyCode,
    position: Position,
    velocity: Velocity,
}

impl Player {
    pub fn new(
        extend_up: usize,
        extend_down: usize,
        key_up: KeyCode,
        key_down: KeyCode,
        position: Position,
    ) -> Self {
        let velocity = Velocity::new(0., 12.0);

        Player {
            extend_up,
            extend_down,
            key_up,
            key_down,
            position,
            velocity,
        }
    }

    pub fn update_position(
        &mut self,
        max_height: f64,
        pressed_keys: &HashMap<KeyCode, KeyEvent>,
        dt: Duration,
    ) {
        if pressed_keys.contains_key(&self.key_up) {
            self.position.x += self.velocity.vx * dt.as_secs_f64();
            self.position.y += self.velocity.vy * dt.as_secs_f64();
        }
        if pressed_keys.contains_key(&self.key_down) {
            self.position.x -= self.velocity.vx * dt.as_secs_f64();
            self.position.y -= self.velocity.vy * dt.as_secs_f64();
        }

        self.position.y = self
            .position
            .y
            .min(max_height - self.extend_up as f64)
            .max(0.0 + self.extend_down as f64);
    }

    pub fn collides_with(&self, position: Position) -> bool {
        let discrete_position = position.to_discrete();
        let own_discrete_position = self.position.to_discrete();

        own_discrete_position.y - self.extend_down <= discrete_position.y
            && discrete_position.y <= own_discrete_position.y + self.extend_up
            && own_discrete_position.x == discrete_position.x
    }
}

pub struct Ball {
    position: Position,
    velocity: Velocity,
}

impl Ball {
    pub fn new(position: Position) -> Self {
        Ball {
            position,
            velocity: Self::random_ball_velocity(),
        }
    }

    pub fn get_position(&self) -> Position {
        self.position
    }

    pub fn update_position(
        &mut self,
        max_height: f64,
        player1: &Player,
        player2: &Player,
        dt: Duration,
    ) {
        self.update_if_collision_with_wall(max_height, dt);

        if self.velocity.vx <= 0.0 {
            self.update_if_collision_with_player1(player1, dt);
        } else {
            self.update_if_collision_with_player2(player2, dt);
        }

        self.position = self.calc_next_position(dt);
        self.velocity.vx *= VELOCITY_INCREASE;
        self.velocity.vy *= VELOCITY_INCREASE;
    }

    pub fn update_if_collision_with_wall(&mut self, max_height: f64, dt: Duration) {
        let next_position = self.calc_next_position(dt);
        if next_position.y <= 0.0 || next_position.y >= max_height {
            self.velocity.vy = -self.velocity.vy;
        }
    }

    pub fn update_if_collision_with_player1(&mut self, player1: &Player, dt: Duration) {
        let possible_collision_point = self.calculate_collision_point_with_player(player1);
        let next_position = self.calc_next_position(dt);

        if possible_collision_point.x >= next_position.x
            && player1.collides_with(possible_collision_point)
        {
            self.velocity.vx = -self.velocity.vx;
        }
    }

    pub fn update_if_collision_with_player2(&mut self, player2: &Player, dt: Duration) {
        let possible_collision_point = self.calculate_collision_point_with_player(player2);
        let next_position = self.calc_next_position(dt);

        if possible_collision_point.x <= next_position.x
            && player2.collides_with(possible_collision_point)
        {
            self.velocity.vx = -self.velocity.vx;
        }
    }

    fn calculate_collision_point_with_player(&self, player: &Player) -> Position {
        let collision_r = (player.position.x - self.position.x) / self.velocity.vx;
        let possible_collision_x = self.position.x + self.velocity.vx * collision_r;
        let possible_collision_y = self.position.y + self.velocity.vy * collision_r;
        Position::new(possible_collision_x, possible_collision_y)
    }

    fn calc_next_position(&self, dt: Duration) -> Position {
        let next_position_x = self.position.x + self.velocity.vx * dt.as_secs_f64();
        let next_position_y = self.position.y + self.velocity.vy * dt.as_secs_f64();
        Position::new(next_position_x, next_position_y)
    }

    pub fn random_ball_velocity() -> Velocity {
        let vx = match random::<bool>() {
            true => rand::thread_rng().gen_range(10.0..20.0),
            false => rand::thread_rng().gen_range(-20.0..-10.0),
        };
        let vy = rand::thread_rng().gen_range(-6.0..6.0);
        Velocity::new(vx, vy)
    }
}

pub struct GameState {
    width: usize,
    height: usize,
    player1_score: usize,
    player2_score: usize,
    player1: Player,
    player2: Player,
    ball: Ball,
}

impl GameState {
    pub fn new(
        width: usize,
        height: usize,
        extend_player_height_up: usize,
        extend_player_height_down: usize,
    ) -> Self {
        let player1 = Player::new(
            extend_player_height_up,
            extend_player_height_down,
            KeyCode::Char('w'),
            KeyCode::Char('s'),
            Self::initial_player1_position(width, height),
        );

        let player2 = Player::new(
            extend_player_height_up,
            extend_player_height_down,
            KeyCode::Up,
            KeyCode::Down,
            Self::initial_player2_position(width, height),
        );

        let ball = Ball::new(Self::initial_ball_position(width, height));

        GameState {
            width,
            height,
            player1_score: 0,
            player2_score: 0,
            player1,
            player2,
            ball,
        }
    }

    pub fn update(&mut self, dt: Duration, pressed_keys: HashMap<KeyCode, KeyEvent>) {
        if pressed_keys.contains_key(&KeyCode::Char('r')) {
            self.reset_ball_and_players();
            return;
        }

        self.player1
            .update_position(self.height as f64, &pressed_keys, dt);
        self.player2
            .update_position(self.height as f64, &pressed_keys, dt);

        self.ball
            .update_position(self.height as f64, &self.player1, &self.player2, dt);

        self.update_score();
    }

    fn update_score(&mut self) {
        if self.ball.velocity.vx <= 0.0 && self.ball.position.x < self.player1.position.x {
            self.player2_score += 1;
            self.reset_ball_and_players();
        } else if self.ball.velocity.vx > 0.0 && self.ball.position.x > self.player2.position.x {
            self.player1_score += 1;
            self.reset_ball_and_players();
        }
    }

    fn reset_ball_and_players(&mut self) {
        self.player1.position = Self::initial_player1_position(self.width, self.height);
        self.player2.position = Self::initial_player2_position(self.width, self.height);
        self.ball.position = Self::initial_ball_position(self.width, self.height);

        self.ball.velocity = Ball::random_ball_velocity();
    }

    pub fn display(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        stdout.queue(terminal::Clear(ClearType::All))?;
        stdout.queue(cursor::Hide)?;
        stdout.queue(cursor::MoveTo(0, 0))?;

        stdout.queue(Print(format!(
            "\r\nGoals of player1: {},  Goals of player2: {}\r\n\r\n",
            self.player1_score, self.player2_score
        )))?;

        for _ in 0..=self.width {
            stdout.queue(Print('\u{2588}'))?;
        }
        stdout.queue(Print("\r\n"))?;

        for y in (0..=self.height).rev() {
            for x in 0..=self.width {
                let current_cell = DiscretePosition::new(x, y);

                let character = if self.ball.get_position().to_discrete() == current_cell {
                    '\u{25CF}'
                } else if self.player1.collides_with(current_cell.to_continuous()) {
                    '\u{2588}'
                } else if self.player2.collides_with(current_cell.to_continuous()) {
                    '\u{2588}'
                } else {
                    ' '
                };

                stdout.queue(Print(character))?;
            }
            stdout.queue(Print("\r\n"))?;
        }

        for _ in 0..=self.width {
            stdout.queue(Print('\u{2588}'))?;
        }
        stdout.queue(Print("\r\n"))?;

        stdout.queue(cursor::Show)?;
        stdout.flush()?;

        Ok(())
    }

    fn initial_player1_position(_: usize, height: usize) -> Position {
        let x = 0.0;
        let y = (height as f64) / 2.;

        Position::new(x, y)
    }

    fn initial_player2_position(width: usize, height: usize) -> Position {
        let x = width as f64;
        let y = (height as f64) / 2.;

        Position::new(x, y)
    }

    fn initial_ball_position(width: usize, height: usize) -> Position {
        let x = (width as f64) / 2.;
        let y = (height as f64) / 2.;

        Position::new(x, y)
    }
}
