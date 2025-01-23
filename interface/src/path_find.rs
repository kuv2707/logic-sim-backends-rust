use std::{
    cmp,
    collections::{BinaryHeap, HashMap, HashSet},
    mem::swap,
};

use crate::{
    consts::{WINDOW_HEIGHT, WINDOW_WIDTH},
    display_elems::{Screen, UnitArea},
};

type Cost = usize;
type Point = (i32, i32);
struct OrdPt {
    pos: Point,
    gcost: Cost,
    hcost: Cost,
}

impl OrdPt {
    fn neighbours(&self) -> Vec<Point> {
        let mut nbrs = Vec::new();
        for mov in [
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
        ] {
            let p = ((self.pos.0 as i32 + mov[0]), (self.pos.1 as i32 + mov[1]));
            if self.is_valid(p.0, p.1) {
                nbrs.push(p);
            }
        }

        // println!("{:?}", nbrs);
        nbrs
    }
    fn is_valid(&self, i: i32, j: i32) -> bool {
        i >= 0 && j >= 0 && i < (WINDOW_WIDTH as i32) && j < (WINDOW_HEIGHT as i32)
    }
    fn fcost(&self) -> Cost {
        return self.gcost + self.hcost;
    }
}

impl Ord for OrdPt {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let cmpres = self.fcost().cmp(&other.fcost());
        match cmpres {
            // if fcosts are same, compare by hcosts (dist. from end)
            cmp::Ordering::Equal => other.hcost.cmp(&self.hcost),
            _ => cmpres,
        }
    }
}

impl PartialOrd for OrdPt {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        other.fcost().partial_cmp(&self.fcost())
    }
}

impl Eq for OrdPt {}

impl PartialEq for OrdPt {
    fn eq(&self, other: &Self) -> bool {
        self.fcost() == other.fcost()
    }
}

pub fn a_star_get_pts(from: Point, to: Point, scr: &Screen) -> Vec<Point> {
    let mut open: BinaryHeap<OrdPt> = BinaryHeap::new();
    let mut closed: HashSet<Point> = HashSet::new();

    let mut parents: HashMap<Point, Point> = HashMap::new();
    let mut gh_costs: HashMap<Point, (Cost, Cost)> = HashMap::new();

    let mut pts = Vec::new();

    open.push(OrdPt {
        pos: from,
        gcost: 0,
        hcost: heuristic_cost(from, to),
    });
    while let Some(curr) = open.pop() {
        closed.insert(curr.pos);
        // println!("inspecting {:?}", curr.pos);
        if curr.pos.eq(&to) {
            let mut pos = &to;
            while *pos != from {
                pts.push((pos.0, pos.1));
                pos = parents.get(pos).unwrap();
            }
            pts.push(from);
            pts.reverse();
            break;
        }
        for nbp in curr.neighbours() {
            if closed.contains(&nbp) || scr[nbp.1 as usize][nbp.0 as usize] == UnitArea::Unvisitable
            {
                continue;
            }

            let movement_cost = curr.gcost + heuristic_cost(curr.pos, nbp);
            if movement_cost < gh_costs.get(&nbp).unwrap_or(&(Cost::MAX, Cost::MAX)).0 {
                let new_hcost = heuristic_cost(to, nbp);
                gh_costs.insert(nbp, (movement_cost, new_hcost));

                open.push(OrdPt {
                    pos: nbp,
                    gcost: movement_cost,
                    hcost: new_hcost,
                });
                parents.insert(nbp, curr.pos);
            }
        }
    }
    // println!("{:?} {:?} {} {} {:?}", from, to, scr.len(), scr[0].len(), pts);
    pts
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
    (x_diff * 30 + (y_diff - x_diff) * 10) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heuristic_cost() {
        assert_eq!(heuristic_cost((0, 0), (1, 1)), 14);
        assert_eq!(heuristic_cost((0, 0), (1, 0)), 10);
    }
    fn make_screen() -> Screen {
        [[UnitArea::VACANT; WINDOW_WIDTH as usize]; WINDOW_HEIGHT as usize]
    }
    #[test]
    fn test_path() {
        let mut s = make_screen();
        s[7][2] = UnitArea::Unvisitable;
        s[7][3] = UnitArea::Unvisitable;
        let pts = a_star_get_pts((14, 5), (2, 4), &s);
        println!("\n->{:?}", pts);
    }
}
