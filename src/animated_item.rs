use crate::core::{Position, TextureAtlas};
use macroquad::color::WHITE;
use macroquad::prelude::draw_texture;

#[derive(Clone, Debug)]
pub struct AnimatedItem {
    i: usize,
    texture_atlas: TextureAtlas,
    one_shot: bool,
    animation_scheme: AnimationScheme,
    playing: PlayingState,
}

impl AnimatedItem {
    pub fn new(
        texture_atlas: TextureAtlas,
        one_shot: bool,
        animation_scheme: AnimationScheme,
    ) -> Self {
        Self {
            i: 0,
            texture_atlas,
            one_shot,
            animation_scheme,
            playing: PlayingState::Stopped,
        }
    }

    pub fn draw(&self, position: &Position) {
        let texture = self.texture_atlas.get(self.i);
        let (x, y) = position.to_world();
        draw_texture(texture, x, y, WHITE)
    }

    fn incr(&mut self) {
        self.i += 1;
        if self.i >= self.texture_atlas.size() {
            self.i = 0;
            if self.one_shot {
                self.playing = PlayingState::Stopped;
            }
        }
    }

    pub fn update(&mut self) {
        if let PlayingState::Playing(start_time) = self.playing {
            let time_to_next_tick = match self.animation_scheme {
                AnimationScheme::TimePerFrame(time_per_frame) => time_per_frame,
                AnimationScheme::TotalTime(total_time) => {
                    total_time / self.texture_atlas.size() as f64
                }
            };
            if macroquad::time::get_time() - start_time > time_to_next_tick {
                self.playing = PlayingState::Playing(macroquad::time::get_time());
                self.incr();
            }
        }
    }

    pub fn start(&mut self) {
        println!("Starting {:?}", self);
        self.playing = PlayingState::Playing(macroquad::time::get_time());
    }
    pub fn stop(&mut self) {
        self.playing = PlayingState::Stopped;
    }
    pub fn pause(&mut self) {
        if let PlayingState::Playing(start_time) = self.playing {
            self.playing = PlayingState::Paused(start_time);
        }
    }
    pub fn resume(&mut self) {
        match self.playing {
            PlayingState::Paused(start_time) => {
                self.playing = PlayingState::Playing(start_time);
            }
            PlayingState::Stopped => {
                self.playing = PlayingState::Playing(macroquad::time::get_time());
            }
            _ => {}
        }
    }
    pub fn is_playing(&self) -> bool {
        match self.playing {
            PlayingState::Playing(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AnimationScheme {
    TimePerFrame(f64),
    TotalTime(f64),
}

#[derive(Clone, Debug)]
enum PlayingState {
    Playing(f64),
    Paused(f64),
    Stopped,
}
