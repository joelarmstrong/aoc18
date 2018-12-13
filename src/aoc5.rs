use std::io;
use std::io::BufRead;
use std::collections::BTreeSet;
use failure::Error;

pub fn aoc5(part2: bool) -> Result<(), Error> {
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    if part2 {
        println!("After reacting and removing most problematic: {}", react_removing_most_problematic(&line.trim()).len());
    } else {
        println!("After reaction: {}", react(&line.trim()).len());
    }
    Ok(())
}

fn react(polymer: &str) -> String {
    let mut polymer_chars: Vec<char> = polymer.chars().collect();
    let mut i = 0;
    while i + 1 < polymer_chars.len() {
        let prev = polymer_chars[i];
        let next = polymer_chars[i + 1];
        if prev.to_ascii_lowercase() == next.to_ascii_lowercase() &&
           (prev.is_ascii_uppercase() && next.is_ascii_lowercase() ||
            prev.is_ascii_lowercase() && next.is_ascii_uppercase()) {
            // This is horribly inefficient
            polymer_chars.remove(i);
            polymer_chars.remove(i);
            i = i.checked_sub(1).unwrap_or(0);
        } else {
            i += 1;
        }
    }
    polymer_chars.into_iter().collect()
}

fn react_removing_most_problematic(polymer: &str) -> String {
    let new_polymer = react(polymer);
    let chars: BTreeSet<char> = new_polymer.chars().map(|c| c.to_ascii_lowercase()).collect();
    let less_problematic_polymers: Vec<String> = chars.iter()
        .map(|char_to_remove| {
            new_polymer.chars().filter(|c| c.to_ascii_lowercase() != *char_to_remove).collect::<String>()
        })
        .map(|s| react(&s))
        .collect();
    less_problematic_polymers.iter().min_by_key(|s| s.len()).unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_react() {
        assert_eq!(react("aA"), "");
        assert_eq!(react("abBA"), "");
        assert_eq!(react("abAB"), "abAB");
        assert_eq!(react("aabAAB"), "aabAAB");
        assert_eq!(react("dabAcCaCBAcCcaDA"), "dabCBAcaDA");
    }

    #[test]
    fn test_react_removing_most_problematic() {
        assert_eq!(react_removing_most_problematic("dabAcCaCBAcCcaDA"), "daDA");
    }
}
