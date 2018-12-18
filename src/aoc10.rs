use std::io;
use std::io::BufRead;
use std::fmt;
use std::fmt::Display;
use regex::Regex;
use failure::{Error, format_err, ensure};

pub fn aoc10(part2: bool) -> Result<(), Error> {
    let mut particle_field = parse_particles(&mut io::stdin().lock())?;
    particle_field.advance_to_best_distance(0);
    if part2 {
        println!("{}", particle_field.cur_step);
    } else {
        println!("{}", particle_field);
    }
    Ok(())
}

fn parse_particles(input: &mut impl BufRead) -> Result<ParticleField, Error> {
    let regex = Regex::new(r"position=<([- 0-9]+), ([- 0-9]+)> velocity=<([- 0-9]+), ([- 0-9]+)>")?;
    let mut particles = vec![];
    for line_res in input.lines() {
        let line = line_res?;
        let caps = regex.captures(&line).ok_or_else(|| format_err!("Can't parse line {}", line))?;
        let coords: Vec<_> = caps.iter().skip(1).flat_map(|c_opt| c_opt.and_then(|c| c.as_str().trim().parse::<i64>().ok())).collect();
        ensure!(coords.len() == 4, "Line {} has fewer than 4 coordinates", line);
        particles.push(Particle { x: coords[0], y: coords[1], vel_x: coords[2], vel_y: coords[3] });
    }
    Ok(ParticleField { particles, cur_step: 0 })
}

#[derive(PartialEq, Debug)]
struct Particle {
    x: i64,
    y: i64,
    vel_x: i64,
    vel_y: i64,
}

#[derive(PartialEq, Debug)]
struct ParticleField {
    particles: Vec<Particle>,
    cur_step: i64,
}

impl ParticleField {
    /// Advance the particles' positions by a number of
    /// timesteps. Negative values are allowed.
    fn advance(&mut self, steps: i64) {
        for particle in self.particles.iter_mut() {
            particle.x += particle.vel_x * steps;
            particle.y += particle.vel_y * steps;
        }
        self.cur_step += steps;
    }

    fn advance_to_step(&mut self, step: i64) {
        self.advance(step - self.cur_step);
    }

    /// Get the sum of Manhattan distances between every
    /// particle. Every pair is counted twice.
    fn mutual_distance(&self) -> u64 {
        let mut distance: u64 = 0;
        for particle1 in self.particles.iter() {
            for particle2 in self.particles.iter() {
                distance += ((particle1.x - particle2.x).abs() + (particle1.y - particle2.x).abs()) as u64;
            }
        }
        distance
    }

    fn deriv_at(&mut self, step: i64) -> i64 {
        let old_step = self.cur_step;
        self.advance_to_step(step);
        let dist = self.mutual_distance() as i64;
        self.advance(1);
        let next_dist = self.mutual_distance() as i64;
        self.advance_to_step(old_step);
        next_dist - dist
    }

    /// Find the step with lowest mutual distance in O(p^2 log n) time
    /// (p: number of particles, n: value of the best step).
    fn advance_to_best_distance(&mut self, fudge_steps: i64) -> i64 {
        let mut best_distance = self.mutual_distance();
        let mut cur_increase = 1;
        let mut lo = 0;
        let mut hi;
        // Step 1: advance by exponentially growing amounts until we
        // find the region that must contain the best step. We will
        // find this region in O(log n) loops.
        loop {
            self.advance(cur_increase);
            let distance = self.mutual_distance();
            if distance > best_distance {
                // We've overshot.
                hi = self.cur_step;
                break;
            } else {
                lo = self.cur_step - cur_increase;
                best_distance = distance;
            }
            cur_increase *= 2;
        }

        // Step 2: binary search to find the best step. Since the
        // distance isn't monotone with respect to the step, normal
        // binary search won't work. But since it must be convex, we
        // instead binary search on the *derivative* of the distance
        // to find the step with derivative closest to 0.
        loop {
            if lo >= hi {
                break;
            }
            let mid = lo + (hi - lo) / 2;
            if self.deriv_at(mid) > 0 {
                hi = mid - 1;
            } else {
                lo = mid + 1;
            }
        }

        self.advance_to_step(lo - fudge_steps);
        self.cur_step
    }
}

impl Display for ParticleField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let min_x = self.particles.iter().map(|p| p.x).min().unwrap();
        let max_x = self.particles.iter().map(|p| p.x).max().unwrap();
        let min_y = self.particles.iter().map(|p| p.y).min().unwrap();
        let max_y = self.particles.iter().map(|p| p.y).max().unwrap();
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let mut array_1d = vec![false; (width * height) as usize];
        let mut sliced_2d: Vec<_> = array_1d.as_mut_slice().chunks_mut(width as usize).collect();
        let array_2d = sliced_2d.as_mut_slice();

        for particle in self.particles.iter() {
            array_2d[(particle.y - min_y) as usize][(particle.x - min_x) as usize] = true;
        }

        for row in array_2d.iter() {
            let string: String = row.iter().map(|b| if *b { '#' } else { '.' }).collect();
            writeln!(f, "{}", string)?;
        }
        Ok(())
    }
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

    const PARTICLES: &str = "position=< 9,  1> velocity=< 0,  2>
position=< 7,  0> velocity=<-1,  0>
position=< 3, -2> velocity=<-1,  1>
position=< 6, 10> velocity=<-2, -1>
position=< 2, -4> velocity=< 2,  2>
position=<-6, 10> velocity=< 2, -2>
position=< 1,  8> velocity=< 1, -1>
position=< 1,  7> velocity=< 1,  0>
position=<-3, 11> velocity=< 1, -2>
position=< 7,  6> velocity=<-1, -1>
position=<-2,  3> velocity=< 1,  0>
position=<-4,  3> velocity=< 2,  0>
position=<10, -3> velocity=<-1,  1>
position=< 5, 11> velocity=< 1, -2>
position=< 4,  7> velocity=< 0, -1>
position=< 8, -2> velocity=< 0,  1>
position=<15,  0> velocity=<-2,  0>
position=< 1,  6> velocity=< 1,  0>
position=< 8,  9> velocity=< 0, -1>
position=< 3,  3> velocity=<-1,  1>
position=< 0,  5> velocity=< 0, -1>
position=<-2,  2> velocity=< 2,  0>
position=< 5, -2> velocity=< 1,  2>
position=< 1,  4> velocity=< 2,  1>
position=<-2,  7> velocity=< 2, -2>
position=< 3,  6> velocity=<-1, -1>
position=< 5,  0> velocity=< 1,  0>
position=<-6,  0> velocity=< 2,  0>
position=< 5,  9> velocity=< 1, -2>
position=<14,  7> velocity=<-2,  0>
position=<-3,  6> velocity=< 2, -1>";

    #[test]
    fn test_parse_particles() {
        assert_result_ok(parse_particles(&mut PARTICLES.as_bytes()), ParticleField {
            particles: vec![
                Particle { x: 9, y: 1, vel_x: 0, vel_y: 2 },
                Particle { x: 7, y: 0, vel_x: -1, vel_y: 0 },
                Particle { x: 3, y: -2, vel_x: -1, vel_y: 1 },
                Particle { x: 6, y: 10, vel_x: -2, vel_y: -1 },
                Particle { x: 2, y: -4, vel_x: 2, vel_y: 2 },
                Particle { x: -6, y: 10, vel_x: 2, vel_y: -2 },
                Particle { x: 1, y: 8, vel_x: 1, vel_y: -1 },
                Particle { x: 1, y: 7, vel_x: 1, vel_y: 0 },
                Particle { x: -3, y: 11, vel_x: 1, vel_y: -2 },
                Particle { x: 7, y: 6, vel_x: -1, vel_y: -1 },
                Particle { x: -2, y: 3, vel_x: 1, vel_y: 0 },
                Particle { x: -4, y: 3, vel_x: 2, vel_y: 0 },
                Particle { x: 10, y: -3, vel_x: -1, vel_y: 1 },
                Particle { x: 5, y: 11, vel_x: 1, vel_y: -2 },
                Particle { x: 4, y: 7, vel_x: 0, vel_y: -1 },
                Particle { x: 8, y: -2, vel_x: 0, vel_y: 1 },
                Particle { x: 15, y: 0, vel_x: -2, vel_y: 0 },
                Particle { x: 1, y: 6, vel_x: 1, vel_y: 0 },
                Particle { x: 8, y: 9, vel_x: 0, vel_y: -1 },
                Particle { x: 3, y: 3, vel_x: -1, vel_y: 1 },
                Particle { x: 0, y: 5, vel_x: 0, vel_y: -1 },
                Particle { x: -2, y: 2, vel_x: 2, vel_y: 0 },
                Particle { x: 5, y: -2, vel_x: 1, vel_y: 2 },
                Particle { x: 1, y: 4, vel_x: 2, vel_y: 1 },
                Particle { x: -2, y: 7, vel_x: 2, vel_y: -2 },
                Particle { x: 3, y: 6, vel_x: -1, vel_y: -1 },
                Particle { x: 5, y: 0, vel_x: 1, vel_y: 0 },
                Particle { x: -6, y: 0, vel_x: 2, vel_y: 0 },
                Particle { x: 5, y: 9, vel_x: 1, vel_y: -2 },
                Particle { x: 14, y: 7, vel_x: -2, vel_y: 0 },
                Particle { x: -3, y: 6, vel_x: 2, vel_y: -1 },
            ],
            cur_step: 0,
        })
    }

    #[test]
    fn test_particlefield_display() {
        let particle_field = parse_particles(&mut PARTICLES.as_bytes()).expect("Couldn't parse particles");
        assert_eq!(format!("{}", particle_field), "........#.............
................#.....
.........#.#..#.......
......................
#..........#.#.......#
...............#......
....#.................
..#.#....#............
.......#..............
......#...............
...#...#.#...#........
....#..#..#.........#.
.......#..............
...........#..#.......
#...........#.........
...#.......#..........
");
    }

    #[test]
    fn test_particlefield_advance() {
        let mut particle_field = parse_particles(&mut PARTICLES.as_bytes()).expect("Couldn't parse particles");
        particle_field.advance(3);
        assert_eq!(format!("{}", particle_field), "#...#..###
#...#...#.
#...#...#.
#####...#.
#...#...#.
#...#...#.
#...#...#.
#...#..###
");
    }

    #[test]
    fn test_particlefield_advance_to_best_distance() {
        let mut particle_field = parse_particles(&mut PARTICLES.as_bytes()).expect("Couldn't parse particles");
        assert_eq!(particle_field.advance_to_best_distance(0), 3);
        particle_field.advance(-10000);
        assert_eq!(particle_field.advance_to_best_distance(0), 3);
        particle_field.advance(-100000);
        assert_eq!(particle_field.advance_to_best_distance(0), 3);
    }
}
