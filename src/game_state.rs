use crate::{Phase, Position, Token};
use hashbrown::HashMap;

pub struct GameState {
    pub grid_size: usize,
    pub tokens: HashMap<Position, Token>,
    pub selected_token_pos: Option<Position>,
    pub phase: Phase,
}

impl GameState {
    pub fn new(tokens: HashMap<Position, Token>, grid_size: usize) -> Self {
        Self {
            grid_size,
            tokens,
            selected_token_pos: None,
            phase: Phase::default(),
        }
    }
}
