use std::io;
use std::io::BufRead;
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Reverse;
use regex::Regex;
use failure::Error;

pub fn aoc7(part2: bool) -> Result<(), Error> {
    let dependencies = parse_dependency_graph(&mut io::stdin().lock())?;
    if part2 {
        println!("Seconds till completion: {}", seconds_to_completion(&dependencies, 60, 5));
    } else {
        println!("Topological sort: {}", toposort(&dependencies).unwrap().iter().collect::<String>());
    }
    Ok(())
}

fn parse_dependency_graph(input: &mut impl BufRead) -> Result<HashMap<char, Vec<char>>, Error> {
    let edge_regex = Regex::new(r"Step (.) must be finished before step (.) can begin.")?;
    let mut graph = HashMap::new();
    for line_res in input.lines() {
        let line = line_res?;
        let captures = edge_regex.captures(&line).unwrap();
        let required_step = captures.get(1).unwrap().as_str().chars().last().unwrap();
        let dependent_step = captures.get(2).unwrap().as_str().chars().last().unwrap();
        (*graph.entry(required_step).or_insert(vec![])).push(dependent_step);
    }
    Ok(graph)
}

fn toposort(graph: &HashMap<char, Vec<char>>) -> Result<Vec<char>, Error> {
    let mut incoming_edges: HashMap<char, u64> = HashMap::new();
    for dependents in graph.values() {
        for dependent in dependents {
            *incoming_edges.entry(*dependent).or_insert(0) += 1;
        }
    }

    let mut sorted = vec![];
    // Nodes that have all their dependencies met. We use a heap to
    // ensure we always get the alphabetically first character that is
    // ready.
    let mut ready_nodes: BinaryHeap<Reverse<char>> = graph.keys().filter(|n| !incoming_edges.contains_key(n)).map(|&r| Reverse(r)).collect();
    while ready_nodes.len() != 0 {
        let node = ready_nodes.pop().unwrap();
        sorted.push(node.0);
        for adjacency in graph.get(&node.0).unwrap_or(&vec![]) {
            *incoming_edges.get_mut(adjacency).unwrap() -= 1;
            if incoming_edges[adjacency] == 0 {
                ready_nodes.push(Reverse(*adjacency));
            }
        }
    }
    Ok(sorted)
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct WorkedNode {
    completion_time: u64,
    node: char,
}

impl WorkedNode {
    fn new(node: char, base_seconds: u64, cur_time: u64) -> Self {
        let a: u8 = "A".as_bytes()[0];
        let mut char: [u8; 4] = [0; 4];
        node.encode_utf8(&mut char);
        WorkedNode {
            node: node,
            completion_time: cur_time + base_seconds + u64::from(char[0] - a + 1),
        }
    }
}

fn seconds_to_completion(graph: &HashMap<char, Vec<char>>, base_seconds: u64, num_workers: usize) -> u64 {
    let mut incoming_edges: HashMap<char, u64> = HashMap::new();
    for dependents in graph.values() {
        for dependent in dependents {
            *incoming_edges.entry(*dependent).or_insert(0) += 1;
        }
    }
    let mut ready_nodes: BinaryHeap<Reverse<char>> = graph.keys().filter(|n| !incoming_edges.contains_key(n)).map(|&r| Reverse(r)).collect();
    let mut nodes_in_progress: BinaryHeap<Reverse<WorkedNode>> = BinaryHeap::new();
    let mut cur_time: u64 = 0;
    while nodes_in_progress.len() != 0 || ready_nodes.len() != 0 {
        if ready_nodes.len() > 0 && nodes_in_progress.len() < num_workers {
            // We can take a ready node and begin it.
            let node = ready_nodes.pop().unwrap();
            nodes_in_progress.push(Reverse(WorkedNode::new(node.0, base_seconds, cur_time.into())));
            continue;
        }
        if let Some(worked_node) = nodes_in_progress.pop() {
            let completion_time = u64::from(worked_node.0.completion_time);
            assert!(cur_time <= completion_time);
            cur_time = completion_time;
            for adjacency in graph.get(&worked_node.0.node).unwrap_or(&vec![]) {
                *incoming_edges.get_mut(adjacency).unwrap() -= 1;
                if incoming_edges[adjacency] == 0 {
                    ready_nodes.push(Reverse(*adjacency));
                }
            }
        }
    }
    cur_time
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
    fn test_sort_dependency_graph() {
        let steps = "Step C must be finished before step A can begin.
Step C must be finished before step F can begin.
Step A must be finished before step B can begin.
Step A must be finished before step D can begin.
Step B must be finished before step E can begin.
Step D must be finished before step E can begin.
Step F must be finished before step E can begin.";
        let graph = parse_dependency_graph(&mut steps.as_bytes()).expect("Parsing steps failed");
        assert_result_ok(toposort(&graph), vec!['C', 'A', 'B', 'D', 'F', 'E']);
    }

    #[test]
    fn test_seconds_to_completion() {
        let steps = "Step C must be finished before step A can begin.
Step C must be finished before step F can begin.
Step A must be finished before step B can begin.
Step A must be finished before step D can begin.
Step B must be finished before step E can begin.
Step D must be finished before step E can begin.
Step F must be finished before step E can begin.";
        let graph = parse_dependency_graph(&mut steps.as_bytes()).expect("Parsing steps failed");
        assert_eq!(seconds_to_completion(&graph, 0, 2), 15);
    }
}
