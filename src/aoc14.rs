use std::io;
use std::io::Read;
use failure::Error;

pub fn aoc14(part2: bool) -> Result<(), Error> {
    let mut input_str = String::new();
    io::stdin().lock().read_to_string(&mut input_str)?;
    let input: usize = input_str.trim().parse()?;
    let mut rs = RecipeScoreboard::new();
    if part2 {
        println!("Scores: {}", rs.recipes_before_digits_appear(input));
    } else {
        println!("Scores: {}", rs.scores_after_n_recipes(input).iter().map(|n| format!("{}", n)).collect::<String>());
    }
    Ok(())
}

struct RecipeScoreboard {
    recipes: Vec<u8>,
    elf_1: usize,
    elf_2: usize,
}

impl RecipeScoreboard {
    fn new() -> Self {
        Self {
            recipes: vec![3, 7],
            elf_1: 0,
            elf_2: 1,
        }
    }

    fn new_recipes(&self) -> Vec<u8> {
        let sum = self.recipes[self.elf_1] + self.recipes[self.elf_2];
        digits(sum as usize)
    }

    fn advance(&mut self) {
        self.recipes.append(&mut self.new_recipes());
        self.elf_1 = (self.elf_1 + (self.recipes[self.elf_1] as usize) + 1) % self.recipes.len();
        self.elf_2 = (self.elf_2 + (self.recipes[self.elf_2] as usize) + 1) % self.recipes.len();
    }

    fn scores_after_n_recipes(&mut self, steps: usize) -> &[u8] {
        while self.recipes.len() < steps + 10 {
            self.advance();
        }
        &self.recipes[steps..steps + 10]
    }

    fn recipes_before_digits_appear(&mut self, input: usize) -> usize {
        let digits = &digits(input)[..];
        let mut previous_recipes = None;
        while previous_recipes.is_none() {
            if self.recipes.len() >= digits.len() {
                let start = self.recipes.len() - digits.len();
                if &self.recipes[start..] == digits {
                    previous_recipes = Some(start)
                }
                let start = start.saturating_sub(1);
                if &self.recipes[start..self.recipes.len() - 1] == digits {
                    previous_recipes = Some(start)
                }
            }
            self.advance();
        }
        previous_recipes.unwrap()
    }
}

fn digits(n: usize) -> Vec<u8> {
    let mut digits = vec![];
    let mut num = n;
    while num >= 10 {
        digits.push((num % 10) as u8);
        num /= 10;
    }
    digits.push(num as u8);
    digits.reverse();
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digits() {
        assert_eq!(digits(10), vec![1, 0]);
        assert_eq!(digits(5), vec![5]);
        assert_eq!(digits(253), vec![2, 5, 3]);
    }

    #[test]
    fn test_scores_after_n_recipes() {
        let mut rs = RecipeScoreboard::new();
        assert_eq!(rs.scores_after_n_recipes(9), &[5, 1, 5, 8, 9, 1, 6, 7, 7, 9]);

        let mut rs = RecipeScoreboard::new();
        assert_eq!(rs.scores_after_n_recipes(5), &[0, 1, 2, 4, 5, 1, 5, 8, 9, 1]);

        let mut rs = RecipeScoreboard::new();
        assert_eq!(rs.scores_after_n_recipes(18), &[9, 2, 5, 1, 0, 7, 1, 0, 8, 5]);

        let mut rs = RecipeScoreboard::new();
        assert_eq!(rs.scores_after_n_recipes(2018), &[5, 9, 4, 1, 4, 2, 9, 8, 8, 2]);
    }

    #[test]
    fn test_recipes_before_digits_appear() {
        let mut rs = RecipeScoreboard::new();
        assert_eq!(rs.recipes_before_digits_appear(51589), 9);

        let mut rs = RecipeScoreboard::new();
        assert_eq!(rs.recipes_before_digits_appear(92510), 18);

        let mut rs = RecipeScoreboard::new();
        assert_eq!(rs.recipes_before_digits_appear(59414), 2018);
    }
}
