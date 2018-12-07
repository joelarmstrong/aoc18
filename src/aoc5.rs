use std::io;
use std::io::BufRead;
use failure::Error;

pub fn aoc5(part2: bool) -> Result<(), Error> {
    if part2 {
    } else {
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
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

    #[test]
    fn test_react() {
        assert_eq!(react("aA"), "");
        assert_eq!(react("abBA"), "");
        assert_eq!(react("abAB"), "abAB");
        assert_eq!(react("aabAAB"), "aabAAB");
        assert_eq!(react("dabAcCaCBAcCcaDA"), "dabCBAcaDA");
    }
}
