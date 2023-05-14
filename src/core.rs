use crate::{Phase, Token};
use hashbrown::HashMap;
use macroquad::math::Vec2;

const GRID_SIZE: i32 = 32;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Self { x, y }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }
    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x
    }
    pub fn set_y(&mut self, y: i32) {
        self.y = y
    }

    pub fn to_world(&self) -> (f32, f32) {
        let x = (self.x * GRID_SIZE) as f32;
        let y = (self.y * GRID_SIZE) as f32;
        (x, y)
    }

    pub fn lerp_to_world(&self, other: &Self, t: f32) -> (f32, f32) {
        let (x, y) = self.to_world();
        let (x2, y2) = other.to_world();
        let x = x * (1.0 - t) + (x2 * t);
        let y = y * (1.0 - t) + (y2 * t);
        (x, y)
    }

    pub fn from_world((x, y): (f32, f32)) -> Self {
        let x = (x / GRID_SIZE as f32).floor() as i32;
        let y = (y / GRID_SIZE as f32).floor() as i32;
        Self { x, y }
    }
    pub fn from_world_vec2(vec: Vec2) -> Self {
        Self::from_world((vec.x, vec.y))
    }

    pub fn is_adjacent(&self, other: &Self) -> bool {
        let x = self.x - other.x;
        let y = self.y - other.y;
        (x.abs() + y.abs()) == 1
    }

    pub fn neighbours(&self) -> Vec<Self> {
        let mut positions = Vec::new();
        positions.push(Self::new(self.x - 1, self.y));
        positions.push(Self::new(self.x + 1, self.y));
        positions.push(Self::new(self.x, self.y - 1));
        positions.push(Self::new(self.x, self.y + 1));
        positions
    }
}

#[derive(Debug, Clone)]
pub struct AnimationPosition {
    start: Position,
    pub end: Position,
    duration: f64,
    start_time: Option<f64>,
}

impl AnimationPosition {
    pub fn new_start(start: Position, end: Position, duration: f64) -> Self {
        Self {
            start,
            end,
            duration,
            start_time: Some(macroquad::time::get_time()),
        }
    }
    pub fn new(start: Position, end: Position, duration: f64) -> Self {
        Self {
            start,
            end,
            duration,
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(macroquad::time::get_time());
    }

    pub fn is_done(&self) -> bool {
        let now = macroquad::time::get_time();
        let elapsed = now - self.start_time.unwrap_or(now);
        elapsed > self.duration
    }

    pub fn get(&self) -> (f32, f32) {
        let now = macroquad::time::get_time();
        let elapsed = now - self.start_time.unwrap_or(now);
        let t = (elapsed / self.duration).clamp(0., 1.);
        self.start.lerp_to_world(&self.end, t as f32)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MatchKind {
    // i.e. XXX
    Three,
    // i.e. XXXX
    Four,
    // i.e. XXXXX
    Five,
    // i.e. XXX
    //      X
    //      X
    LShape,
    // i.e. XXX
    //       X
    //       X
    TShape,
    // i.e. XXXXX
    //        X
    //        X
    SuperTShape,
}
