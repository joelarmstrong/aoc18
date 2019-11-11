use disjoint_sets::UnionFind;
use std::io;
use std::io::BufRead;
use failure::Error;

pub fn aoc25(_part2: bool) -> Result<(), Error> {
    let points = parse_points(&mut io::stdin().lock())?;
    println!("Number of constellations: {}", num_constellations(&points));
    Ok(())
}

#[derive(Debug, PartialEq)]
struct Point4d {
    x: i32,
    y: i32,
    z: i32,
    w: i32,
}

fn parse_points(input: &mut impl BufRead) -> Result<Vec<Point4d>, Error>{
    let mut ret = vec![];
    for line_res in input.lines() {
        let line = line_res?.trim().to_string();
        if line.is_empty() {
            // Blank line.
            continue;
        }
        let values: Vec<i32> = line.split(",").map(|s| s.parse()).collect::<Result<Vec<_>, _>>()?;
        ret.push(Point4d { x: values[0], y: values[1], z: values[2], w: values[3] });
    }
    Ok(ret)
}

fn manhattan_distance(p1: &Point4d, p2: &Point4d) -> i32 {
    (p1.x - p2.x).abs() +
    (p1.y - p2.y).abs() +
    (p1.z - p2.z).abs() +
    (p1.w - p2.w).abs()
}

fn num_constellations(points: &[Point4d]) -> usize {
    let mut unionfind = UnionFind::new(points.len());
    for (i, point1) in points.iter().enumerate() {
        for (j, point2) in points[i + 1..].iter().enumerate() {
            if manhattan_distance(point1, point2) <= 3 {
                unionfind.union(i, j + i + 1);
            }
        }
    }
    let mut representatives = unionfind.to_vec();
    representatives.sort();
    representatives.dedup();
    representatives.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_points() -> Result<(), Error> {
        let input_str = "0,0,0,0
3,0,0,0
0,3,0,0
0,0,3,0
0,0,0,3
0,0,0,6
9,0,0,0
12,0,0,0
";
        assert_eq!(parse_points(&mut input_str.as_bytes())?, vec![
            Point4d { x: 0, y: 0, z: 0, w: 0 },
            Point4d { x: 3, y: 0, z: 0, w: 0 },
            Point4d { x: 0, y: 3, z: 0, w: 0 },
            Point4d { x: 0, y: 0, z: 3, w: 0 },
            Point4d { x: 0, y: 0, z: 0, w: 3 },
            Point4d { x: 0, y: 0, z: 0, w: 6 },
            Point4d { x: 9, y: 0, z: 0, w: 0 },
            Point4d { x: 12, y: 0, z: 0, w: 0 },
        ]);
        Ok(())
    }

    #[test]
    fn test_num_constellations() -> Result<(), Error> {
        let input_str = "0,0,0,0
3,0,0,0
0,3,0,0
0,0,3,0
0,0,0,3
0,0,0,6
9,0,0,0
12,0,0,0
";
        let points = parse_points(&mut input_str.as_bytes())?;
        assert_eq!(num_constellations(&points), 2);

        let input_str = "-1,2,2,0
0,0,2,-2
0,0,0,-2
-1,2,0,0
-2,-2,-2,2
3,0,2,-1
-1,3,2,2
-1,0,-1,0
0,2,1,-2
3,0,0,0
";
        let points = parse_points(&mut input_str.as_bytes())?;
        assert_eq!(num_constellations(&points), 4);

        let input_str = "1,-1,0,1
2,0,-1,0
3,2,-1,0
0,0,3,1
0,0,-1,-1
2,3,-2,0
-2,2,0,0
2,-2,0,-1
1,-1,0,-1
3,2,0,2
";
        let points = parse_points(&mut input_str.as_bytes())?;
        assert_eq!(num_constellations(&points), 3);

        let input_str = "1,-1,-1,-2
-2,-2,0,1
0,2,1,3
-2,3,-2,1
0,2,3,-2
-1,-1,1,-2
0,-2,-1,0
-2,2,3,-1
1,2,2,0
-1,-2,0,-2
";
        let points = parse_points(&mut input_str.as_bytes())?;
        assert_eq!(num_constellations(&points), 8);
        Ok(())
    }
}
