mod core;
mod shaders;
mod token;
mod token_grid;

use crate::core::Position;
use crate::token::{swap_tokens, Modifier, Token, TokenType};
use futures::future::join_all;
use hashbrown::HashMap;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

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
        let texture_futures = TokenType::ALL
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
    let token_textures: Vec<Texture2D> = {
        let texture_futures = TokenType::ALL
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
    for i in 0..10 {
        for j in 0..10 {
            let type_ = TokenType::ALL[(i + j) % 3];
            let tex = token_textures[(i + j) % 3].clone();
            let position = Position::new(i as i32, j as i32);
            let token = Token::new(type_, position.clone(), tex.clone());
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

    let mut hovered_token_pos: Option<Position> = None;
    let mut selected_token_pos: Option<Position> = None;
    let mut dirty_token_positions: Vec<Position> = Vec::new();

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
        if is_mouse_button_pressed(MouseButton::Left) {
            println!("Mouse clicked at {:?} ({:?})", mouse_pos, mouse_position());
        }

        let modifier_to_set = match is_mouse_button_pressed(MouseButton::Left) {
            true => Modifier::Selected,
            false => Modifier::Hover,
        };

        if tokens.contains_key(&mouse_pos) {
            match modifier_to_set {
                Modifier::Hover => {
                    //Unset the old token, if there was one
                    if let Some(token_pos) = hovered_token_pos {
                        if token_pos != mouse_pos {
                            if let Some(token) = tokens.get_mut(&token_pos) {
                                println!("Setting prev old token to None");
                                token.unset_modifier(Modifier::Hover);
                            }
                        }
                    }
                    //Set the new token
                    if let Some(token) = tokens.get_mut(&mouse_pos) {
                        token.set_modifier(Modifier::Hover);
                    }
                    hovered_token_pos = Some(mouse_pos.clone());
                }
                Modifier::Selected => {
                    if let Some(old_token_pos) = selected_token_pos.take() {
                        // Flip previously selected and new one
                        let mut old_token = tokens.remove(&old_token_pos).unwrap();
                        old_token.set_modifier(Modifier::None);
                        if old_token_pos.is_adjacent(&mouse_pos) {
                            let mut new_token = tokens.remove(&mouse_pos).unwrap();
                            swap_tokens(&mut old_token, &mut new_token);
                            tokens.insert(new_token.position.clone(), old_token);
                            tokens.insert(old_token_pos.clone(), new_token);
                            dirty_token_positions.push(old_token_pos);
                            dirty_token_positions.push(mouse_pos.clone());
                        } else {
                            tokens.insert(old_token.position.clone(), old_token);
                        }
                    } else {
                        // Just select the new token
                        if let Some(token) = tokens.get_mut(&mouse_pos) {
                            token.set_modifier(modifier_to_set);
                        }

                        // And unset the hovered position if it's the same
                        if let Some(pos) = &hovered_token_pos {
                            if *pos == mouse_pos {
                                hovered_token_pos = None;
                            }
                        }
                        selected_token_pos = Some(mouse_pos.clone());
                    }
                }
                _ => {}
            }
        } else {
            let mut none = None;
            let preset_store = match modifier_to_set {
                Modifier::Selected => &mut selected_token_pos,
                Modifier::Hover => &mut hovered_token_pos,
                Modifier::None => &mut none,
            };
            //Unset the old token, if there was one
            if let Some(token_pos) = preset_store.take() {
                if let Some(token) = tokens.get_mut(&token_pos) {
                    token.set_modifier(Modifier::None);
                }
            }
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            if let Some(token) = tokens.get_mut(&mouse_pos) {
                println!("Token: {:?}", token);
            }
        }

        for token in tokens.values_mut() {
            token.draw(mat.clone(), &outline_texture);
        }

        let no_active_animations = {
            let mut no_active_animations = true;
            for token in tokens.values_mut() {
                if token.is_animating() {
                    no_active_animations = false;
                    break;
                }
            }
            no_active_animations
        };

        if !dirty_token_positions.is_empty() && no_active_animations {
            // We have dirty tokens! let's check for matches.
            let matched_lines = token_grid::check_for_matches(&tokens, &dirty_token_positions);

            for line in matched_lines {
                println!("Removing tokens in matched group {:?}", line);
                for pos in line {
                    tokens.remove(&pos);
                }
            }

            dirty_token_positions.clear();
        }

        next_frame().await
    }
}
