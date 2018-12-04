use std::io;
use std::collections::BTreeMap;
use std::io::BufRead;
use failure::Error;

pub fn aoc2(part2: bool) -> Result<(), Error> {
    // This let binding is needed for stdin to live long enough
    let stdin = io::stdin();
    if part2 {
        println!("Common letters: {}", run_part2(&mut stdin.lock())?);
    } else {
        println!("Checksum: {}", run_part1(&mut stdin.lock())?);
    }
    Ok(())
}

fn run_part1(input: &mut BufRead) -> Result<u64, Error> {
    let box_ids = input.lines().collect::<Result<Vec<_>, _>>()?;
    Ok(checksum_boxes(&box_ids))
}

fn run_part2(input: &mut BufRead) -> Result<String, Error> {
    let box_ids = input.lines().collect::<Result<Vec<_>, _>>()?;
    let closest_boxes = find_closest_boxes(&box_ids).expect("No close boxes found");
    Ok(find_common_letters(&closest_boxes))
}

/// Generic trie allowing k-mismatch search.
struct Trie<T: Ord + Copy> {
    root: TrieNode<T>,
}

struct TrieNode<T: Ord + Copy> {
    /// Children of this node within the trie. Using a B-tree should
    /// offer a good compromise between insane space usage (hash) and
    /// insane time usage (vector).
    children: BTreeMap<T, TrieNode<T>>,
    /// Value at this node. None iff the root node.
    value: Option<T>,
    /// Is this the end of some string in the set?
    end: bool,
}

impl<T: Ord + Copy> Trie<T> {
    fn new() -> Self {
        Trie {
            root: TrieNode {
                value: None,
                children: BTreeMap::new(),
                end: false,
            },
        }
    }

    /// Insert a single value into the trie.
    fn insert(&mut self, value: &[T]) {
        self.root.insert(&value);
    }

    /// Search for a value within the trie.
    #[allow(dead_code)]
    fn search(&self, value: &[T]) -> Option<Vec<T>> {
        self.search_allowing_mismatches(value, 0)
    }

    /// Search for a value within the try, allowing a certain number
    /// of mismatches. NB: insertions and deletions do not qualify as
    /// mismatches.
    fn search_allowing_mismatches(&self, value: &[T], mismatches: u32) -> Option<Vec<T>> {
        let mut found_value = vec![];
        if self.root.search_allowing_mismatches(value, mismatches, &mut found_value) {
            // We get the values in reverse order, because we append
            // on the way up from a successful search.
            found_value.reverse();
            Some(found_value)
        } else {
            None
        }
    }
}

impl<T: Ord + Copy> TrieNode<T> {
    fn new(value: T) -> Self {
        TrieNode {
            children: BTreeMap::new(),
            value: Some(value),
            end: false,
        }
    }

    /// Insert a string *below* this trie node (i.e. the string
    /// matches this node, but contains additional characters).
    fn insert(&mut self, remainder: &[T]) {
        if remainder.is_empty() {
            // Mark this node as being the end of a string in our set.
            self.end = true;
            return;
        }
        let next_value = remainder[0];
        let child = self.get_child(next_value);
        child.insert(&remainder[1..]);
    }

    /// Search for a value *below* this trie node. found_value will be
    /// filled in (in reverse order) if a value is found.
    fn search(&self, remainder: &[T], found_value: &mut Vec<T>) -> bool {
        if remainder.is_empty() {
            if self.end {
                if let Some(value) = self.value {
                    found_value.push(value);
                }
                return true;
            } else {
                return false;
            }
        }
        let next_value = remainder[0];
        if let Some(child) = self.children.get(&next_value) {
            if child.search_allowing_mismatches(&remainder[1..], 0, found_value) {
                if let Some(value) = self.value {
                    found_value.push(value);
                }
                return true;
            }
        }
        false
    }

    fn search_allowing_mismatches(&self, remainder: &[T], mismatches_left: u32, found_value: &mut Vec<T>) -> bool {
        if remainder.is_empty() {
            if self.end {
                if let Some(value) = self.value {
                    found_value.push(value);
                }
                return true;
            } else {
                return false;
            }
        }
        let next_value = remainder[0];
        if mismatches_left > 0 {
            // Recurse on all children, including mismatching ones. If
            // they mismatch, we recurse with a lower number of
            // allowed mismatches.
            for child in self.children.values() {
                let mut mismatches_next = mismatches_left;
                if child.value != Some(next_value) {
                    mismatches_next -= 1;
                }
                if child.search_allowing_mismatches(&remainder[1..], mismatches_next, found_value) {
                    if let Some(value) = self.value {
                        found_value.push(value);
                    }
                    return true;
                }
            }
            false
        } else {
            // No mismatches allowed anymore; fall back to the normal search.
            self.search(remainder, found_value)
        }
    }

    /// Get a mutable reference to the child node representing
    /// `value`, creating the node if needed.
    fn get_child(&mut self, value: T) -> &mut Self {
        &mut *self.children.entry(value).or_insert_with(|| TrieNode::new(value))
    }
}

/// Find a pair of "close" (only 1 letter different) box IDs, or None
/// if there are no close box IDs.
fn find_closest_boxes(box_ids: &[String]) -> Option<(String, String)> {
    let mut trie: Trie<char> = Trie::new();
    for box_id in box_ids {
        let box_chars: Vec<char> = box_id.chars().collect();
        if let Some(other_box_id) = trie.search_allowing_mismatches(&box_chars, 1) {
            return Some((box_id.to_string(), other_box_id.iter().collect()))
        }
        trie.insert(&box_chars)
    }
    None
}

/// Find which letters are shared in exactly the same position between
/// two strings.
fn find_common_letters(box_pair: &(String, String)) -> String {
    let box1_chars: Vec<_> = box_pair.0.chars().collect();
    let box2_chars: Vec<_> = box_pair.1.chars().collect();
    assert!(box1_chars.len() == box2_chars.len());
    box1_chars.iter()
        .zip(box2_chars.iter())
        .filter(|(c1, c2)| c1 == c2)
        .map(|(c1, _)| c1)
        .collect()
}

/// Check whether the string contains a letter repeated exactly k
/// times.
fn contains_letter_k_times(str: &str, k: u64) -> bool {
    let counts = str.chars().fold(BTreeMap::new(), |mut a, c| { *a.entry(c).or_insert(0) += 1; a });
    counts.values().any(|i| *i == k)
}

/// Calculate the checksum of box ids ((# of letters repeated twice) *
/// (# of letters repeated thrice)).
fn checksum_boxes(box_ids: &[String]) -> u64 {
    let two_count: u64 = box_ids.iter().map(|s| if contains_letter_k_times(s, 2) { 1 } else { 0 }).sum();
    let three_count: u64 = box_ids.iter().map(|s| if contains_letter_k_times(s, 3) { 1 } else { 0 }).sum();
    two_count * three_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_insert_and_search() {
        let mut trie: Trie<u8> = Trie::new();
        trie.insert(&[4, 5, 6, 1, 2]);
        trie.insert(&[5, 5, 2]);
        trie.insert(&[5, 5]);
        assert_eq!(trie.search(&[5]), None);
        assert_eq!(trie.search(&[5, 5]), Some(vec![5, 5]));
        assert_eq!(trie.search(&[5, 5, 2]), Some(vec![5, 5, 2]));
        assert_eq!(trie.search(&[5, 5, 2, 2]), None);
        assert_eq!(trie.search(&[4, 5, 6, 1, 2]), Some(vec![4, 5, 6, 1, 2]));
        assert_eq!(trie.search(&[]), None);
    }

    #[test]
    fn test_trie_mismatch_search() {
        let mut trie: Trie<u8> = Trie::new();
        trie.insert(&[4, 5, 6, 1, 2]);
        trie.insert(&[5, 5, 2]);
        trie.insert(&[5, 5]);
        assert_eq!(trie.search_allowing_mismatches(&[5], 0), None);
        assert_eq!(trie.search_allowing_mismatches(&[5, 5], 0), Some(vec![5, 5]));
        assert_eq!(trie.search_allowing_mismatches(&[5, 5, 2], 0), Some(vec![5, 5, 2]));
        assert_eq!(trie.search_allowing_mismatches(&[5, 5, 2, 2], 0), None);
        assert_eq!(trie.search_allowing_mismatches(&[4, 5, 6, 1, 2], 0), Some(vec![4, 5, 6, 1, 2]));
        assert_eq!(trie.search_allowing_mismatches(&[], 0), None);

        assert_eq!(trie.search_allowing_mismatches(&[5, 5, 3], 1), Some(vec![5, 5, 2]));
        assert_eq!(trie.search_allowing_mismatches(&[5, 5, 2], 1), Some(vec![5, 5, 2]));
        assert_eq!(trie.search_allowing_mismatches(&[5, 4, 2], 1), Some(vec![5, 5, 2]));
        assert_eq!(trie.search_allowing_mismatches(&[5, 4, 3], 1), None);
        assert_eq!(trie.search_allowing_mismatches(&[], 1), None);
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
        assert_eq!(find_closest_boxes(&input), Some(("fguij".to_string(), "fghij".to_string())));

        // Try one without any close boxes
        let input2: Vec<String> = vec![
            "abcde",
            "fghij",
            "klmno",
            "pqrst",
            "axcye",
            "wvxyz",
        ].iter().map(|s| s.to_string()).collect();
        assert_eq!(find_closest_boxes(&input2), None);
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
