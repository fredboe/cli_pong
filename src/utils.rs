use std::time::{Duration, Instant};

/// # Explanation
/// The game loop is an iterator that waits when the next function is called if the execution is faster
/// than the frame rate allows.
pub struct GameLoop {
    frame: u64,
    current_frame_start: Instant,
    duration_per_frame: Duration,
}

impl GameLoop {
    pub fn new(duration_per_frame: Duration) -> GameLoop {
        let current_frame_start = Instant::now();
        GameLoop {
            frame: 0,
            current_frame_start,
            duration_per_frame,
        }
    }

    pub fn from_fps(fps: usize) -> GameLoop {
        let duration_per_frame = Duration::from_secs_f32(1.0 / (fps as f32));
        Self::new(duration_per_frame)
    }
}

impl Iterator for GameLoop {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let end_time = self.current_frame_start + self.duration_per_frame;
        let now = Instant::now();
        if now <= end_time {
            let sleep_time = end_time - Instant::now();
            std::thread::sleep(sleep_time);
        }

        let frame_number = self.frame;
        self.frame += 1;

        let next_frame_start_time = Instant::now();
        self.current_frame_start = next_frame_start_time;

        Some(frame_number)
    }
}

