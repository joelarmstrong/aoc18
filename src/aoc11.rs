use std::io;
use std::io::Read;
use failure::Error;

pub fn aoc11(part2: bool) -> Result<(), Error> {
    let mut s = String::new();
    io::stdin().lock().read_to_string(&mut s)?;
    let serial: usize = s.trim().parse()?;
    let fc = FuelCells::new(serial);
    if part2 {
        println!("Largest total square: {:?}", fc.find_largest_total_square());
    } else {
        println!("Largest 3x3 square: {:?}", fc.find_largest_square(3, 3));
    }
    Ok(())
}

struct FuelCells {
    cells: [[i8; 300]; 300],
    serial: usize,
}

impl FuelCells {
    fn new(serial: usize) -> Self {
        let mut fc = FuelCells {
            cells: [[0; 300]; 300],
            serial,
        };
        for x in 0..300 {
            for y in 0..300 {
                fc.cells[x][y] = power_level(x, y, fc.serial);
            }
        }
        fc
    }

    fn find_largest_square(&self, width: usize, height: usize) -> ((usize, usize), i64) {
        let mut best_sum = 0;
        let mut best_index = (0, 0);
        for x in 0..300-width {
            for y in 0..300-height {
                let mut sum: i64 = 0;
                for x2 in x..x+height {
                    for y2 in y..y+width {
                        sum += i64::from(self.cells[x2][y2]);
                    }
                }
                if sum > best_sum {
                    best_sum = sum;
                    best_index = (x, y);
                }
            }
        }
        (best_index, best_sum)
    }

    fn find_largest_total_square(&self) -> ((usize, usize), usize) {
        let mut best_sum = 0;
        let mut best_size = 0;
        let mut best_index = (0, 0);
        for size in 1..=300 {
            let (index, sum) = self.find_largest_square(size, size);
            if sum > best_sum {
                best_index = index;
                best_size = size;
                best_sum = sum;
            }
        }
        (best_index, best_size)
    }
}

fn power_level(x: usize, y: usize, serial: usize) -> i8 {
    let rack_id = x + 10;
    let mut power = rack_id * y;
    power += serial;
    power *= rack_id;
    power /= 100;
    power %= 10;
    (power as i8) - 5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_level() {
        assert_eq!(power_level(3, 5, 8), 4);
        assert_eq!(power_level(122, 79, 57), -5);
        assert_eq!(power_level(217, 196, 39), 0);
        assert_eq!(power_level(101, 153, 71), 4);
    }

    #[test]
    fn test_find_largest_square() {
        let fc = FuelCells::new(18);
        assert_eq!(fc.find_largest_square(3, 3), ((33, 45), 29));

        let fc = FuelCells::new(42);
        assert_eq!(fc.find_largest_square(3, 3), ((21, 61), 30));
    }

    #[test] #[ignore]
    fn test_find_largest_total_square() {
        let fc = FuelCells::new(18);
        assert_eq!(fc.find_largest_total_square(), ((90, 269), 16));

        let fc = FuelCells::new(42);
        assert_eq!(fc.find_largest_total_square(), ((231, 251), 12));
    }
}
