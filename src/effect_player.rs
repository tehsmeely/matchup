use crate::animated_item::{AnimatedItem, AnimationScheme};
use crate::core::{Position, TextureAtlas};
use crate::token;
use hashbrown::HashMap;
use macroquad::audio::{load_sound, play_sound, PlaySoundParams, Sound};
use macroquad::time::get_time;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone)]
struct SoundEffect {
    sound: Sound,
    duration: f64,
    volume: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum EffectKind {
    Explosion,
}

#[derive(Debug)]
pub struct EffectPlayer {
    active_effects: HashMap<Position, AnimatedItem>,
    active_sounds: HashMap<EffectKind, f64>,
    effect_store: HashMap<EffectKind, AnimatedItem>,
    audio_store: HashMap<EffectKind, SoundEffect>,
    pub audio_effect_volume: f32,
}

impl EffectPlayer {
    pub async fn new() -> Self {
        Self {
            active_effects: HashMap::new(),
            active_sounds: HashMap::new(),
            effect_store: initialise_effect_store().await,
            audio_store: initialise_audio_store().await,
            audio_effect_volume: 1.0,
        }
    }

    pub fn spawn_effect(&mut self, pos: Position, kind: EffectKind) {
        let mut effect = self.effect_store.get(&kind).unwrap().clone();
        effect.start();
        self.active_effects.insert(pos, effect);
        if let Some(sound) = self.audio_store.get(&kind) {
            let can_play = if let Some(already_playing_at) = self.active_sounds.get(&kind) {
                let duration_played = get_time() - already_playing_at;
                let pct_played = duration_played / sound.duration;
                pct_played > 0.8
            } else {
                true
            };
            if can_play {
                let volume = (sound.volume * self.audio_effect_volume) / 2.0;
                play_sound(
                    sound.sound,
                    PlaySoundParams {
                        looped: false,
                        volume,
                    },
                );
                self.active_sounds.insert(kind, get_time());
            }
        }
    }

    pub fn update(&mut self) {
        self.active_effects.retain(|_, effect| {
            effect.update();
            effect.is_playing()
        });
    }

    pub fn draw(&self) {
        for (pos, effect) in self.active_effects.iter() {
            effect.draw(pos);
        }
    }
}

impl EffectKind {
    async fn to_sound(&self) -> Option<SoundEffect> {
        match self {
            Self::Explosion => {
                let sound = load_sound("res/audio/explosion.wav").await.unwrap();
                Some(SoundEffect {
                    sound,
                    duration: 1f64,
                    volume: 0.5,
                })
            }
        }
    }
}

async fn initialise_audio_store() -> HashMap<EffectKind, SoundEffect> {
    let mut store = HashMap::new();
    for effect in EffectKind::iter() {
        if let Some(sound) = effect.to_sound().await {
            store.insert(effect, sound);
        }
    }
    store
}

async fn initialise_effect_store() -> HashMap<EffectKind, AnimatedItem> {
    let mut store = HashMap::new();

    let explosion = {
        let atlas = TextureAtlas::new("res/explosion.png", (32f32, 32f32), 1, 6).await;
        AnimatedItem::new(
            atlas,
            true,
            AnimationScheme::TotalTime(token::ANIMATION_TIME_PER_TILE - 0.05),
        )
    };
    store.insert(EffectKind::Explosion, explosion);
    store
}
