use std::time::{Duration, Instant};

use uuid::Uuid;

use super::protocol::{Movement, Position};

pub const FALL_CHECK_TICKS: u16 = 100;
pub const FALL_CHECK_DURATION: Duration = Duration::from_secs(3);
const MAX_TRAJECTORY_TICKS: u16 = 128;
const MAX_VALID_POSITION_DIFFERENCE: f64 = 0.01;
const MAX_STATIONARY_MOVEMENTS: u8 = 10;
pub const CAPTCHA_ATTEMPTS: u8 = 3;
const DEFAULT_LIMBO_TIMEOUT: Duration = Duration::from_secs(30);
pub const MOVEMENT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimboPhase {
    WaitingForJoin,
    Falling,
    Captcha,
    Passed,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallResult {
    Ignored,
    Accepted { tick: u16 },
    Invalid,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaResult {
    Correct,
    Retry { attempts_left: u8 },
    Failed,
}

pub struct LimboSession {
    pub phase: LimboPhase,
    pub origin: Option<Position>,
    pub origin_teleport_id: i32,
    synthetic_teleports: i32,
    pub expected_x: f64,
    pub expected_z: f64,
    last_y: f64,
    ticks: u16,
    stationary_packets: u8,
    attempts_left: u8,
    answer: String,
    started_at: Instant,
    last_movement_at: Instant,
    timeout: Duration,
}

impl LimboSession {
    pub fn new() -> Self {
        let seed = Uuid::new_v4().as_u128();
        let answer = format!("{:04}", seed % 10_000);
        let now = Instant::now();
        Self {
            phase: LimboPhase::WaitingForJoin,
            origin: None,
            origin_teleport_id: 0,
            synthetic_teleports: 0,
            expected_x: 0.0,
            expected_z: 0.0,
            last_y: 0.0,
            ticks: 0,
            stationary_packets: 0,
            attempts_left: CAPTCHA_ATTEMPTS,
            answer,
            started_at: now,
            last_movement_at: now,
            timeout: DEFAULT_LIMBO_TIMEOUT,
        }
    }

    pub fn configure(&mut self, timeout_seconds: u64, attempts: u8) {
        self.timeout = Duration::from_secs(timeout_seconds.max(1));
        self.attempts_left = attempts.max(1);
    }

    pub fn start_fall(&mut self, origin: Position, starting_y: f64) {
        self.origin = Some(origin);
        self.origin_teleport_id = origin.teleport_id;
        self.synthetic_teleports = 1; 
        self.expected_x = origin.x;
        self.expected_z = origin.z;
        self.last_y = starting_y;
        self.phase = LimboPhase::Falling;
        self.started_at = Instant::now();
        self.last_movement_at = self.started_at;
    }

    pub fn start_check_after_client_ready(&mut self) {
        if self.phase == LimboPhase::Falling {
            self.started_at = Instant::now();
            self.last_movement_at = self.started_at;
        }
    }

    pub fn answer(&self) -> &str {
        &self.answer
    }

    pub fn regenerate_answer(&mut self) {
        let seed = Uuid::new_v4().as_u128();
        self.answer = format!("{:04}", seed % 10_000);
    }

    pub fn next_teleport_id(&mut self) -> i32 {
        self.synthetic_teleports = self.synthetic_teleports.wrapping_add(1);
        self.origin_teleport_id
            .wrapping_add(self.synthetic_teleports)
    }

    pub fn progress(&self) -> u16 {
        ((self.started_at.elapsed().as_secs_f64() / FALL_CHECK_DURATION.as_secs_f64())
            * f64::from(FALL_CHECK_TICKS))
            .floor()
            .clamp(0.0, f64::from(FALL_CHECK_TICKS)) as u16
    }

    pub fn started_elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn check_seconds_remaining(&self) -> u64 {
        let remaining = FALL_CHECK_DURATION.saturating_sub(self.started_at.elapsed());
        ((remaining.as_millis() + 999) / 1000) as u64
    }

    pub fn finish_fall_if_ready(&mut self) -> bool {
        if self.phase == LimboPhase::Falling && self.started_at.elapsed() >= FALL_CHECK_DURATION {
            self.phase = LimboPhase::Captcha;
            self.started_at = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn expired(&self) -> bool {
        self.started_at.elapsed() > self.timeout
    }

    pub fn movement_timed_out(&self) -> bool {
        self.phase == LimboPhase::Falling
            && self.ticks < FALL_CHECK_TICKS
            && self.last_movement_at.elapsed() > MOVEMENT_TIMEOUT
    }

    pub fn note_movement_packet(&mut self) {
        if self.phase == LimboPhase::Falling {
            self.last_movement_at = Instant::now();
        }
    }

    pub fn observe_movement(&mut self, movement: Movement) -> FallResult {
        if self.phase != LimboPhase::Falling {
            return FallResult::Ignored;
        }
        self.last_movement_at = Instant::now();
        let position = movement.position;
        if (position.x - self.expected_x).abs() > 0.35
            || (position.z - self.expected_z).abs() > 0.35
            || position.y > self.last_y + 0.05
            || movement.on_ground
        {
            self.phase = LimboPhase::Rejected;
            return FallResult::Invalid;
        }

        let delta = self.last_y - position.y;
        self.last_y = position.y;
        if delta == 0.0 {
            self.stationary_packets = self.stationary_packets.saturating_add(1);
            if self.stationary_packets > MAX_STATIONARY_MOVEMENTS {
                self.phase = LimboPhase::Rejected;
                return FallResult::Invalid;
            }
            return FallResult::Ignored;
        }
        self.stationary_packets = 0;
        while self.ticks < MAX_TRAJECTORY_TICKS
            && (delta - loaded_chunk_speed(self.ticks)).abs() > MAX_VALID_POSITION_DIFFERENCE
        {
            self.ticks += 1;
        }
        if self.ticks >= MAX_TRAJECTORY_TICKS {
            self.phase = LimboPhase::Rejected;
            return FallResult::Invalid;
        }
        self.ticks += 1;
        FallResult::Accepted { tick: self.ticks }
    }

    pub fn check_answer(&mut self, input: &str) -> CaptchaResult {
        if self.phase != LimboPhase::Captcha {
            return CaptchaResult::Retry {
                attempts_left: self.attempts_left,
            };
        }
        let input = input.trim().strip_prefix('/').unwrap_or(input.trim());
        if input == self.answer {
            self.phase = LimboPhase::Passed;
            return CaptchaResult::Correct;
        }
        self.attempts_left = self.attempts_left.saturating_sub(1);
        if self.attempts_left == 0 {
            self.phase = LimboPhase::Rejected;
            CaptchaResult::Failed
        } else {
            CaptchaResult::Retry {
                attempts_left: self.attempts_left,
            }
        }
    }
}

fn loaded_chunk_speed(tick: u16) -> f64 {
    -((0.98_f64.powi(i32::from(tick)) - 1.0) * 3.92)
}
