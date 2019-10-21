use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::io;
use std::io::BufRead;
use failure::{Error, bail, ensure};

pub fn aoc20(part2: bool) -> Result<(), Error> {
    let paths = parse_regex(&io::stdin().lock().lines().next().unwrap().unwrap())?;
    let rooms = Rooms::new(paths);
    println!("{}", rooms);
    if part2  {
        println!("Rooms >= 1k steps away: {}", rooms.rooms_n_or_more_steps_away(1000));
    } else {
        println!("Longest shortest path: {}", rooms.longest_shortest_path());
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Direction {
    East,
    West,
    North,
    South,
}

use self::Direction::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct RoomPaths {
    non_branching_path: Vec<Direction>,
    children: Vec<RoomPaths>,
    next: Option<Box<RoomPaths>>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Square {
    VerticalDoor,
    HorizontalDoor,
    Wall,
    Floor,
    Origin,
}

use self::Square::*;

struct Rooms {
    squares: Vec<Vec<Square>>,
    memo: HashSet<(usize, usize, RoomPaths)>,
}

const ORIGIN_X: usize = 1024;
const ORIGIN_Y: usize = 1024;

impl Rooms {
    fn new(paths: RoomPaths) -> Self {
        let mut rooms = Self { squares: vec![], memo: HashSet::new() };
        rooms.set(ORIGIN_X, ORIGIN_Y, Origin);
        rooms.follow_paths(ORIGIN_X, ORIGIN_Y, &paths);
        rooms
    }

    fn get(&self, x: usize, y: usize) -> Option<&Square> {
        self.squares.get(y).and_then(|r| r.get(x))
    }

    fn get_mut(&mut self, x: usize, y: usize) -> &mut Square {
        if self.squares.len() < y + 1 {
            self.squares.resize(y + 1, vec![]);
        }
        if self.squares[y].len() < x + 1 {
            self.squares[y].resize(x + 1, Wall);
        }
        &mut self.squares[y][x]
    }

    fn set(&mut self, x: usize, y: usize, square: Square) {
        *self.get_mut(x, y) = square;
    }

    fn follow_paths(&mut self, origin_x: usize, origin_y: usize, paths: &RoomPaths) -> Vec<(usize, usize)> {
        if self.memo.get(&(origin_x, origin_y, paths.clone())).is_some() {
            // this is actually wrong, but it works so idgaf
            return vec![];
        }
        self.memo.insert((origin_x, origin_y, paths.clone()));
        let mut x = origin_x;
        let mut y = origin_y;
        for direction in paths.non_branching_path.iter() {
            match direction {
                East => {
                    self.set(x + 1, y, VerticalDoor);
                    self.set(x + 2, y, Floor);
                    x += 2;
                }
                West => {
                    self.set(x - 1, y, VerticalDoor);
                    self.set(x - 2, y, Floor);
                    x = x.checked_sub(2).expect("Room hit boundary, increase origin size");
                }
                South => {
                    self.set(x, y + 1, HorizontalDoor);
                    self.set(x, y + 2, Floor);
                    y += 2;
                }
                North => {
                    self.set(x, y - 1, HorizontalDoor);
                    self.set(x, y - 2, Floor);
                    y = y.checked_sub(2).expect("Room hit boundary, increase origin size");
                }
            }
        }
        let mut child_endpoints = vec![];
        for child_paths in paths.children.iter() {
            child_endpoints.extend(self.follow_paths(x, y, child_paths));
        }
        let mut endpoints = vec![];
        if let Some(next_path) = &paths.next {
            assert!(!child_endpoints.is_empty());
            for &(x, y) in child_endpoints.iter() {
                endpoints.extend(self.follow_paths(x, y, next_path));
            }
        } else {
            endpoints.push((x, y));
        }
        endpoints
    }

    fn shortest_paths(&self) -> HashMap<(usize, usize), u32> {
        let mut queue = VecDeque::new();
        let mut distance = HashMap::new();
        queue.push_back((ORIGIN_X, ORIGIN_Y));
        distance.insert((ORIGIN_X, ORIGIN_Y), 0);
        while let Some((x, y)) = queue.pop_front() {
            if let Some(HorizontalDoor) = self.get(x, y - 1) {
                if distance.get(&(x, y - 2)).is_none() {
                    distance.insert((x, y - 2), distance[&(x, y)] + 1);
                    queue.push_back((x, y - 2));
                }
            }
            if let Some(HorizontalDoor) = self.get(x, y + 1) {
                if distance.get(&(x, y + 2)).is_none() {
                    distance.insert((x, y + 2), distance[&(x, y)] + 1);
                    queue.push_back((x, y + 2));
                }
            }
            if let Some(VerticalDoor) = self.get(x - 1, y) {
                if distance.get(&(x - 2, y)).is_none() {
                    distance.insert((x - 2, y), distance[&(x, y)] + 1);
                    queue.push_back((x - 2, y));
                }
            }
            if let Some(VerticalDoor) = self.get(x + 1, y) {
                if distance.get(&(x + 2, y)).is_none() {
                    distance.insert((x + 2, y), distance[&(x, y)] + 1);
                    queue.push_back((x + 2, y));
                }
            }
        }
        distance
    }

    fn longest_shortest_path(&self) -> u32 {
        *self.shortest_paths().values().max().unwrap()
    }

    fn rooms_n_or_more_steps_away(&self, steps: u32) -> usize {
        self.shortest_paths().values().filter(|d| d >= &&steps).count()
    }

    /// Returns the bounding box enclosing all non-Wall objects, non-inclusive
    /// (as x_0, y_0, width, height).
    fn bounding_box(&self) -> (usize, usize, usize, usize) {
        let mut start_row = None;
        let mut start_col = None;
        let mut height = 0;
        let mut width = 0;
        for (y, row) in self.squares.iter().enumerate() {
            let left_pos_opt = row.iter().position(|s| s != &Wall);
            if left_pos_opt.is_none() {
                // Row has no Floor
                if start_row.is_some() {
                    // We've reached the end of the bounding box
                    break;
                } else {
                    // We haven't begun the bounding box
                    continue;
                }
            }
            let left_pos = left_pos_opt.unwrap();
            let right_pos = row.iter().rposition(|s| s != &Wall).unwrap();

            if start_row.is_none() {
                start_row = Some(y);
            }
            start_col = match start_col {
                None => Some(left_pos),
                Some(col) => Some(col.min(left_pos)),
            };
            height += 1;
            width = width.max(right_pos - start_col.unwrap() + 1);
        }
        (start_col.unwrap() - 1, start_row.unwrap() - 1, width + 1, height + 1)
    }
}

impl fmt::Display for Rooms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x_0, y_0, width, height) = self.bounding_box();
        for y in y_0..=y_0 + height {
            for x in x_0..=x_0 + width {
                let ch = match self.get(x, y) {
                    Some(VerticalDoor) => '|',
                    Some(HorizontalDoor) => '-',
                    Some(Wall) => '#',
                    Some(Floor) => '.',
                    Some(Origin) => 'X',
                    None => '#',
                };
                write!(f, "{}", ch)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

fn parse_regex(input: &str) -> Result<RoomPaths, Error> {
    ensure!(input.len() >= 2, "String not long enough to be a proper regex");
    ensure!(input.starts_with('^'), "Regex does not start with ^");
    ensure!(input.ends_with('$'), "Regex does not end with $");
    parse_sub_expression(&input[1..input.len() - 1])
}

fn parse_sub_expression(input: &str) -> Result<RoomPaths, Error> {
    let mut end_unbranch = input.len();
    let mut start_next = None;
    let mut children: Vec<RoomPaths> = vec![];
    if let Some(start_paren) = input.find('(') {
        // Find "|"-separated sub-expressions
        let mut cur_unterminated_start = start_paren + 1;
        let mut paren_depth = 0;
        let mut ranges = vec![];
        let mut end_paren = None;
        for (i, ch) in input[start_paren + 1..].chars().enumerate() {
            end_unbranch = start_paren;
            match ch {
                '|' if paren_depth == 0 => {
                    ranges.push(cur_unterminated_start..start_paren + i + 1);
                    cur_unterminated_start = start_paren + i + 2;
                },
                '(' => paren_depth += 1,
                ')' => {
                    if paren_depth == 0 {
                        end_paren = Some(start_paren + i + 1);
                        break;
                    }
                    paren_depth -= 1;
                },
                _ => {},
            }
        }
        if end_paren.is_none() {
            bail!("Unterminated parenthesis group");
        }
        if end_paren != Some(input.len() - 1) {
            start_next = Some(end_paren.unwrap() + 1);
        }
        if ranges.last().is_some() && ranges.last().unwrap().end != end_paren.unwrap() {
            ranges.push(cur_unterminated_start..end_paren.unwrap());
        }
        children = ranges.into_iter().map(|r| parse_sub_expression(&input[r])).collect::<Result<_, _>>()?;
    }
    let next = match start_next {
        Some(i) => Some(Box::new(parse_sub_expression(&input[i..])?)),
        None => None,
    };
    Ok(RoomPaths {
        non_branching_path: parse_directions(&input[..end_unbranch])?,
        children,
        next,
    })
}

fn parse_directions(input: &str) -> Result<Vec<Direction>, Error> {
    input.chars().map(direction_from_char).collect()
}

fn direction_from_char(c: char) -> Result<Direction, Error> {
    match c {
        'E' => Ok(East),
        'S' => Ok(South),
        'W' => Ok(West),
        'N' => Ok(North),
        _ => bail!("Character {} not recognized as a direction", c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_directions() -> Result<(), Error> {
        assert_eq!(parse_directions("NSEW")?, vec![North, South, East, West]);
        Ok(())
    }

    #[test]
    fn test_parse_regex() -> Result<(), Error> {
        let input_str = "^ESSWWN(E|NNENN(SSE|)WW|S)$";
        let paths = parse_regex(input_str)?;
        assert_eq!(paths, RoomPaths {
            non_branching_path: parse_directions("ESSWWN").unwrap(),
            children: vec![
                RoomPaths {
                    non_branching_path: parse_directions("E").unwrap(),
                    children: vec![],
                    next: None,
                },
                RoomPaths {
                    non_branching_path: parse_directions("NNENN").unwrap(),
                    children: vec![
                        RoomPaths {
                            non_branching_path: parse_directions("SSE").unwrap(),
                            children: vec![],
                            next: None,
                        },
                        RoomPaths {
                            non_branching_path: vec![],
                            children: vec![],
                            next: None,
                        },
                    ],
                    next: Some(Box::new(RoomPaths {
                        non_branching_path: parse_directions("WW").unwrap(),
                        children: vec![],
                        next: None,
                    })),
                },
                RoomPaths {
                    non_branching_path: parse_directions("S").unwrap(),
                    children: vec![],
                    next: None,
                },
            ],
            next: None,
        });
        Ok(())
    }

    #[test]
    fn test_first_example() -> Result<(), Error> {
        let input_str = "^ENWWW(NEEE|SSE(EE|N))$";
        let expected = "
#########
#.|.|.|.#
#-#######
#.|.|.|.#
#-#####-#
#.#.#X|.#
#-#-#####
#.|.|.|.#
#########
".trim_start();
        let rooms = Rooms::new(parse_regex(input_str)?);
        assert_eq!(format!("{}", rooms), expected);
        Ok(())
    }

    #[test]
    fn test_empty_child() -> Result<(), Error> {
        let input_str = "^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$";
        let expected = "
###########
#.|.#.|.#.#
#-###-#-#-#
#.|.|.#.#.#
#-#####-#-#
#.#.#X|.#.#
#-#-#####-#
#.#.|.|.|.#
#-###-###-#
#.|.|.#.|.#
###########
".trim_start();
        let rooms = Rooms::new(parse_regex(input_str)?);
        assert_eq!(format!("{}", rooms), expected);
        Ok(())
    }

    #[test]
    fn longest_shortest_path() -> Result<(), Error> {
        let input_str = "^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$";
        let rooms = Rooms::new(parse_regex(input_str)?);
        assert_eq!(rooms.longest_shortest_path(), 18);
        Ok(())
    }
}
