use crate::effect_player::EffectPlayer;
use crate::{Phase, Position, Token};
use hashbrown::HashMap;

pub struct GameState {
    pub grid_size: usize,
    pub tokens: HashMap<Position, Token>,
    pub selected_token_pos: Option<Position>,
    pub phase: Phase,
    pub effect_player: EffectPlayer,
}

impl GameState {
    pub fn new(
        tokens: HashMap<Position, Token>,
        grid_size: usize,
        effect_player: EffectPlayer,
    ) -> Self {
        Self {
            grid_size,
            tokens,
            selected_token_pos: None,
            phase: Phase::default(),
            effect_player,
        }
    }
}
