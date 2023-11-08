use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Print;
use crossterm::terminal::ClearType;
use crossterm::{cursor, terminal, QueueableCommand};
use rand::{random, Rng};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::time::Duration;

/// Defines how much the velocity of the ball should increase with each frame.
const VELOCITY_INCREASE: f64 = 1.003;

#[derive(Debug, Copy, Clone)]
pub struct Position2D {
    x: f64,
    y: f64,
}

impl Position2D {
    pub fn new(x: f64, y: f64) -> Self {
        Position2D { x, y }
    }
}

impl Position2D {
    pub fn to_discrete(&self) -> DiscretePosition2D {
        let x = self.x.round() as usize;
        let y = self.y.round() as usize;

        DiscretePosition2D::new(x, y)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DiscretePosition2D {
    x: usize,
    y: usize,
}

impl DiscretePosition2D {
    pub fn new(x: usize, y: usize) -> Self {
        DiscretePosition2D { x, y }
    }

    pub fn to_continuous(&self) -> Position2D {
        Position2D::new(self.x as f64, self.y as f64)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity2D {
    vx: f64,
    vy: f64,
}

impl Velocity2D {
    pub fn new(vx: f64, vy: f64) -> Self {
        Velocity2D { vx, vy }
    }
}

/// This struct represents a player in the pong game.
pub struct Player {
    extend_up: usize,
    extend_down: usize,
    key_up: KeyCode,
    key_down: KeyCode,
    position: Position2D,
    velocity: Velocity2D,
}

impl Player {
    /// Constructs a new `Player`.
    ///
    /// # Arguments
    /// * `extend_up` - The distance the player extends upwards.
    /// * `extend_down` - The distance the player extends downwards.
    /// * `key_up` - The `KeyCode` for moving the player up.
    /// * `key_down` - The `KeyCode` for moving the player down.
    /// * `position` - The starting `Position2D` of the player.
    ///
    /// # Returns
    /// A new `Player` instance with a default velocity.
    pub fn new(
        extend_up: usize,
        extend_down: usize,
        key_up: KeyCode,
        key_down: KeyCode,
        position: Position2D,
    ) -> Self {
        let velocity = Velocity2D::new(0., 12.0);

        Player {
            extend_up,
            extend_down,
            key_up,
            key_down,
            position,
            velocity,
        }
    }

    /// Updates the player's position based on the keys pressed and the elapsed time.
    ///
    /// # Arguments
    /// * `max_height` - The maximum height of the playing field.
    /// * `pressed_keys` - A reference to a `HashMap` containing `KeyCode`s of currently pressed keys.
    /// * `dt` - The `Duration` since the last update.
    ///
    /// # Remarks
    /// This method updates the `position` of the player based on the `velocity`, `key_up`, and `key_down` inputs.
    /// It also ensures that the player's position does not exceed the maximum height constraints.
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

    /// Checks for collision between the player and a given position.
    ///
    /// # Arguments
    /// * `position` - The `Position2D` for which the collision is to be checked.
    ///
    /// # Returns
    /// `true` if the player collides with the given position, otherwise `false`.
    pub fn collides_with(&self, position: Position2D) -> bool {
        let discrete_position = position.to_discrete();
        let own_discrete_position = self.position.to_discrete();

        own_discrete_position.y - self.extend_down <= discrete_position.y
            && discrete_position.y <= own_discrete_position.y + self.extend_up
            && own_discrete_position.x == discrete_position.x
    }
}

/// This struct represents the ball used in the pong game.
pub struct Ball {
    position: Position2D,
    velocity: Velocity2D,
}

impl Ball {
    /// Constructs a new `Ball` with a random velocity.
    ///
    /// # Arguments
    /// * `position` - The starting `Position2D` of the ball.
    ///
    /// # Returns
    /// A new `Ball` instance.
    pub fn new(position: Position2D) -> Self {
        Ball {
            position,
            velocity: Self::random_ball_velocity(),
        }
    }

    pub fn get_position(&self) -> Position2D {
        self.position
    }

    /// Updates the ball's position based on its velocity, collision with walls or players, and time passed.
    ///
    /// # Arguments
    /// * `max_height` - The maximum height of the game field to handle vertical wall collisions.
    /// * `player1` - A reference to the first player's `Player` instance for potential collision detection.
    /// * `player2` - A reference to the second player's `Player` instance for potential collision detection.
    /// * `dt` - The `Duration` since the last update.
    ///
    /// # Remarks
    /// This method updates the `position` of the ball and handles collision logic with the walls and players.
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

    fn update_if_collision_with_wall(&mut self, max_height: f64, dt: Duration) {
        let next_position = self.calc_next_position(dt);
        if next_position.y <= 0.0 || next_position.y >= max_height {
            self.velocity.vy = -self.velocity.vy;
        }
    }

    fn update_if_collision_with_player1(&mut self, player1: &Player, dt: Duration) {
        let possible_collision_point = self.calculate_collision_point_with_player(player1);
        let next_position = self.calc_next_position(dt);

        if possible_collision_point.x >= next_position.x
            && player1.collides_with(possible_collision_point)
        {
            self.velocity.vx = -self.velocity.vx;
        }
    }

    fn update_if_collision_with_player2(&mut self, player2: &Player, dt: Duration) {
        let possible_collision_point = self.calculate_collision_point_with_player(player2);
        let next_position = self.calc_next_position(dt);

        if possible_collision_point.x <= next_position.x
            && player2.collides_with(possible_collision_point)
        {
            self.velocity.vx = -self.velocity.vx;
        }
    }

    fn calculate_collision_point_with_player(&self, player: &Player) -> Position2D {
        let collision_r = (player.position.x - self.position.x) / self.velocity.vx;
        let possible_collision_x = self.position.x + self.velocity.vx * collision_r;
        let possible_collision_y = self.position.y + self.velocity.vy * collision_r;
        Position2D::new(possible_collision_x, possible_collision_y)
    }

    fn calc_next_position(&self, dt: Duration) -> Position2D {
        let next_position_x = self.position.x + self.velocity.vx * dt.as_secs_f64();
        let next_position_y = self.position.y + self.velocity.vy * dt.as_secs_f64();
        Position2D::new(next_position_x, next_position_y)
    }

    /// Generates a random velocity for the ball when it is initialized or reset.
    ///
    /// # Returns
    /// A `Velocity2D` representing a random velocity within a specified range.
    pub fn random_ball_velocity() -> Velocity2D {
        let vx = match random::<bool>() {
            true => rand::thread_rng().gen_range(10.0..20.0),
            false => rand::thread_rng().gen_range(-20.0..-10.0),
        };
        let vy = rand::thread_rng().gen_range(-6.0..6.0);
        Velocity2D::new(vx, vy)
    }
}

/// The `GameState` struct holds the entire state the pong game.
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
    /// Constructs a new `GameState`.
    ///
    /// # Arguments
    /// * `width` - The width of the game field.
    /// * `height` - The height of the game field.
    /// * `extend_player_height_up` - The extension of player's reach upwards.
    /// * `extend_player_height_down` - The extension of player's reach downwards.
    ///
    /// # Returns
    /// A new `GameState` instance with initialized players and ball.
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

    /// Updates the state of the game including player positions, ball position, and score based on the time elapsed and the pressed keys.
    ///
    /// # Arguments
    /// * `pressed_keys` - A `HashMap` representing the keys currently pressed by the players.
    /// * `dt` - The `Duration` since the last update.
    pub fn update(&mut self, pressed_keys: HashMap<KeyCode, KeyEvent>, dt: Duration) {
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

    /// Renders the current game state to the terminal.
    ///
    /// # Returns
    /// An `io::Result` indicating the outcome of the render operation.
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
                let current_cell = DiscretePosition2D::new(x, y);

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

    fn initial_player1_position(_: usize, height: usize) -> Position2D {
        let x = 0.0;
        let y = (height as f64) / 2.;

        Position2D::new(x, y)
    }

    fn initial_player2_position(width: usize, height: usize) -> Position2D {
        let x = width as f64;
        let y = (height as f64) / 2.;

        Position2D::new(x, y)
    }

    fn initial_ball_position(width: usize, height: usize) -> Position2D {
        let x = (width as f64) / 2.;
        let y = (height as f64) / 2.;

        Position2D::new(x, y)
    }
}
