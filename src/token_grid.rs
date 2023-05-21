use crate::core::MatchKind;
use crate::token::TokenType;
use crate::{Position, Token};
use hashbrown::{HashMap, HashSet};
use std::ops::Range;

pub fn check_for_matches(
    tokens: &HashMap<Position, Token>,
    dirty_positions: &[Position],
) -> Vec<Vec<Position>> {
    // Actually need to do this individually for each token in dirty_positions because they might have different types ... think about this

    let mut matched_lines = Vec::new();
    // >> Wrap below in a function that takes a type and an initial position
    // Collect all contiguous tokens of this type
    for (start_position) in dirty_positions {
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
    println!("Matched lines: {:?}", matched_lines);
    matched_lines
}

fn check_contiguous_area(
    tokens: &HashMap<Position, Token>,
    start_position: &Position,
    token_type: &TokenType,
) -> Vec<Position> {
    let mut unchecked_positions = vec![start_position.clone()];
    let mut checked_positions = HashSet::new();

    while let Some(position) = unchecked_positions.pop() {
        checked_positions.insert(position.clone());

        for neighbour in position.neighbours() {
            if !checked_positions.contains(&neighbour) {
                if let Some(token) = tokens.get(&neighbour) {
                    if token.type_ == *token_type {
                        unchecked_positions.push(neighbour.clone());
                    }
                }
            }
        }
    }
    checked_positions.into_iter().collect()
}

/// A struct that represents a direction in which we may want to check for continuous positions
struct ByDirection {
    // bounds are (] - i.e. start is inclusive, end is exclusive
    bounds: (i32, i32),
    get: fn(&Position) -> i32,
    set: fn(&mut Position, i32),
}

impl ByDirection {
    fn to_range(&self) -> Range<i32> {
        Range {
            start: self.bounds.0,
            end: self.bounds.1,
        }
    }
}

fn find_lines(
    matched_lines: &mut Vec<Vec<Position>>,
    check_by_direction: &ByDirection,
    step_by_direction: &ByDirection,
    area: &[Position],
) {
    //We step in the "step_by_direction" direction, and check in the "check_by_direction" direction, so we can flip them in future
    // One can imagine y is "step" and x is "check". we go thought y=0 ... y=10, and for each y we go through x=0 ... x=10
    // only for each x do we check for continuous lines, breaking on gaps or when we reach the end of the range for x
    let mut position = Position { x: 0, y: 0 };
    for step_unit in step_by_direction.to_range() {
        let mut line = Vec::new();
        (step_by_direction.set)(&mut position, step_unit);
        for check_unit in check_by_direction.to_range() {
            (check_by_direction.set)(&mut position, check_unit);
            if area.contains(&position) {
                line.push(position.clone());
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

// TODO: Consider passing around &[Position] instead of Vec<Position> to avoid copying
fn lines_intersect(line1: Vec<Position>, line2: Vec<Position>) {}

fn make_by_direction(
    area: &[Position],
    get: fn(&Position) -> i32,
    set: fn(&mut Position, i32),
) -> ByDirection {
    let min = area.iter().map(get).min().unwrap();
    let max = area.iter().map(get).max().unwrap();
    ByDirection {
        bounds: (min, max + 1),
        get,
        set,
    }
}

fn check_contiguous_area_for_linearity(area: &[Position]) -> Vec<(Vec<Position>, MatchKind)> {
    // check for lines in x and y
    // check for intersection

    /*
    let (min_x, max_x, min_y, max_y) =
        area.iter().fold((i32::MAX, 0, i32::MAX, 0),
        |(min_x, max_x, min_y, max_y), pos| {
        (min_x.min(pos.x), max_x.max(pos.x), min_y.min(pos.y), max_y.max(pos.y))
        });

    let by_direction_x = ByDirection {
        bounds: (min_x, max_x),
        get: Position::get_x,
        set: Position::set_x,
    };
    let by_direction_y = ByDirection {
        bounds: (min_y, max_y),
        get: Position::get_y,
        set: Position::set_y,
    };
     */
    let by_direction_x = make_by_direction(area, Position::get_x, Position::set_x);
    let by_direction_y = make_by_direction(area, Position::get_y, Position::set_y);

    let mut matched_lines = Vec::new();
    find_lines(&mut matched_lines, &by_direction_x, &by_direction_y, area);
    find_lines(&mut matched_lines, &by_direction_y, &by_direction_x, area);

    println!("Input: {:?}", area);
    println!("Matched lines: {:?}", matched_lines);
    println!("----");

    // TODO: Check for intersections, we are lying here
    matched_lines
        .into_iter()
        .map(|line| (line, MatchKind::Three))
        .collect()
}

pub fn check_entire_grid(tokens: &HashMap<Position, Token>) -> Vec<(Vec<Position>, MatchKind)> {
    let mut checked_positions = HashSet::new();
    let mut contiguous_areas = Vec::new();

    for (position, token) in tokens {
        if !checked_positions.contains(position) {
            checked_positions.insert(position.clone());
            let contiguous_area = check_contiguous_area(tokens, position, &token.type_);
            contiguous_areas.push(contiguous_area);
        }
    }

    let mut lines_with_match_kind = Vec::new();
    for area in contiguous_areas {
        if area.len() >= 3 {
            // Don't bother checking unless it's an area of 3 or more, we can't get a three-in-a-row with less than three!
            lines_with_match_kind.extend(check_contiguous_area_for_linearity(&area));
        }
    }
    lines_with_match_kind
}

mod test {
    use super::*;
    use crate::core::Position;

    #[test]
    fn test_areas() {
        let input = vec![
            Position { x: 6, y: 4 },
            Position { x: 7, y: 4 },
            Position { x: 8, y: 4 },
            Position { x: 9, y: 4 },
        ];

        let by_direction_x = make_by_direction(&input, Position::get_x, Position::set_x);
        assert_eq!(by_direction_x.to_range(), 6..10);
        let by_direction_y = make_by_direction(&input, Position::get_y, Position::set_y);
        assert_eq!(by_direction_y.to_range(), 4..5);
        let mut matched_lines = Vec::new();
        find_lines(&mut matched_lines, &by_direction_y, &by_direction_x, &input);
        assert_eq!(matched_lines, Vec::<Vec<Position>>::new());
        find_lines(&mut matched_lines, &by_direction_x, &by_direction_y, &input);
        assert_eq!(
            matched_lines,
            vec![vec![
                Position { x: 6, y: 4 },
                Position { x: 7, y: 4 },
                Position { x: 8, y: 4 },
                Position { x: 9, y: 4 }
            ]]
        );
    }
}
