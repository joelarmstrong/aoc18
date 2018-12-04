use std::io;
use std::io::BufRead;
use failure::Error;

pub fn aoc3(part2: bool) -> Result<(), Error> {
    let stdin = io::stdin();
    let claims: Vec<Claim> = stdin.lock()
        .lines()
        .flat_map(|l_r| l_r.map(|l| parse_claim(&l)))
        .collect::<Result<_, _>>()?;
    if part2 {
        if let Some(claim) = find_intact_claim(&claims) {
            println!("First intact claim found: {}", claim.id);
        } else {
            bail!("Couldn't find any intact claim");
        }
    } else {
        let overlap = calculate_overlap(&claims);
        println!("Overlapping squares: {}", overlap);
    }
    Ok(())
}

#[derive(PartialEq, Debug)]
struct Claim {
    id: u64,
    /// Distance from left edge of the fabric to upper left corner of the claim.
    x: u64,
    /// Distance from top edge of the fabric to upper left corner of the claim.
    y: u64,
    width: u64,
    height: u64,
}

fn parse_claim(line: &str) -> Result<Claim, Error> {
    let fields: Vec<_> = line.split(' ').collect();
    ensure!(fields.len() == 4, "Incorrect number of fields in claim");
    // Field 1: ID
    ensure!(&fields[0][0..1] == "#", "ID format not valid");
    let id: u64 = fields[0][1..].parse()?;

    // Field 2: junk
    ensure!(fields[1] == "@", "No @ separator found");

    // Field 3: corner coords
    ensure!(&fields[2][fields[2].len() - 1..] == ":", "Coords don't end with :");
    let corner_coords: Vec<_> = fields[2][..fields[2].len() - 1]
        .split(',')
        .map(|s| s.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()?;
    ensure!(corner_coords.len() == 2, "Too many values for corner coordinates");
    let x = corner_coords[0];
    let y = corner_coords[1];

    // Field 4: width / height
    let width_height: Vec<_> = fields[3]
        .split('x')
        .map(|s| s.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()?;
    ensure!(width_height.len() == 2, "Too many values for width/height");
    let width = width_height[0];
    let height = width_height[1];
    Ok(Claim {
        id,
        x,
        y,
        width,
        height,
    })
}

/// Represents the fabric and its coverage by various claims.
struct Fabric {
    /// Bit-vector representing square inches covered once or more.
    /// (this is implemented as a Vec<u8> because Vec<bool> takes the
    /// same amount of space).
    covered_once: Vec<u8>,
    /// Bit-vector representing square inches covered twice or more.
    covered_twice: Vec<u8>,
    height: usize,
}

impl Fabric {
    fn new(width: usize, height: usize) -> Self {
        Fabric {
            height,
            covered_once: vec![0; (width * height / 8) + 1],
            covered_twice: vec![0; (width * height / 8) + 1],
        }
    }

    /// Get the index of the u8 that contains our square.
    fn index(&self, width: usize, height: usize) -> usize {
        width * self.height / 8 + height / 8
    }

    /// Add a unit of coverage for a single square.
    fn add_coverage(&mut self, width: usize, height: usize) {
        let index = self.index(width, height);
        let covered_once = self.covered_once[index] & (0x1 << (height % 8));
        self.covered_once[index] |= 0x1 << (height % 8);
        if covered_once > 0 {
            self.covered_twice[index] |= 0x1 << (height % 8);
        }
    }

    /// Add coverage from all given claims.
    fn add_claims(&mut self, claims: &[Claim]) {
        for claim in claims {
            for x in claim.x..claim.x + claim.width {
                for y in claim.y..claim.y + claim.height {
                    self.add_coverage(x as usize, y as usize)
                }
            }
        }
    }

    /// Is this square covered once (but not twice)?
    fn covered_exactly_once(&self, width: usize, height: usize) -> bool {
        let index = self.index(width, height);
        let covered_once = self.covered_once[index] & (0x1 << (height % 8));
        let covered_twice = self.covered_twice[index] & (0x1 << (height % 8));
        covered_once > 0 && covered_twice == 0
    }

    /// Calculate the total number of squares covered more than once.
    fn total_overlap(&self) -> u64 {
        self.covered_twice.iter().map(|x| u64::from(x.count_ones())).sum()
    }
}

/// Create a Fabric that is barely large enough to cover the given Claims.
fn create_minimal_fabric(claims: &[Claim]) -> Fabric {
    let fabric_width: u64 = claims.iter().map(|c| c.x + c.width).max().unwrap_or(0);
    let fabric_height: u64 = claims.iter().map(|c| c.y + c.height).max().unwrap_or(0);
    Fabric::new(fabric_width as usize, fabric_height as usize)
}

/// Implements part 1.
fn calculate_overlap(claims: &[Claim]) -> u64 {
    let mut fabric = create_minimal_fabric(claims);
    fabric.add_claims(claims);
    fabric.total_overlap()
}

/// Implements part 2.
fn find_intact_claim(claims: &[Claim]) -> Option<&Claim> {
    let mut fabric = create_minimal_fabric(claims);
    fabric.add_claims(claims);
    'claim_loop: for claim in claims {
        for x in claim.x..claim.x + claim.width {
            for y in claim.y..claim.y + claim.height {
                if !fabric.covered_exactly_once(x as usize, y as usize) {
                    continue 'claim_loop;
                }
            }
        }
        return Some(claim);
    }
    None
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
            Err(e) => panic!("got Err: {}", e),
        }
    }

    #[test]
    fn test_parse_claim() {
        assert_result_ok(parse_claim("#123 @ 3,2: 5x4"), Claim {
            id: 123,
            x: 3,
            y: 2,
            width: 5,
            height: 4,
        });
    }

    #[test]
    fn test_calculate_overlap() {
        let claims = vec![
            Claim {
                id: 1,
                x: 1,
                y: 3,
                width: 4,
                height: 4,
            },
            Claim {
                id: 2,
                x: 3,
                y: 1,
                width: 4,
                height: 4,
            },
            Claim {
                id: 3,
                x: 5,
                y: 5,
                width: 2,
                height: 2,
            },
        ];
        assert_eq!(calculate_overlap(&claims), 4);
    }

    #[test]
    fn test_find_intact_claim() {
        let claims = vec![
            Claim {
                id: 1,
                x: 1,
                y: 3,
                width: 4,
                height: 4,
            },
            Claim {
                id: 2,
                x: 3,
                y: 1,
                width: 4,
                height: 4,
            },
            Claim {
                id: 3,
                x: 5,
                y: 5,
                width: 2,
                height: 2,
            },
        ];
        assert_eq!(find_intact_claim(&claims), Some(&claims[2]));
    }
}
