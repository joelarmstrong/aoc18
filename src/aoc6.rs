use std::io;
use std::io::BufRead;
use std::collections::{HashSet, HashMap};
use failure::{Error, format_err};

pub fn aoc6(part2: bool) -> Result<(), Error> {
    let coords = parse_coords(&mut io::stdin().lock())?;
    if part2 {
        println!("Size of close region: {}", size_of_close_region(&coords, 10000).expect("Couldn't find close region"));
    } else {
        println!("Largest non-infinite area: {}", largest_non_infinite_area(&coords).expect("Couldn't find area"));
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Coord {
    pub x: i64,
    pub y: i64,
}

impl Coord {
    fn distance(&self, other: &Coord) -> i64 {
        (other.x - self.x).abs() + (other.y - self.y).abs()
    }
}

fn parse_coord(string: &str) -> Result<Coord, Error> {
    let mut iter = string.split(", ").map(|c| c.parse::<i64>());
    let x = iter.next().ok_or_else(|| format_err!("not enough fields"))??;
    let y = iter.next().ok_or_else(|| format_err!("not enough fields"))??;
    Ok(Coord { x, y })
}

fn parse_coords(input: &mut impl BufRead) -> Result<Vec<Coord>, Error> {
    input.lines().map(|s| parse_coord(&s?)).collect()
}

type BBox = (i64, i64, i64, i64);

fn get_bounding_box(coords: &[Coord]) -> Option<BBox> {
    let min_x = coords.iter().min_by_key(|c| c.x)?.x;
    let max_x = coords.iter().max_by_key(|c| c.x)?.x;
    let min_y = coords.iter().min_by_key(|c| c.y)?.y;
    let max_y = coords.iter().max_by_key(|c| c.y)?.y;
    Some((min_x, max_x, min_y, max_y))
}

fn largest_non_infinite_area(coords: &[Coord]) -> Option<u64> {
    let bbox = get_bounding_box(&coords)?;
    let width = bbox.1 - bbox.0;
    let height = bbox.3 - bbox.2;
    let mut assignments: Vec<Option<&Coord>> = vec![None; (width * height) as usize];
    let mut infinite_coords: HashSet<&Coord> = HashSet::new();
    for x in bbox.0..=bbox.1 {
        for y in bbox.2..=bbox.3 {
            let on_edge = x == bbox.0 || x == bbox.1 || y == bbox.2 || y == bbox.3;
            let point = Coord { x, y };
            let min_coord = coords.iter().min_by_key(|c| point.distance(c)).unwrap();
            let unique = coords.iter().filter(|coord| point.distance(coord) == point.distance(min_coord)).count() == 1;
            if on_edge {
                infinite_coords.insert(min_coord);
            } else if unique {
                let index: usize = (width * (y - bbox.2) + (x - bbox.0)) as usize;
                assert!(assignments[index] == None);
                assignments[index] = Some(min_coord);
            }
        }
    }
    let mut counts: HashMap<&Coord, u64> = HashMap::new();
    for assignment in assignments {
        if let Some(coord) = assignment {
             if !infinite_coords.contains(coord) {
                 *counts.entry(coord).or_insert(0) += 1;
             }
        }
    }
    Some(*counts.values().max()?)
}

fn size_of_close_region(coords: &[Coord], close_distance: i64) -> Option<u64> {
    let bbox = get_bounding_box(&coords)?;
    let mut area = 0;
    for x in bbox.0..=bbox.1 {
        for y in bbox.2..=bbox.3 {
            let point = Coord { x, y };
            let distance_sum: i64 = coords.iter().map(|c| point.distance(c)).sum();
            if distance_sum < close_distance {
                area += 1;
            }
        }
    }
    Some(area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    /// This function allows us to assert that a Result is
    /// Ok(expected) without requiring PartialEq on the Error type.
    fn assert_result_ok<T: Debug + PartialEq>(r: Result<T, Error>, expected: T) {
        match r {
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("got Err: {}, local backtrace: {}", e, e.backtrace()),
        }
    }


    const COORDS: &str = "1, 1
1, 6
8, 3
3, 4
5, 5
8, 9";

    #[test]
    fn test_parse_coords() {
        assert_result_ok(parse_coords(&mut COORDS.as_bytes()), vec![
            Coord { x: 1, y: 1 },
            Coord { x: 1, y: 6 },
            Coord { x: 8, y: 3 },
            Coord { x: 3, y: 4 },
            Coord { x: 5, y: 5 },
            Coord { x: 8, y: 9 },
        ]);
    }

    #[test]
    fn test_get_bounding_box() {
        let coords = parse_coords(&mut COORDS.as_bytes()).expect("Couldn't parse coordinates");
        assert_eq!(get_bounding_box(&coords), Some((1, 8, 1, 9)));
    }

    #[test]
    fn test_largest_non_infinite_area() {
        let coords = parse_coords(&mut COORDS.as_bytes()).expect("Couldn't parse coordinates");
        assert_eq!(largest_non_infinite_area(&coords), Some(17))
    }

    #[test]
    fn test_size_of_close_region() {
        let coords = parse_coords(&mut COORDS.as_bytes()).expect("Couldn't parse coordinates");
        assert_eq!(size_of_close_region(&coords, 32), Some(16))
    }
}
