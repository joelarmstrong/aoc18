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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct AlignedCuboid {
    /// Corner which is smallest in all of (x, y, z).
    left_corner: Point3d,
    /// Width in X axis.
    length: u32,
    /// Width in Y axis.
    width: u32,
    /// Width in Z axis.
    height: u32,
    /// Number of nanobots in range.
    in_range: u32,
}

#[derive(Debug, PartialEq, Hash, Eq)]
struct CuboidIntersection {
    cuboid: AlignedCuboid,
    intersects: Intersects,
}

fn cuboid_intersects_from_range_intersects(x: Intersects, y: Intersects, z: Intersects) -> Intersects {
    if x == Both && y == Both && z == Both {
        Both
    } else if x == Range1 || y == Range1 || z == Range1 {
        assert!(x != Range2 && y != Range2 && z != Range2);
        Range1
    } else {
        assert!(x != Range1 && y != Range1 && z != Range1);
        Range2
    }
}

impl AlignedCuboid {
    fn overlaps(&self, other: &AlignedCuboid) -> bool {
        range_overlaps(self.x_range(), other.x_range()) &&
        range_overlaps(self.y_range(), other.y_range()) &&
        range_overlaps(self.z_range(), other.z_range())
    }

    fn intersect(&self, other: &AlignedCuboid) -> Vec<CuboidIntersection> {
        dbg!(self, other);
        let mut ret = vec![];
        for x in range_split(self.x_range(), other.x_range()) {
            for y in range_split(self.y_range(), other.y_range()).into_iter().filter(|y| x.intersects == Both || y.intersects == Both || x.intersects == y.intersects) {
                for z in range_split(self.z_range(), other.z_range()).into_iter().filter(|z| {
                    if z.intersects == Both {
                        true
                    } else if x.intersects == Both {
                        y.intersects == Both || z.intersects == y.intersects
                    } else if y.intersects == Both {
                        x.intersects == Both || z.intersects == x.intersects
                    } else if z.intersects == y.intersects {
                        true
                    } else {
                        false
                    }
                }) {
                    let intersects = cuboid_intersects_from_range_intersects(
                            x.intersects, y.intersects, z.intersects); 
                    ret.push(CuboidIntersection {
                        intersects,
                        cuboid: AlignedCuboid {
                            left_corner: Point3d {
                                x: *x.range.start(),
                                y: *y.range.start(),
                                z: *z.range.start(),
                            },
                            length: (x.range.end() - x.range.start()) as u32,
                            width: (y.range.end() - y.range.start()) as u32,
                            height: (z.range.end() - z.range.start()) as u32,
                            in_range: match intersects {
                                Both => self.in_range + other.in_range,
                                Range1 => self.in_range,
                                Range2 => other.in_range,
                            }
                        },
                    });
                }
            }
        }
        let c: Vec<_> = ret.iter().map(|ci| ci.cuboid.clone()).collect();
        dbg!(&c);
        assert_eq!(adjusted_volume(&c), self.adjusted_volume() + other.adjusted_volume());
        ret
    }

    fn closest_to_origin(&self) -> Point3d {
        let x = if self.x_range().contains(&0) {
            0
        } else if self.x_range().start().abs() < self.x_range().end().abs() {
            *self.x_range().start()
        } else {
            *self.x_range().end()
        };
        let y = if self.y_range().contains(&0) {
            0
        } else if self.y_range().start().abs() < self.y_range().end().abs() {
            *self.y_range().start()
        } else {
            *self.y_range().end()
        };
        let z = if self.z_range().contains(&0) {
            0
        } else if self.z_range().start().abs() < self.z_range().end().abs() {
            *self.z_range().start()
        } else {
            *self.z_range().end()
        };
        Point3d { x, y, z }
    }

    fn x_range(&self) -> RangeInclusive<i32> {
        self.left_corner.x..=self.left_corner.x + (self.length as i32)
    }

    fn y_range(&self) -> RangeInclusive<i32> {
        self.left_corner.y..=self.left_corner.y + (self.width as i32)
    }

    fn z_range(&self) -> RangeInclusive<i32> {
        self.left_corner.z..=self.left_corner.z + (self.height as i32)
    }

    fn adjusted_volume(&self) -> u32 {
        self.in_range * (self.width + 1) * (self.height + 1) * (self.length + 1)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Intersects {
    Range1,
    Range2,
    Both,
}

use Intersects::*;

#[derive(Debug, PartialEq, Clone)]
struct IntersectionResult {
    range: RangeInclusive<i32>,
    intersects: Intersects,
}

fn adjusted_volume(cuboids: &[AlignedCuboid]) -> u32 {
    cuboids.iter().map(|c| c.adjusted_volume()).sum()
}

fn range_overlaps<T: Ord>(range_1: RangeInclusive<T>, range_2: RangeInclusive<T>) -> bool {
    range_1.start() <= range_2.end() && range_2.start() <= range_1.end()
}

fn range_split(range_1: RangeInclusive<i32>, range_2: RangeInclusive<i32>) -> Vec<IntersectionResult> {
    let mut ret = vec![];
    let (range_1, range_2, v1, v2) = if range_2.start() <= range_1.start()
        || (range_1.start() == range_2.start() && range_1.end() < range_2.end()) {
        (range_2, range_1, Range2, Range1)
    } else {
        (range_1, range_2, Range1, Range2)
    };
    if range_2.end() < range_1.end() {
        // Containment of range_2 by range_1.
        if range_1.start() < range_2.start() {
            ret.push(IntersectionResult {
                range: *range_1.start()..=(range_2.start() - 1),
                intersects: v1,
            });
        }
        ret.push(IntersectionResult {
            range: *range_2.start()..=(*range_2.end()),
            intersects: Both,
        });
        if range_2.end() < range_1.end() {
            ret.push(IntersectionResult {
                range: (range_2.end() + 1)..=*range_1.end(),
                intersects: v1,
            });
        }

    } else {
        if range_1.start() < range_2.start() {
            ret.push(IntersectionResult {
                range: *range_1.start()..=(range_2.start() - 1),
                intersects: v1,
            });
        }
        ret.push(IntersectionResult {
            range: *range_2.start()..=(*range_1.end()),
            intersects: Both,
        });
        if range_1.end() < range_2.end() {
            ret.push(IntersectionResult {
                range: (range_1.end() + 1)..=*range_2.end(),
                intersects: v2,
            });
        }
    }
    ret
}

fn insert_cuboid_breaking_overlaps(cuboid: AlignedCuboid, non_overlapping_cuboids: &mut Vec<AlignedCuboid>) {
    let prev_volume = adjusted_volume(non_overlapping_cuboids);
    let added_volume = cuboid.adjusted_volume();
    dbg!(&cuboid);
    let mut potentially_overlapping_pieces = vec![cuboid];
    while !potentially_overlapping_pieces.is_empty() {
        let current_piece = potentially_overlapping_pieces.pop().unwrap();
        dbg!(&current_piece);
        let mut to_insert = vec![];
        let mut intersected_piece = None;
        for i in 0..non_overlapping_cuboids.len() {
            let other_cuboid = &non_overlapping_cuboids[i];
            if current_piece.overlaps(other_cuboid) {
                let intersection_results = current_piece.intersect(other_cuboid);
                for intersection_result in intersection_results {
                    match intersection_result.intersects {
                        Both | Range2 => to_insert.push(intersection_result.cuboid),
                        Range1 => potentially_overlapping_pieces.push(intersection_result.cuboid),
                    }
                }
                intersected_piece = Some(i);
                break;
            }
        }
        if let Some(i) = intersected_piece {
            non_overlapping_cuboids.remove(i);
        } else {
            to_insert.push(current_piece);
        }
        dbg!(&to_insert);
        non_overlapping_cuboids.extend(to_insert);
    }
    for i in 0..non_overlapping_cuboids.len() {
        for j in (i + 1)..non_overlapping_cuboids.len() {
            assert!(!non_overlapping_cuboids[i].overlaps(&non_overlapping_cuboids[j]));
        }
    }
    assert_eq!(adjusted_volume(non_overlapping_cuboids), prev_volume + added_volume);
}

fn initial_cuboid_for_nanobot(nanobot: &Nanobot) -> AlignedCuboid {
    let diameter = (nanobot.radius as u32) * 2;
    AlignedCuboid {
        left_corner: Point3d {
            x: nanobot.pos.x - nanobot.radius,
            y: nanobot.pos.y - nanobot.radius,
            z: nanobot.pos.z - nanobot.radius,
        },
        length: diameter,
        width: diameter,
        height: diameter,
        in_range: 1,
    }
}

fn non_overlapping_cuboids(nanobots: &[Nanobot]) -> Vec<AlignedCuboid> {
    let mut cuboids = vec![];
    for nanobot in nanobots {
        let cuboid = initial_cuboid_for_nanobot(nanobot);
        insert_cuboid_breaking_overlaps(cuboid, &mut cuboids);
        dbg!(&cuboids);
    }
    cuboids
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
    fn test_initial_cuboid_for_nanobot() {
        let nanobot = Nanobot { pos: Point3d { x: 0, y: 5, z: 0 }, radius: 3 };
        assert_eq!(initial_cuboid_for_nanobot(&nanobot), AlignedCuboid {
            left_corner: Point3d { x: -3, y: 2, z: -3 },
            width: 6,
            height: 6,
            length: 6,
            in_range: 1,
        });
    }

    #[test]
    fn test_alignedcuboid_overlaps() {
        let nanobot_1 = Nanobot { pos: Point3d { x: 0, y: 5, z: 0 }, radius: 3 };
        let cuboid_1 = initial_cuboid_for_nanobot(&nanobot_1);
        let nanobot_2 = Nanobot { pos: Point3d { x: 4, y: 0, z: 4 }, radius: 4 };
        let cuboid_2 = initial_cuboid_for_nanobot(&nanobot_2);
        let nanobot_3 = Nanobot { pos: Point3d { x: 8, y: -8, z: 5 }, radius: 4 };
        let cuboid_3 = initial_cuboid_for_nanobot(&nanobot_3);
        assert!(cuboid_1.overlaps(&cuboid_2));
        assert!(cuboid_2.overlaps(&cuboid_1));
        assert!(!cuboid_3.overlaps(&cuboid_1));
        assert!(!cuboid_1.overlaps(&cuboid_3));
        assert!(cuboid_3.overlaps(&cuboid_2));
        assert!(cuboid_2.overlaps(&cuboid_3));
    }

    #[test]
    fn test_range_split() {
        assert_eq!(range_split(0..=5, 0..=5), vec![IntersectionResult { range: 0..=5, intersects: Both }]);
        assert_eq!(range_split(2..=3, 0..=2), vec![
            IntersectionResult { range: 0..=1, intersects: Range2 },
            IntersectionResult { range: 2..=2, intersects: Both },
            IntersectionResult { range: 3..=3, intersects: Range1 },
        ]);
        assert_eq!(range_split(0..=2, 2..=3), vec![
            IntersectionResult { range: 0..=1, intersects: Range1 },
            IntersectionResult { range: 2..=2, intersects: Both },
            IntersectionResult { range: 3..=3, intersects: Range2 },
        ]);
        assert_eq!(range_split(2..=5, 0..=8), vec![
            IntersectionResult { range: 0..=1, intersects: Range2, },
            IntersectionResult { range: 2..=5, intersects: Both },
            IntersectionResult { range: 6..=8, intersects: Range2 },
            ]);
        assert_eq!(range_split(0..=8, 2..=5), vec![
            IntersectionResult { range: 0..=1, intersects: Range1, },
            IntersectionResult { range: 2..=5, intersects: Both },
            IntersectionResult { range: 6..=8, intersects: Range1 },
            ]);
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

    #[test]
    fn test_alignedcuboid_intersect() {
        let cuboid_1 = AlignedCuboid {
            left_corner: Point3d {
                x: 10,
                y: 12,
                z: 10,
            },
            length: 4,
            width: 4,
            height: 4,
            in_range: 1,
        };
        let cuboid_2 = AlignedCuboid {
            left_corner: Point3d {
                x: 8,
                y: 10,
                z: 10,
            },
            length: 4,
            width: 4,
            height: 4,
            in_range: 1,
        };
        let results: HashSet<_> = cuboid_1.intersect(&cuboid_2).into_iter().collect();
        let expected: HashSet<_> = vec![
            CuboidIntersection {
                cuboid: AlignedCuboid {
                    left_corner: Point3d { x: 10, y: 12, z: 10 },
                    length: 2,
                    width: 2,
                    height: 4,
                    in_range: 2,
                },
                intersects: Both
            },
            CuboidIntersection {
                cuboid: AlignedCuboid {
                    left_corner: Point3d { x: 8, y: 10, z: 10 },
                    length: 1,
                    width: 1,
                    height: 4,
                    in_range: 1,
                },
                intersects: Range2
            },
            CuboidIntersection {
                cuboid: AlignedCuboid {
                    left_corner: Point3d { x: 8, y: 12, z: 10 },
                    length: 1,
                    width: 2,
                    height: 4,
                    in_range: 1,
                },
                intersects: Range2
            },
            CuboidIntersection {
                cuboid: AlignedCuboid {
                    left_corner: Point3d { x: 10, y: 10, z: 10 },
                    length: 2,
                    width: 1,
                    height: 4,
                    in_range: 1,
                },
                intersects: Range2
            },
            CuboidIntersection {
                cuboid: AlignedCuboid {
                    left_corner: Point3d { x: 13, y: 12, z: 10 },
                    length: 1,
                    width: 2,
                    height: 4,
                    in_range: 1,
                },
                intersects: Range1
            },
            CuboidIntersection {
                cuboid: AlignedCuboid {
                    left_corner: Point3d { x: 10, y: 15, z: 10 },
                    length: 2,
                    width: 1,
                    height: 4,
                    in_range: 1,
                },
                intersects: Range1
            },
            CuboidIntersection {
                cuboid: AlignedCuboid {
                    left_corner: Point3d { x: 13, y: 15, z: 10 },
                    length: 1,
                    width: 1,
                    height: 4,
                    in_range: 1,
                },
                intersects: Range1
            },
        ].into_iter().collect();
        dbg!(results.difference(&expected));
        dbg!(expected.difference(&results));
        assert_eq!(results, expected);
    }
}
