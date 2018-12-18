use std::io;
use std::io::BufRead;
use failure::{Error, format_err};

pub fn aoc8(part2: bool) -> Result<(), Error> {
    let tree = parse_tree(&mut io::stdin().lock())?;
    if part2 {
        println!("Value of root node: {}", tree_value(&tree));
    } else {
        println!("Sum of metadata entries: {}", sum_metadata_entries(&tree));
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
struct Tree {
    children: Vec<Tree>,
    metadata: Vec<u64>,
}

impl Tree {
    fn new(num_children: u64, num_metadata: u64) -> Self {
        Tree {
            children: Vec::with_capacity(num_children as usize),
            metadata: Vec::with_capacity(num_metadata as usize),
        }
    }
}

fn parse_tree(input: &mut BufRead) -> Result<Tree, Error> {
    let mut contents = String::new();
    input.read_to_string(&mut contents)?;
    let fields: Vec<u64> = contents.trim().split(' ').map(|s| s.parse::<u64>()).collect::<Result<_, _>>()?;
    let root = parse_node(&mut fields.into_iter())?;
    Ok(root)
}

fn parse_node(fields: &mut impl Iterator<Item = u64>) -> Result<Tree, Error> {
    let num_children = fields.next().ok_or_else(|| format_err!("ran out of fields too early"))?;
    let num_metadata = fields.next().ok_or_else(|| format_err!("ran out of fields too early"))?;
    let mut node = Tree::new(num_children, num_metadata);
    for _ in 0..num_children {
        node.children.push(parse_node(fields)?);
    }
    for _ in 0..num_metadata {
        let metadata = fields.next().ok_or_else(|| format_err!("ran out of fields too early"))?;
        node.metadata.push(metadata);
    }
    Ok(node)
}

fn sum_metadata_entries(tree: &Tree) -> u64 {
    let children_sum: u64 = tree.children.iter().map(sum_metadata_entries).sum();
    let my_sum: u64 = tree.metadata.iter().sum();
    children_sum + my_sum
}

fn tree_value(tree: &Tree) -> u64 {
    if tree.children.is_empty() {
        sum_metadata_entries(tree)
    } else {
        let mut sum = 0;
        for metadata in tree.metadata.iter() {
            let index = metadata.checked_sub(1);
            if let Some(child) = index.and(tree.children.get(index.unwrap() as usize)) {
                // We could be repeating work here, but who gives a bibble
                sum += tree_value(child);
            }
        }
        sum
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

    #[test]
    fn test_parse_tree() {
        let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
        assert_result_ok(parse_tree(&mut input.as_bytes()), Tree {
            children: vec![
                Tree {
                    children: vec![],
                    metadata: vec![10, 11, 12],
                },
                Tree {
                    children: vec![
                        Tree {children: vec![], metadata: vec![99]},
                    ],
                    metadata: vec![2],
                },
            ],
            metadata: vec![1, 1, 2],
        })
    }

    #[test]
    fn test_metadata_sum() {
        let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
        let tree = parse_tree(&mut input.as_bytes()).expect("Couldn't parse test tree");
        assert_eq!(sum_metadata_entries(&tree), 138);
    }

    #[test]
    fn test_tree_value() {
        let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
        let tree = parse_tree(&mut input.as_bytes()).expect("Couldn't parse test tree");
        assert_eq!(tree_value(&tree), 66);
    }
}
