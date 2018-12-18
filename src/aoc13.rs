use std::io;
use std::io::BufRead;
use failure::{Error, bail};
use crate::aoc6::Coord;

pub fn aoc13(part2: bool) -> Result<(), Error> {
    let mut tracks = parse_tracks(&mut io::stdin().lock())?;
    if part2 {
        println!("Last minecart: {:?}", tracks.find_last_minecart());
    } else {
        println!("First collision: {:?}", tracks.advance_till_crash());
    }
    Ok(())
}

#[derive(PartialEq, Debug)]
enum TrackContents {
    Empty,
    CurveLeft,
    CurveRight,
    Vertical,
    Horizontal,
    Intersection,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Direction {
    LeftTurn,
    Straight,
    RightTurn,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Orientation {
    Up,
    Down,
    Left,
    Right,
}

use self::TrackContents::*;
use self::Direction::*;
use self::Orientation::*;

#[derive(PartialEq, Debug)]
struct Minecart {
    position: Coord,
    next_direction: Direction,
    orientation: Orientation,
}

impl Minecart {
    fn advance(&mut self, grid: &Vec<Vec<TrackContents>>) {
        match self.orientation {
            Up    => self.position.y -= 1,
            Down  => self.position.y += 1,
            Left  => self.position.x -= 1,
            Right => self.position.x += 1,
        }
        let spot = &grid[self.position.y as usize][self.position.x as usize];
        let turn = match spot {
            Intersection => {
                let turn = self.next_direction.clone();
                self.change_next_direction();
                turn
            },
            CurveLeft => match &self.orientation {
                Up => RightTurn,
                Left => LeftTurn,
                Down => RightTurn,
                Right => LeftTurn,
            }
            CurveRight => match &self.orientation {
                Up => LeftTurn,
                Left => RightTurn,
                Down => LeftTurn,
                Right => RightTurn,
            }
            Empty => panic!("Minecart went off the rails"),
            _ => Straight,
        };
        self.turn(turn);
    }

    fn change_next_direction(&mut self) {
        self.next_direction = match self.next_direction {
            LeftTurn  => Straight,
            Straight  => RightTurn,
            RightTurn => LeftTurn,
        }
    }

    fn turn(&mut self, direction: Direction) {
        self.orientation = match (self.orientation, direction) {
            (Up, LeftTurn) => Left,
            (Up, RightTurn) => Right,
            (Left, LeftTurn) => Down,
            (Left, RightTurn) => Up,
            (Down, LeftTurn) => Right,
            (Down, RightTurn) => Left,
            (Right, LeftTurn) => Up,
            (Right, RightTurn) => Down,
            (o, Straight) => o,
        }
    }
}

#[derive(PartialEq, Debug)]
struct Collision(Coord);

struct Tracks {
    minecarts: Vec<Minecart>,
    contents: Vec<Vec<TrackContents>>,
}

impl Tracks {
    fn advance(&mut self) -> Result<(), Collision> {
        let mut first_collision = None;
        self.minecarts.sort_by_key(|m| (m.position.y, m.position.x));
        let mut i = 0;
        while i < self.minecarts.len() {
            self.minecarts[i].advance(&self.contents);
            if let Some(collision) = self.find_collision() {
                let rindex = self.minecarts.iter().rposition(|m| m.position == collision.0).unwrap();
                self.minecarts.retain(|m| m.position != collision.0);
                if first_collision.is_none() {
                    first_collision = Some(collision);
                }
                // Hacky
                if rindex == i {
                    i = i.saturating_sub(2);
                } else {
                    i = i.saturating_sub(1);
                }
            }
            i += 1;
        }
        if let Some(collision) = first_collision {
            return Err(collision);
        }
        Ok(())
    }

    fn find_collision(&self) -> Option<Collision> {
        for (i, minecart1) in self.minecarts.iter().enumerate() {
            for minecart2 in self.minecarts[i + 1..].iter() {
                if minecart1.position == minecart2.position {
                    return Some(Collision(minecart1.position));
                }
            }
        }
        None
    }

    fn advance_till_crash(&mut self) -> Collision {
        loop {
            let res = self.advance();
            if let Err(collision) = res {
                return collision;
            }
        }
    }

    fn find_last_minecart(&mut self) -> Coord {
        while self.minecarts.len() > 1 {
            self.advance_till_crash();
        }
        return self.minecarts[0].position;
    }
}

fn parse_tracks(input: &mut impl BufRead) -> Result<Tracks, Error> {
    let mut contents = vec![];
    let mut minecarts = vec![];
    for (y, line_res) in input.lines().enumerate() {
        let line = line_res?;
        let row_contents: Vec<_> = line.chars()
            .map(|c| parse_track_part(c))
            .collect::<Result<Vec<_>, _>>()?;
        let mut row_carts: Vec<_> = line.chars()
            .enumerate()
            .filter_map(|(x, c)| parse_minecart(c, x, y))
            .collect();
        contents.push(row_contents);
        minecarts.append(&mut row_carts);
    }
    Ok(Tracks {
        contents,
        minecarts,
    })
}

fn parse_track_part(c: char) -> Result<TrackContents, Error> {
    Ok(match c {
        '/'             => CurveLeft,
        '\\'            => CurveRight,
        '|' | '^' | 'v' => Vertical,
        '-' | '>' | '<' => Horizontal,
        '+'             => Intersection,
        ' '             => Empty,
        _               => bail!("Can't parse track character {}", c),
    })
}

fn parse_minecart(c: char, x: usize, y: usize) -> Option<Minecart> {
    let orientation = match c {
        '>' => Right,
        '<' => Left,
        '^' => Up,
        'v' => Down,
        _   => return None,
    };
    Some(Minecart {
        position: Coord { x: x as i64, y: y as i64 },
        next_direction: LeftTurn,
        orientation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRACKS: &str = r"/->-\        
|   |  /----\
| /-+--+-\  |
| | |  | v  |
\-+-/  \-+--/
  \------/   ";

    #[test]
    fn test_parse_tracks() {
        let tracks = parse_tracks(&mut TRACKS.as_bytes()).expect("Couldn't parse tracks");
        assert_eq!(tracks.minecarts, vec![
                Minecart {
                    position: Coord { x: 2, y: 0 },
                    next_direction: LeftTurn,
                    orientation: Right,
                },
                Minecart {
                    position: Coord { x: 9, y: 3 },
                    next_direction: LeftTurn,
                    orientation: Down,
                }
        ]);
        assert_eq!(tracks.contents[0], vec![
            CurveLeft,
            Horizontal,
            Horizontal,
            Horizontal,
            CurveRight,
            Empty,
            Empty,
            Empty,
            Empty,
            Empty,
            Empty,
            Empty,
            Empty,
        ]);
    }

    #[test]
    fn test_advance_till_crash() {
        let mut tracks = parse_tracks(&mut TRACKS.as_bytes()).expect("Couldn't parse tracks");
        assert_eq!(tracks.advance_till_crash(), Collision(Coord { x: 7, y: 3 }));
    }

    const TRACKS2: &str = r"/>-<\  
|   |  
| /<+-\
| | | v
\>+</ |
  |   ^
  \<->/";

    #[test]
    fn test_find_last_minecart() {
        let mut tracks = parse_tracks(&mut TRACKS2.as_bytes()).expect("Couldn't parse tracks");
        assert_eq!(tracks.find_last_minecart(), Coord { x: 6, y: 4 });
    }
}
