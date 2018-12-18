use std::io;
use std::io::Read;
use std::cell::RefCell;
use std::rc::{Weak, Rc};
use regex::Regex;
use failure::{Error, format_err};

pub fn aoc9(part2: bool) -> Result<(), Error> {
    let mut line = String::new();
    io::stdin().lock().read_to_string(&mut line)?;
    let (num_players, num_marbles) = parse_game_settings(&line)?;
    let mut marbles = Marbles::new(num_players as usize);
    if part2 {
        marbles.play(num_marbles * 100);
    } else {
        marbles.play(num_marbles);
    }
    println!("Highest score: {}", marbles.highest_score());
    Ok(())
}

fn parse_game_settings(line: &str) -> Result<(u64, u64), Error> {
    let regex = Regex::new(r"([0-9]+) players; last marble is worth ([0-9]+) points")?;
    let captures = regex.captures(line).ok_or_else(|| format_err!("Unable to parse game settings"))?;
    let num_players: u64 = captures.get(1).ok_or_else(|| format_err!("Unable to parse game settings"))?.as_str().parse()?;
    let num_marbles: u64 = captures.get(2).ok_or_else(|| format_err!("Unable to parse game settings"))?.as_str().parse()?;
    Ok((num_players, num_marbles))
}

struct Node<T> {
    next: Option<Rc<RefCell<Node<T>>>>,
    prev: Option<Weak<RefCell<Node<T>>>>,
    value: T,
}

impl<T> Node<T> {
    fn new_rc(value: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            next: None,
            prev: None,
            value,
        }))
    }

    fn remove(&mut self) {
        if let Some(ref mut prev) = self.prev {
            prev.upgrade().unwrap().borrow_mut().next = self.next.clone();
        }
        if let Some(ref mut next) = self.next {
            next.borrow_mut().prev = self.prev.clone();
        }
    }

    fn insert(node: &mut Rc<RefCell<Node<T>>>, value: T) {
        let new_node = Node {
            next: node.borrow().next.clone(),
            prev: Some(Rc::downgrade(node)),
            value,
        };
        let new_rc = Rc::new(RefCell::new(new_node));
        let mut node_mut = node.borrow_mut();
        if let Some(ref mut next) = node_mut.next {
            if Rc::ptr_eq(next, node) {
                // This is a single-element circularized list. Have to
                // treat this specially to avoid mutably borrowing the
                // same node twice.
                node_mut.prev = Some(Rc::downgrade(&new_rc));
            } else {
                next.borrow_mut().prev = Some(Rc::downgrade(&new_rc));
            }
        }
        node_mut.next = Some(new_rc);
    }

    /// Circularize a *single-node* list.
    fn circularize(node: &mut Rc<RefCell<Node<T>>>) {
        node.borrow_mut().next = Some(node.clone());
        node.borrow_mut().prev = Some(Rc::downgrade(node));
    }

    fn iter(node: &mut Rc<RefCell<Node<T>>>) -> NodeIter<T> {
        NodeIter {
            cur_node: Some(node.clone()),
            direction: Direction::Right,
        }
    }

    fn reverse_iter(node: &mut Rc<RefCell<Node<T>>>) -> NodeIter<T> {
        NodeIter {
            cur_node: Some(node.clone()),
            direction: Direction::Left,
        }
    }
}

enum Direction {
    Right,
    Left,
}

struct NodeIter<T> {
    cur_node: Option<Rc<RefCell<Node<T>>>>,
    direction: Direction,
}

impl<T> Iterator for NodeIter<T> {
    type Item = Rc<RefCell<Node<T>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_node.is_some() {
            let node = self.cur_node.as_mut().unwrap().clone();
            let ret = Some(node.clone());
            self.cur_node = match &self.direction {
                Direction::Right => node.borrow_mut().next.clone(),
                Direction::Left => node.borrow_mut().prev.clone().and_then(|weak| weak.upgrade()),
            };
            ret
        } else {
            None
        }
    }
}

struct Marbles {
    num_marbles: u64,
    cur_node: Rc<RefCell<Node<u64>>>,
    num_players: usize,
    cur_player: usize,
    scores: Vec<u64>,
}

impl Marbles {
    fn new(num_players: usize) -> Self {
        let mut node = Node::new_rc(0);
        Node::circularize(&mut node);
        Marbles {
            cur_node: node,
            num_marbles: 1,
            num_players,
            cur_player: 0,
            scores: vec![0; num_players],
        }
    }

    fn place_next(&mut self) {
        if self.num_marbles % 23 == 0 {
            self.scores[self.cur_player] += self.num_marbles;
            let mut marbles = Node::reverse_iter(&mut self.cur_node).skip(6);
            let next_marble = marbles.next().unwrap();
            let marble_to_remove = marbles.next().unwrap();
            marble_to_remove.borrow_mut().remove();
            self.scores[self.cur_player] += marble_to_remove.borrow().value;
            self.cur_node = next_marble;
        } else {
            let mut iter = Node::iter(&mut self.cur_node);
            let mut insert_node = iter.nth(1).unwrap();
            Node::insert(&mut insert_node, self.num_marbles);
            self.cur_node = Node::iter(&mut insert_node).nth(1).unwrap();
        }
        self.num_marbles += 1;
        self.cur_player = (self.cur_player + 1) % self.num_players;
    }

    /// Play for the given number of turns.
    fn play(&mut self, num_turns: u64) {
        for _ in 0..num_turns {
            self.place_next();
        }
    }

    fn highest_score(&self) -> u64 {
        *self.scores.iter().max().unwrap()
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
    fn test_simple_linked_list() {
        let mut node_1 = Node::new_rc("foo");
        Node::insert(&mut node_1, "bar");
        Node::insert(&mut node_1, "baz");
        // Check that the values are in the order that we expect
        let node_iter = Node::iter(&mut node_1);
        let values: Vec<_> = node_iter.map(|n| n.borrow().value).collect();
        assert_eq!(values, vec!["foo", "baz", "bar"]);

        // Now add a value after the second node
        let mut node_2 = Node::iter(&mut node_1).skip(1).next().unwrap();
        Node::insert(&mut node_2, "quux");

        let values: Vec<_> = Node::iter(&mut node_1).map(|n| n.borrow().value).collect();
        assert_eq!(values, vec!["foo", "baz", "quux", "bar"]);

        // Try iterating backwards
        let values: Vec<_> = Node::reverse_iter(&mut node_2).map(|n| n.borrow().value).collect();
        assert_eq!(values, vec!["baz", "foo"]);

        node_2.borrow_mut().remove();

        let values: Vec<_> = Node::iter(&mut node_1).map(|n| n.borrow().value).collect();
        assert_eq!(values, vec!["foo", "quux", "bar"]);
    }

    #[test]
    fn test_circularized_list() {
        let mut node_1 = Node::new_rc("foo");
        Node::circularize(&mut node_1);
        let node_iter = Node::iter(&mut node_1);
        let values: Vec<_> = node_iter.take(5).map(|n| n.borrow().value).collect();
        assert_eq!(values, vec!["foo", "foo", "foo", "foo", "foo"]);

        // Insert a new value and verify it makes sense
        Node::insert(&mut node_1, "bar");
        let node_iter = Node::iter(&mut node_1);
        let values: Vec<_> = node_iter.take(5).map(|n| n.borrow().value).collect();
        assert_eq!(values, vec!["foo", "bar", "foo", "bar", "foo"]);

        // Remove a node and verify it's still a proper loop
        let node_2 = Node::iter(&mut node_1).skip(1).next().unwrap();
        node_2.borrow_mut().remove();
        let node_iter = Node::iter(&mut node_1);
        let values: Vec<_> = node_iter.take(5).map(|n| n.borrow().value).collect();
        assert_eq!(values, vec!["foo", "foo", "foo", "foo", "foo"]);
    }

    #[test]
    fn test_marble_game() {
        let mut marbles = Marbles::new(9);
        marbles.play(25);
        assert_eq!(marbles.highest_score(), 32);

        let mut marbles = Marbles::new(10);
        marbles.play(1618);
        assert_eq!(marbles.highest_score(), 8317);

        let mut marbles = Marbles::new(13);
        marbles.play(7999);
        assert_eq!(marbles.highest_score(), 146373);

        let mut marbles = Marbles::new(17);
        marbles.play(1104);
        assert_eq!(marbles.highest_score(), 2764);

        let mut marbles = Marbles::new(21);
        marbles.play(6111);
        assert_eq!(marbles.highest_score(), 54718);

        let mut marbles = Marbles::new(30);
        marbles.play(5807);
        assert_eq!(marbles.highest_score(), 37305);
    }

    #[test]
    fn test_parse_game_settings() {
        let line = "438 players; last marble is worth 71626 points\n";
        assert_result_ok(parse_game_settings(line), (438, 71626));
    }
}
