mod animated_item;
mod core;
mod effect_player;
mod game_state;
mod phases;
mod shaders;
mod token;
mod token_grid;

use crate::animated_item::{AnimatedItem, AnimationScheme};
use crate::core::{Position, TextureAtlas};
use crate::effect_player::EffectPlayer;
use crate::game_state::GameState;
use crate::token::{is_valid_swap, swap_tokens, Modifier, Token, TokenType};
use crate::token_grid::check_for_matches;
use futures::future::join_all;
use hashbrown::HashMap;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use phases::Phase;

fn window_conf() -> Conf {
    Conf {
        window_title: "MatchUp!".to_owned(),
        window_width: 640,
        window_height: 640,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let do_shader_toy = false;
    if do_shader_toy {
        shader_toy().await;
    } else {
        game().await;
    }
}

async fn shader_toy() {
    let token_textures: Vec<Texture2D> = {
        let texture_futures = TokenType::ALL_REGULAR
            .iter()
            .map(|t| load_texture(t.to_sprite_name()))
            .collect::<Vec<_>>();
        join_all(texture_futures)
            .await
            .into_iter()
            .map(|t| {
                let t = t.unwrap();
                t.set_filter(FilterMode::Nearest);
                t
            })
            .collect()
    };

    let mut materials = [
        Some(("OUTLINE", shaders::outline_material())),
        Some(("GLOW", shaders::glow_material())),
        Some(("FILE", shaders::material_from_file())),
        None,
    ];
    let mut mat_index = 0;
    let file_shader_index = materials
        .iter()
        .position(|m| match m {
            Some(("FILE", _)) => true,
            _ => false,
        })
        .unwrap();

    for m in materials.iter() {
        if let Some((name, mat)) = m {
            mat.set_uniform("test_color", vec4(1.5, 1.5, 1.5, 1.));
        }
    }

    let mut camera =
        Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen_width(), screen_height()));
    camera.zoom = Vec2::new(0.0061250003, -0.0061250003);

    let x = 300.0;
    loop {
        clear_background(GRAY);

        let mut y = 250.0;

        set_camera(&camera);

        draw_text_ex(
            get_material_name(&materials, mat_index),
            200.0,
            200.0,
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
        );

        set_material(&materials, mat_index);
        for texture in token_textures.iter() {
            draw_texture(texture.clone(), x, y, WHITE);
            y += 40.0;
        }

        if is_key_pressed(KeyCode::Space) {
            mat_index = (mat_index + 1) % materials.len();
            println!(
                "Setting material to {}",
                get_material_name(&materials, mat_index)
            );
        }

        if is_key_pressed(KeyCode::R) {
            println!("Reloading file shader");
            let new_mat = shaders::material_from_file();
            materials[file_shader_index] = Some(("FILE", new_mat));
        }

        next_frame().await
    }
}

fn get_material_name<'a>(materials: &'a [Option<(&str, Material)>], idx: usize) -> &'a str {
    if let Some((name, _)) = materials[idx] {
        name
    } else {
        "DEFAULT"
    }
}

fn set_material(materials: &[Option<(&str, Material)>], idx: usize) {
    if let Some((name, mat)) = materials[idx] {
        gl_use_material(mat);
    } else {
        gl_use_default_material();
    }
}

async fn game() {
    let mat = shaders::glow_material();
    let outline_texture = load_texture("res/outline.png").await.unwrap();
    let cross_texture = load_texture("res/cross.png").await.unwrap();
    let mut cross_positions: Vec<Position> = Vec::new();
    let token_textures: Vec<Texture2D> = {
        let texture_futures = TokenType::ALL_REGULAR
            .iter()
            .map(|t| load_texture(t.to_sprite_name()))
            .collect::<Vec<_>>();
        join_all(texture_futures)
            .await
            .into_iter()
            .map(|t| {
                let t = t.unwrap();
                t.set_filter(FilterMode::Nearest);
                t
            })
            .collect()
    };

    let mut tokens = HashMap::new();
    let mut token_texture_map = HashMap::new();
    for (i, t) in TokenType::ALL_REGULAR.iter().enumerate() {
        token_texture_map.insert(t.clone(), token_textures[i].clone());
    }

    let mut effect_player = EffectPlayer::new().await;
    effect_player.audio_effect_volume = 0.1;

    let grid_size = 10;
    for i in 0..grid_size {
        for j in 0..grid_size {
            let mut modulo = TokenType::ALL_REGULAR.len();
            if i % 3 == 0 {
                modulo -= 1
            };
            let idx = (i + j) % modulo;
            let type_ = TokenType::ALL_REGULAR[idx];
            let tex = token_textures[idx].clone();
            let position = Position::new(i as i32, j as i32);
            let token = Token::new(type_, tex.clone());
            tokens.insert(position, token);
        }
    }

    let bg_colour = Color::from_rgba(75, 106, 115, 255);

    let mut camera =
        Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen_width(), screen_height()));
    let zoom_ratio = camera.zoom.x / camera.zoom.y;
    println!("Initial zoom: {:?}", camera.zoom);

    let prebaked_zoom = Vec2::new(0.0061250003, -0.0061250003);
    let prebaked_offset = Vec2::new(160.0, 160.0);
    camera.zoom = prebaked_zoom;
    camera.target = prebaked_offset;

    let mut game_state = GameState::new(tokens, grid_size, effect_player);

    loop {
        clear_background(bg_colour);

        let mut set_zoom = false;
        if is_key_pressed(KeyCode::Up) {
            camera.zoom.x += 0.001;
            set_zoom = true;
        } else if is_key_pressed(KeyCode::Down) {
            camera.zoom.x -= 0.001;
            set_zoom = true;
        }
        if set_zoom {
            if camera.zoom.x < 0.001 {
                camera.zoom.x = 0.001;
            }
            camera.zoom.y = camera.zoom.x / zoom_ratio;
            println!("Set zoom to {:?}", camera.zoom);
        }

        let mut camera_move = Vec2::new(0.0, 0.0);
        for (key, diff) in [
            (KeyCode::W, (0.0, -1.0)),
            (KeyCode::A, (-1.0, 0.0)),
            (KeyCode::S, (0.0, 1.0)),
            (KeyCode::D, (1.0, 0.0)),
        ] {
            if is_key_down(key) {
                camera_move.x += diff.0;
                camera_move.y += diff.1;
            }
        }
        if camera_move != Vec2::ZERO {
            let move_modifier = 1.0;
            camera.target += camera_move * move_modifier;
        }

        if is_mouse_button_pressed(MouseButton::Middle) {
            println!("Zoom: {:?}", camera.zoom);
            println!("Target: {:?}", camera.target);
        }

        set_camera(&camera);

        let mouse_pos = {
            let (mouse_x, mouse_y) = mouse_position();
            let mouse = camera.screen_to_world(Vec2::new(mouse_x, mouse_y));
            Position::from_world_vec2(mouse)
        };

        match game_state.phase {
            Phase::TakingInput => phases::taking_input_phase(mouse_pos.clone(), &mut game_state),
            Phase::MovedAndAnimating(ref moved_positions) => {
                // Having to clone this list to make borrow checker happy (i.e. can't borrow the
                // vec inside the phase variant from the game state and pass it in mutably).
                // sad times
                let moved_positions = moved_positions.clone();
                phases::post_token_swap_phase(&moved_positions, &mut game_state)
            }
            Phase::GravityRefill => {
                phases::gravity_refill_phase(&mut game_state, &token_texture_map)
            }
            Phase::CheckWholeGrid => {
                phases::check_whole_grid_phase(&mut game_state, &mut cross_positions)
            }
            Phase::Animating(ref next_phase) => {
                let next_phase = next_phase.clone();
                phases::animating_phase(&mut game_state, next_phase);
            }
        }

        game_state.effect_player.update();
        game_state.effect_player.draw();

        // Draw
        for (pos, token) in &mut game_state.tokens {
            token.update();

            let is_selected_already = game_state.selected_token_pos.as_ref() == Some(pos);
            let modifier = if is_selected_already {
                Modifier::Selected
            } else if pos == &mouse_pos {
                Modifier::Hover
            } else {
                Modifier::None
            };

            token.draw(pos, &modifier, mat.clone(), &outline_texture);
        }

        draw_text(
            &format!("Phase: {:?}", game_state.phase),
            10.0,
            10.0,
            10.0,
            WHITE,
        );

        for cross_pos in cross_positions.iter() {
            let (x, y) = cross_pos.to_world();
            draw_texture(cross_texture, x, y, WHITE);
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            if let Some(token) = game_state.tokens.get_mut(&mouse_pos) {
                println!("Token: {:?}", token);
            }
        }

        next_frame().await
    }
}
