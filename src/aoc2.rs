use std::io;
use std::collections::BTreeMap;
use std::io::BufRead;
use failure::Error;

pub fn aoc2(part2: bool) {
    // This let binding is needed for stdin to live long enough
    let stdin = io::stdin();
    if part2 {
        println!("Common letters: {}", run_part2(&mut stdin.lock()).expect("Encountered error"));
    } else {
        println!("Checksum: {}", run_part1(&mut stdin.lock()));
    }
}

fn run_part1(input: &mut BufRead) -> u64 {
    let box_ids = input.lines().collect::<Result<Vec<_>, _>>().expect("Can't read input");
    checksum_boxes(&box_ids)
}

fn run_part2(input: &mut BufRead) -> Result<String, Error> {
    let box_ids = input.lines().collect::<Result<Vec<_>, _>>()?;
    let closest_boxes = find_closest_boxes(&box_ids).expect("No close boxes found");
    Ok(find_common_letters(&closest_boxes))
}

fn find_closest_boxes(box_ids: &Vec<String>) -> Option<(String, String)> {
    let desired_length = box_ids[0].len() - 1;
    // O(k*n^2) cause we dumb
    for box_id1 in box_ids {
        for box_id2 in box_ids {
            if find_common_letters(&(box_id1.to_string(), box_id2.to_string())).len() == desired_length {
                return Some((box_id1.to_string(), box_id2.to_string()))
            }
        }
    }
    None
}

fn find_common_letters(box_pair: &(String, String)) -> String {
    let box1_chars: Vec<_> = box_pair.0.chars().collect();
    let box2_chars: Vec<_> = box_pair.1.chars().collect();
    let mut common = String::new();
    assert!(box1_chars.len() == box2_chars.len());
    box1_chars.iter()
        .zip(box2_chars.iter())
        .filter(|(c1, c2)| c1 == c2)
        .map(|(c1, _)| c1)
        .collect()
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

    #[test]
    fn test_find_closest_boxes() {
        let input: Vec<String> = vec![
            "abcde",
            "fghij",
            "klmno",
            "pqrst",
            "fguij",
            "axcye",
            "wvxyz",
        ].iter().map(|s| s.to_string()).collect();
        assert_eq!(find_closest_boxes(&input), Some(("fghij".to_string(), "fguij".to_string())));
    }

    #[test]
    fn test_common_letters() {
        let pair: (String, String) = (
            "fghij".to_string(),
            "fguij".to_string(),
        );
        assert_eq!(find_common_letters(&pair), "fgij");
    }
}
