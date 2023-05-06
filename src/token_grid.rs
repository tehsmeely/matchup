use crate::{Position, Token};
use hashbrown::{HashMap, HashSet};

pub fn check_for_matches(
    tokens: &HashMap<Position, Token>,
    dirty_positions: &Vec<Position>,
) -> Vec<Vec<Position>> {
    // Actually need to do this individually for each token in dirty_positions because they might have different types ... think about this

    let mut matched_lines = Vec::new();
    // >> Wrap below in a function that takes a type and an initial position
    // Collect all contiguous tokens of this type
    for start_position in dirty_positions {
        let mut unchecked_positions = vec![start_position.clone()];
        let mut checked_positions = HashSet::new();
        println!("Checking for matches starting at {:?}", start_position);
        let token_type = tokens.get(start_position).unwrap().type_.clone();
        println!("The starting token is type {:?}", token_type);

        while let Some(position) = unchecked_positions.pop() {
            checked_positions.insert(position.clone());

            for neighbour in position.neighbours() {
                if !checked_positions.contains(&neighbour) {
                    if let Some(token) = tokens.get(&neighbour) {
                        if token.type_ == token_type {
                            unchecked_positions.push(neighbour.clone());
                        }
                    }
                }
            }
        }

        // try and match lines
        let ys = checked_positions.iter().map(|pos| pos.y);
        let min_y = ys.clone().min().unwrap();
        let max_y = ys.max().unwrap();

        let xs = checked_positions.iter().map(|pos| pos.x);
        let min_x = xs.clone().min().unwrap();
        let max_x = xs.max().unwrap();

        for y in min_y..=max_y {
            let mut line = Vec::new();
            for x in min_x..=max_x {
                let pos = Position { x, y };
                if checked_positions.contains(&pos) {
                    line.push(pos);
                } else {
                    if line.len() >= 3 {
                        matched_lines.push(line.clone());
                    }
                    line.clear();
                }
            }
            if line.len() >= 3 {
                matched_lines.push(line.clone());
            }
        }

        for x in min_x..=max_x {
            let mut line = Vec::new();
            for y in min_y..=max_y {
                let pos = Position { x, y };
                if checked_positions.contains(&pos) {
                    line.push(pos);
                } else {
                    if line.len() >= 3 {
                        matched_lines.push(line.clone());
                    }
                    line.clear();
                }
            }
            if line.len() >= 3 {
                matched_lines.push(line.clone());
            }
        }
    }
    matched_lines
}
