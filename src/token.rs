use crate::core::{AnimationPosition, Position};
use crate::token_grid;
use hashbrown::HashMap;
use macroquad::color::WHITE;
use macroquad::material::Material;
use macroquad::math::vec4;
use macroquad::prelude::{draw_texture, gl_use_default_material, gl_use_material, Texture2D};
use rand::distributions::{Distribution, Standard};

pub const ANIMATION_TIME_PER_TILE: f64 = 02.2;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum TokenType {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Modifier {
    None,
    Hover,
    Selected,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub type_: TokenType,
    animation_position: Option<AnimationPosition>,
    texture: Texture2D,
}

impl Token {
    pub(crate) fn new(type_: TokenType, texture: Texture2D) -> Token {
        Self {
            type_,
            animation_position: None,
            texture,
        }
    }

    pub fn update(&mut self) {
        if let Some(ref mut animation_position) = self.animation_position {
            if animation_position.is_done() {
                self.animation_position = None;
            }
        }
    }

    pub fn draw(
        &self,
        grid_position: &Position,
        modifier: &Modifier,
        shader_material: Material,
        outline_texture: &Texture2D,
    ) {
        let (x, y) = match self.animation_position {
            Some(ref animation_position) => animation_position.get(),
            None => grid_position.to_world(),
        };
        match modifier {
            Modifier::None => gl_use_default_material(),
            Modifier::Hover => {
                shader_material.set_uniform("test_color", vec4(1.5, 1.5, 1.5, 1.));
                gl_use_material(shader_material);
            }
            Modifier::Selected => {
                gl_use_default_material();
                draw_texture(*outline_texture, x, y, WHITE);
            }
        }
        draw_texture(self.texture, x, y, WHITE);
    }

    pub fn animate_move_to(
        &mut self,
        from_position: Position,
        to_position: Position,
        animation_time: f64,
    ) {
        self.animation_position = Some(AnimationPosition::new_start(
            from_position,
            to_position,
            animation_time,
        ));
    }

    pub fn is_animating(&self) -> bool {
        self.animation_position.is_some()
    }
}

impl TokenType {
    pub const ALL: [TokenType; 3] = [Self::Red, Self::Green, Self::Blue];
    pub fn to_sprite_name(&self) -> &str {
        match self {
            Self::Red => "res/red_token.png",
            Self::Green => "res/green_token.png",
            Self::Blue => "res/blue_token.png",
        }
    }
}

impl Distribution<TokenType> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> TokenType {
        match rng.gen_range(0..3) {
            0 => TokenType::Red,
            1 => TokenType::Green,
            2 => TokenType::Blue,
            _ => unreachable!(),
        }
    }
}

impl Modifier {
    fn valid_transition_to(&self, other: Modifier) -> bool {
        match (self, other) {
            (Self::None, Self::Hover)
            | (Self::Hover, Self::None)
            | (Self::Hover, Self::Selected)
            | (Self::Selected, Self::None) => true,
            (Self::Selected, Self::Hover) => false,
            _ => false,
        }
    }
}

pub fn is_valid_swap(
    tokens: &mut HashMap<Position, Token>,
    pos1: &Position,
    pos2: &Position,
) -> bool {
    if pos1.is_adjacent(pos2) {
        // Must trigger a match
        println!(
            "Swapping tokens to check for matches. Pos1: {:?}, Pos2: {:?}",
            pos1, pos2
        );
        swap_tokens(tokens, pos1.clone(), pos2.clone(), false);
        let matches_exist = {
            let matches = token_grid::check_for_matches(tokens, &[pos1.clone(), pos2.clone()]);
            !matches.is_empty()
        };
        println!(
            "Swapping back tokens after checking. Pos1: {:?}, Pos2: {:?}",
            pos1, pos2
        );
        swap_tokens(tokens, pos1.clone(), pos2.clone(), false);

        matches_exist
    } else {
        false
    }
}

pub fn swap_tokens(
    tokens: &mut HashMap<Position, Token>,
    token_a_pos: Position,
    token_b_pos: Position,
    animate: bool,
) {
    let mut token_a = tokens.remove(&token_a_pos).unwrap();
    let mut token_b = tokens.remove(&token_b_pos).unwrap();
    if animate {
        token_a.animate_move_to(
            token_a_pos.clone(),
            token_b_pos.clone(),
            ANIMATION_TIME_PER_TILE,
        );
        token_b.animate_move_to(
            token_b_pos.clone(),
            token_a_pos.clone(),
            ANIMATION_TIME_PER_TILE,
        );
    }
    tokens.insert(token_b_pos, token_a);
    tokens.insert(token_a_pos, token_b);
}
