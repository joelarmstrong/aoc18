use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::ops::RangeInclusive;
use regex::Regex;
use rayon::prelude::*;
use failure::{Error, ensure, format_err};
use rand::{thread_rng, seq::IteratorRandom};
pub fn aoc23(part2: bool) -> Result<(), Error> {
    let nanobots = parse_nanobots(&mut io::stdin().lock())?;
    if part2 {
        let point = best_point(&nanobots);
        println!("best point: {:?}, distance: {}", point, manhattan_distance(ORIGIN, point));
    } else {
        let strongest = strongest_nanobot(&nanobots);
        println!("Nanobots in range of strongest: {}", nanobots_in_range(&strongest, &nanobots));
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Point3d {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, PartialEq)]
struct Nanobot {
    pos: Point3d,
    radius: i32,
}

const ORIGIN: Point3d = Point3d { x: 0, y: 0, z: 0 };

fn decaying_score(distance: i32, nanobot: &Nanobot) -> f64 {
    if distance <= nanobot.radius {
        1.0
    } else {
        0.999999_f64.powf((distance - nanobot.radius) as f64)
    }
}

fn decaying_score_for_pos(pos: Point3d, nanobots: &[Nanobot]) -> f64 {
    nanobots.iter().map(|n| decaying_score(manhattan_distance(pos, n.pos), n)).sum()
}

fn hill_climb(original_point: Point3d, nanobots: &[Nanobot]) -> Point3d {
    let mut point = original_point;
    let mut score = decaying_score_for_pos(point, nanobots);
    // Find best point through hill-climbing.
    loop {
        let points = vec![Point3d { x: point.x + 1, y: point.y, z: point.z },
                          Point3d { x: point.x, y: point.y + 1, z: point.z },
                          Point3d { x: point.x, y: point.y, z: point.z + 1 },
                          Point3d { x: point.x - 1, y: point.y, z: point.z },
                          Point3d { x: point.x, y: point.y - 1, z: point.z },
                          Point3d { x: point.x, y: point.y, z: point.z - 1 },
                          point,
                          Point3d { x: point.x + 500, y: point.y, z: point.z },
                          Point3d { x: point.x, y: point.y + 500, z: point.z },
                          Point3d { x: point.x, y: point.y, z: point.z + 500 },
                          Point3d { x: point.x - 500, y: point.y, z: point.z },
                          Point3d { x: point.x, y: point.y - 500, z: point.z },
                          Point3d { x: point.x, y: point.y, z: point.z - 500 },
        ];
        let points_and_scores: Vec<_> = points.into_iter().map(|p| (p, decaying_score_for_pos(p, nanobots))).collect();
        let (p, s) = points_and_scores.into_iter().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap();
        if p == point {
            // Converged.
            break;
        }
        point = p;
        score = s;
    }
    point
}

fn best_point(nanobots: &[Nanobot]) -> Point3d {
    let mut rng = thread_rng();
    let mut start_positions: Vec<Point3d> = nanobots.iter().map(|c| c.pos).choose_multiple(&mut rng, 23);
    start_positions.push(ORIGIN);
    dbg!(&start_positions);
    let mut rng = thread_rng();
    let final_positions: Vec<Point3d> = start_positions.par_iter().map(|p| hill_climb(*p, nanobots)).collect();
    let points_and_scores: Vec<_> = final_positions.into_iter().map(|p| (p, decaying_score_for_pos(p, nanobots))).collect();
    dbg!(&points_and_scores);
    points_and_scores.into_iter().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).map(|(p, _)| p).unwrap()
}

fn parse_nanobot(line: &str) -> Result<Nanobot, Error> {
    let regex = Regex::new(r"pos=<([-0-9]+),([-0-9]+),([-0-9]+)>, r=([0-9]+)")?;
    let caps = regex.captures(line).ok_or_else(|| format_err!("Can't understand line {}", line))?;
    ensure!(caps.len() == 5, "Line has fewer than 4 captures");
    let x: i32 = caps.get(1).unwrap().as_str().parse()?;
    let y: i32 = caps.get(2).unwrap().as_str().parse()?;
    let z: i32 = caps.get(3).unwrap().as_str().parse()?;
    let radius: i32 = caps.get(4).unwrap().as_str().parse()?;
    Ok(Nanobot {
        pos: Point3d { x, y, z },
        radius,
    })
}

fn manhattan_distance(point1: Point3d, point2: Point3d) -> i32 {
    (point1.x - point2.x).abs() +
    (point1.y - point2.y).abs() +
    (point1.z - point2.z).abs()
}

fn nanobots_in_range(ref_nanobot: &Nanobot, nanobots: &[Nanobot]) -> usize {
    let mut num = 0;
    for nanobot in nanobots.iter() {
        if manhattan_distance(ref_nanobot.pos, nanobot.pos) <= ref_nanobot.radius {
            num += 1;
        }
    }
    num
}

fn strongest_nanobot<'a>(nanobots: &'a [Nanobot]) -> &'a Nanobot {
    nanobots.iter().max_by_key(|n| n.radius).unwrap()
}

fn parse_nanobots(input: &mut impl BufRead) -> Result<Vec<Nanobot>, Error> {
    let mut nanobots = vec![];
    for line_res in input.lines() {
        let line = line_res?;
        nanobots.push(parse_nanobot(&line)?);
    }
    Ok(nanobots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nanobots() -> Result<(), Error> {
        let input_str = "pos=<0,0,0>, r=4
pos=<1,0,0>, r=1
pos=<4,0,0>, r=3
pos=<0,2,0>, r=1
pos=<0,5,0>, r=3
pos=<0,0,3>, r=1
pos=<1,1,1>, r=1
pos=<1,1,2>, r=1
pos=<1,3,1>, r=1
";
        let nanobots = parse_nanobots(&mut input_str.as_bytes())?;
        assert_eq!(nanobots, vec![
            Nanobot { pos: Point3d { x: 0, y: 0, z: 0 }, radius: 4 },
            Nanobot { pos: Point3d { x: 1, y: 0, z: 0 }, radius: 1 },
            Nanobot { pos: Point3d { x: 4, y: 0, z: 0 }, radius: 3 },
            Nanobot { pos: Point3d { x: 0, y: 2, z: 0 }, radius: 1 },
            Nanobot { pos: Point3d { x: 0, y: 5, z: 0 }, radius: 3 },
            Nanobot { pos: Point3d { x: 0, y: 0, z: 3 }, radius: 1 },
            Nanobot { pos: Point3d { x: 1, y: 1, z: 1 }, radius: 1 },
            Nanobot { pos: Point3d { x: 1, y: 1, z: 2 }, radius: 1 },
            Nanobot { pos: Point3d { x: 1, y: 3, z: 1 }, radius: 1 },
        ]);
        Ok(())
    }

    #[test]
    fn test_nanobots_in_range() -> Result<(), Error> {
        let input_str = "pos=<0,0,0>, r=4
pos=<1,0,0>, r=1
pos=<4,0,0>, r=3
pos=<0,2,0>, r=1
pos=<0,5,0>, r=3
pos=<0,0,3>, r=1
pos=<1,1,1>, r=1
pos=<1,1,2>, r=1
pos=<1,3,1>, r=1
";
        let nanobots = parse_nanobots(&mut input_str.as_bytes())?;
        let strongest = strongest_nanobot(&nanobots);
        assert_eq!(nanobots_in_range(&strongest, &nanobots), 7);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<(), Error> {
        let input_str = "pos=<10,12,12>, r=2
pos=<12,14,12>, r=2
pos=<16,12,12>, r=4
pos=<14,14,14>, r=6
pos=<50,50,50>, r=200
pos=<10,10,10>, r=5
";
        let nanobots = parse_nanobots(&mut input_str.as_bytes())?;
        assert_eq!(best_point(&nanobots), Point3d { x: 12, y: 12, z: 12 });
        Ok(())
    }
}
