use std::io;
use std::collections::BTreeMap;
use std::io::BufRead;
use failure::Error;

pub fn aoc2(part2: bool) {
    // This let binding is needed for stdin to live long enough
    let stdin = io::stdin();
    if part2 {
        panic!("nope");
    } else {
        println!("Checksum: {}", part1(&mut stdin.lock()));
    }
}

fn part1(input: &mut BufRead) -> u64 {
    let box_ids = input.lines().collect::<Result<Vec<_>, _>>().expect("Can't read input");
    checksum_boxes(&box_ids)
}

fn contains_letter_k_times(str: &str, k: u64) -> bool {
    let counts = str.chars().fold(BTreeMap::new(), |mut a, c| { *a.entry(c).or_insert(0) += 1; a });
    counts.values().find(|i| **i == k).is_some()
}

fn checksum_boxes(box_ids: &Vec<String>) -> u64 {
    let two_count: u64 = box_ids.iter().map(|s| if contains_letter_k_times(s, 2) { 1 } else { 0 }).sum();
    let three_count: u64 = box_ids.iter().map(|s| if contains_letter_k_times(s, 3) { 1 } else { 0 }).sum();
    two_count * three_count
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
            Err(e) => panic!("got error result {}", e),
        }
    }

    #[test]
    fn test_contains_letter_k_times() {
        assert_eq!(contains_letter_k_times("abcdef", 2), false);
        assert_eq!(contains_letter_k_times("bababc", 2), true);
        assert_eq!(contains_letter_k_times("bababc", 3), true);
        assert_eq!(contains_letter_k_times("abbcde", 2), true);
        assert_eq!(contains_letter_k_times("abbcde", 3), false);
    }

    #[test]
    fn test_checksum_boxes() {
        let input: Vec<String> = vec![
            "abcdef",
            "bababc",
            "abbcde",
            "abcccd",
            "aabcdd",
            "abcdee",
            "ababab",
        ].iter().map(|s| s.to_string()).collect();
        assert_eq!(checksum_boxes(&input), 12);
    }
}
