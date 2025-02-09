use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    mem::swap,
};

use crate::display_elems::Screen;

type Cost = usize;
type Point = (i32, i32);

const NOWHERE: Point = (-1, -1);

const DIRS: [[i32; 2]; 8] = [
    // orthogonal dirs
    [-1, 0],
    [1, 0],
    [0, -1],
    [0, 1],
    // diagonal dirs
    [1, 1],
    [1, -1],
    [-1, 1],
    [-1, -1],
];

fn neighbours(p: &Point, scr: &Screen) -> Vec<Point> {
    let mut nbrs = Vec::new();
    for mov in DIRS {
        let p = ((p.0 as i32 + mov[0]), (p.1 as i32 + mov[1]));
        if is_valid(p.0, p.1, scr) {
            nbrs.push(p);
        }
    }
    nbrs
}
fn is_valid(i: i32, j: i32, scr: &Screen) -> bool {
    i >= 0 && j >= 0 && i < (scr.logical_width() as i32) && j < (scr.logical_height() as i32)
}

pub fn a_star_get_pts(from: Point, to: Point, scr: &Screen) -> Vec<Point> {
    let mut frontier = BinaryHeap::new();
    let mut visited: HashSet<Point> = HashSet::new();

    let mut prev_loc: HashMap<Point, Point> = HashMap::new();

    // this cost is the one from start till the point
    // the heuristic cost from the point to the end is only used in
    // prioritising the next node to explore, but the cost of a node
    // only depends on its path from start till now.
    let mut g_costs_so_far: HashMap<Point, Cost> = HashMap::new();

    g_costs_so_far.insert(from, 0);
    frontier.push(Reverse((0, from)));
    prev_loc.insert(from, NOWHERE);

    while let Some(Reverse((_, curr_pt))) = frontier.pop() {
        visited.insert(curr_pt);
        if curr_pt.eq(&to) {
            let mut pts = Vec::new();
            let mut pos = &to;
            while *pos != NOWHERE {
                pts.push((pos.0, pos.1));
                pos = prev_loc.get(pos).unwrap();
            }
            return pts;
        }
        for nbp in neighbours(&curr_pt, scr) {
            if visited.contains(&nbp) {
                continue;
            }

            let movement_cost = g_costs_so_far.get(&curr_pt).unwrap()
                + heuristic_cost(curr_pt, nbp)
                + scr.weights[nbp.1 as usize][nbp.0 as usize];

            if movement_cost < *g_costs_so_far.get(&nbp).unwrap_or(&Cost::MAX) {
                let h_cost = heuristic_cost(to, nbp);
                g_costs_so_far.insert(nbp, movement_cost);

                frontier.push(Reverse((movement_cost + h_cost, nbp)));
                prev_loc.insert(nbp, curr_pt);
            }
        }
    }
    vec![]
}

fn heuristic_cost(from: Point, to: Point) -> Cost {
    let mut x_diff = (from.0 - to.0).abs();
    let mut y_diff = (from.1 - to.1).abs();
    if x_diff > y_diff {
        swap(&mut x_diff, &mut y_diff);
    }

    // we penalize diagonal movement by making its cost
    // 3x that of rectangular movement, to obtain a mostly
    // rectangular path. (c'est beau!)
    // (x_diff * 21 + (y_diff - x_diff) * 10) as usize
    (((x_diff.pow(2) + y_diff.pow(2)) as f64).sqrt() * 10000.0) as usize
}
