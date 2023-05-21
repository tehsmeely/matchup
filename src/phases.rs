use crate::animated_item::AnimatedItem;
use crate::effect_player::EffectKind;
use crate::game_state::GameState;
use crate::token::{Token, TokenType};
use crate::{check_for_matches, game, is_valid_swap, swap_tokens, Position};
use hashbrown::HashMap;
use macroquad::input::{is_mouse_button_pressed, mouse_position, MouseButton};
use macroquad::texture::Texture2D;
use rand::random;
use std::collections::vec_deque::VecDeque;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Phase {
    // User Input is displayed and taken
    TakingInput,
    // Tokens were moved and we need to calculate
    MovedAndAnimating(Vec<Position>),
    // Tokens may be missing and we need to refill from above
    GravityRefill,
    // Grid has changed due to gravity, check everything
    CheckWholeGrid,
    // Generic animation phase, will move onto the next phase when all animations are complete
    Animating(Rc<Phase>),
}

impl Default for Phase {
    fn default() -> Self {
        Self::TakingInput
    }
}

pub fn animating_phase(game_state: &mut GameState, next: Rc<Phase>) {
    // Transition to the next phase when where are no animating tokens
    let no_active_animations = {
        let mut no_active_animations = true;
        for token in game_state.tokens.values() {
            if token.is_animating() {
                no_active_animations = false;
                break;
            }
        }
        no_active_animations
    };

    if no_active_animations {
        game_state.phase = (*next).clone()
    }
}

pub fn taking_input_phase(mouse_pos: Position, game_state: &mut GameState) {
    if is_mouse_button_pressed(MouseButton::Left) {
        println!(
            "Mouse left clicked at {:?} ({:?})",
            mouse_pos,
            mouse_position()
        );

        if game_state.tokens.contains_key(&mouse_pos) {
            if let Some(ref already_selected_pos) = game_state.selected_token_pos.take() {
                // TODO: Handle swap
                if is_valid_swap(&mut game_state.tokens, already_selected_pos, &mouse_pos) {
                    println!("Swapping tokens");
                    let prev_token_pos = already_selected_pos.clone();
                    let new_token_pos = mouse_pos.clone();
                    swap_tokens(
                        &mut game_state.tokens,
                        prev_token_pos.clone(),
                        new_token_pos.clone(),
                        true,
                    );

                    // Transition phase
                    game_state.phase = Phase::Animating(Rc::new(Phase::MovedAndAnimating(vec![
                        prev_token_pos,
                        new_token_pos,
                    ])));
                }
            } else {
                game_state.selected_token_pos = Some(mouse_pos.clone());
            }
        }
    }
}

pub fn post_token_swap_phase(moved_positions: &Vec<Position>, game_state: &mut GameState) {
    let matched_lines = check_for_matches(&mut game_state.tokens, moved_positions);
    for line in matched_lines {
        println!("Removing tokens in matched group {:?}", line);
        for pos in line {
            game_state.tokens.remove(&pos);
            game_state
                .effect_player
                .spawn_effect(pos, EffectKind::Explosion);
        }
    }

    game_state.phase = Phase::GravityRefill;
}

pub fn gravity_refill_phase(
    game_state: &mut GameState,
    token_textures: &HashMap<TokenType, Texture2D>,
) {
    println!("Gravity Refill Phase");
    //iterate from bottom to top, if there is a gap, move the token above it down to replace it
    let mut gaps_by_x: HashMap<i32, usize> = HashMap::new();
    /*
    for y in (0..game_state.grid_size).rev() {
        for x in 0..game_state.grid_size {
            let pos = Position::new(x as i32, y as i32);
            if !game_state.tokens.contains_key(&pos) {
                //there is a gap here, move the token above it down
                for y_above in (0..y).rev() {
                    let pos_above = Position::new(x as i32, y_above as i32);
                    if let Some(mut token) = game_state.tokens.remove(&pos_above) {
                        let animation_time =
                            (y - y_above) as f64 * crate::token::ANIMATION_TIME_PER_TILE;
                        token.animate_move_to(pos_above.clone(), pos.clone(), animation_time);
                        game_state.tokens.insert(pos, token);
                        break;
                    }
                }

                {
                    let count = gaps_by_x.entry(x).or_insert(0);
                    *count += 1;
                }
            }
        }
    }
    */

    for x in 0i32..(game_state.grid_size as i32) {
        let mut known_gaps = VecDeque::new();
        for y in (0i32..game_state.grid_size as i32).rev() {
            let pos = Position::new(x as i32, y);
            if !game_state.tokens.contains_key(&pos) {
                known_gaps.push_back(pos);
            } else {
                //There is a token here, if we have any known gaps, move this token down to fill it
                // and don't remember to treat this as a gap to fill now it's moved
                if let Some(gap_pos) = known_gaps.pop_front() {
                    //move this token down,
                    println!("Found a gap at {:?} to fill with {:?}", gap_pos, pos);
                    if let Some(mut token) = game_state.tokens.remove(&pos) {
                        let dy = gap_pos.y - y;
                        let animation_time = dy as f64 * crate::token::ANIMATION_TIME_PER_TILE;
                        token.animate_move_to(pos.clone(), gap_pos.clone(), animation_time);
                        game_state.tokens.insert(gap_pos.clone(), token);
                    }
                    known_gaps.push_back(pos);
                }
            }
        }
        // record unfilled gaps, we'll need to spawn that many tokens
        gaps_by_x.insert(x, known_gaps.len());
    }

    // We we moved tokens down, we need to spawn new ones above
    for (x, gap_count) in gaps_by_x.iter() {
        let gap_count = *gap_count as i32;
        if gap_count > 0 {
            println!("Found {} gaps at x={}", gap_count, x);
            let r = 1..=gap_count;
            println!("r={:?}", r);
            for i in r {
                // We want to move from, say, 1 above to (gap_count - 1) below
                // Let's say we have a gap len of 2, this means we need new tokens at y = 0 and y = 1
                // so that's one:   y=-1 -> y=1,  y=-2 -> y=0
                // i.e. for [i = 1, i = 2],    y = -i -> 2-i = -1 -> 2-1 = -1 -> 1
                let pos_above = Position::new(*x as i32, -i);
                let pos_below = Position::new(*x as i32, gap_count - i);
                let new_token_type: TokenType = random();
                let new_token_texture = token_textures.get(&new_token_type).unwrap().clone();
                let mut new_token = Token::new(new_token_type, new_token_texture);
                new_token.animate_move_to(
                    pos_above.clone(),
                    pos_below.clone(),
                    crate::token::ANIMATION_TIME_PER_TILE * gap_count as f64,
                );
                println!(
                    "Spawning new token, moving from {:?} to {:?}",
                    pos_above, pos_below
                );
                game_state.tokens.insert(pos_below, new_token);
            }
        }
    }
    game_state.phase = Phase::Animating(Rc::new(Phase::CheckWholeGrid));
}

pub fn check_whole_grid_phase(game_state: &mut GameState, _cross_positions: &mut Vec<Position>) {
    let lines_with_match_kind = crate::token_grid::check_entire_grid(&game_state.tokens);

    if lines_with_match_kind.is_empty() {
        game_state.phase = Phase::TakingInput;
    } else {
        for (line, _match_kind) in lines_with_match_kind {
            println!("Removing tokens in matched group {:?}", line);
            for pos in line {
                game_state.tokens.remove(&pos);
                game_state
                    .effect_player
                    .spawn_effect(pos, EffectKind::Explosion);
            }
        }
        game_state.phase = Phase::GravityRefill;
    }
}
