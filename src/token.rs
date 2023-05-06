use crate::core::{AnimationPosition, Position};
use macroquad::color::WHITE;
use macroquad::material::Material;
use macroquad::math::vec4;
use macroquad::prelude::{draw_texture, gl_use_default_material, gl_use_material, Texture2D};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
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
    pub position: Position,
    animation_position: Option<AnimationPosition>,
    modifier: Modifier,
    texture: Texture2D,
    pub dirty: bool,
}

impl Token {
    pub(crate) fn new(type_: TokenType, position: Position, texture: Texture2D) -> Token {
        Self {
            type_,
            position,
            animation_position: None,
            modifier: Modifier::None,
            texture,
            dirty: false,
        }
    }

    pub fn draw(&mut self, shader_material: Material, outline_texture: &Texture2D) {
        let (x, y) = match self.animation_position {
            Some(ref mut animation_position) => {
                if animation_position.is_done() {
                    self.position = animation_position.end.clone();
                    self.animation_position = None;
                    self.position.to_world()
                } else {
                    animation_position.get()
                }
            }
            None => self.position.to_world(),
        };
        match self.modifier {
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

    pub fn is_at(&self, position: &Position) -> bool {
        self.position == *position
    }

    pub fn set_modifier(&mut self, modifier: Modifier) {
        if self.modifier.valid_transition_to(modifier) {
            println!(
                "Setting modifier for {:?} to {:?} (was {:?})",
                self.position, modifier, self.modifier
            );
            self.modifier = modifier;
        }
    }
    pub fn unset_modifier(&mut self, modifier: Modifier) {
        if self.modifier == modifier {
            println!(
                "Unsetting modifier for {:?} (was {:?})",
                self.position, self.modifier
            );
            self.modifier = Modifier::None;
        }
    }

    pub fn move_to(&mut self, position: Position, animation_time: Option<f64>) {
        match animation_time {
            Some(time) => {
                self.animation_position = Some(AnimationPosition::new_start(
                    self.position.clone(),
                    position,
                    time,
                ));
            }
            None => {
                self.position = position;
                self.animation_position = None;
            }
        }
        self.dirty = true;
    }

    pub fn is_animating(&self) -> bool {
        match &self.animation_position {
            Some(anim_pos) => !anim_pos.is_done(),
            None => false,
        }
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

pub fn swap_tokens(tok_a: &mut Token, tok_b: &mut Token) {
    let move_a_to = tok_b.position.clone();
    let move_b_to = tok_a.position.clone();
    tok_a.move_to(move_a_to, Some(0.5));
    tok_b.move_to(move_b_to, Some(0.5));
    tok_a.dirty = true;
    tok_b.dirty = true;
}
