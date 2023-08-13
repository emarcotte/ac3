use crate::ac3::ConstraintProvider;

/// Direction tracks relationships between tiles.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Direction {
    Down,
    Left,
    Right,
    Up,
}

impl Direction {
    fn reverse(self) -> Direction {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

/// Define a position on the 2d map.
/// Assume that **Down the screen** is lower Y -- The bottom of the screen y = 0.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

impl Ord for Coordinate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.x.cmp(&other.x).then(self.y.cmp(&other.y))
    }
}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Coordinate {
    /// Make a coordinate
    #[allow(dead_code)]
    #[must_use]
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Returns the direction `other` is in relation to `self`.
    #[must_use]
    pub fn is_adjacent(&self, other: &Self) -> Option<Direction> {
        let x_adjacent =
            (self.x > 0 && self.x - 1 == other.x) || (self.x < usize::MAX && self.x + 1 == other.x);
        let y_adjacent =
            (self.y > 0 && self.y - 1 == other.y) || (self.y < usize::MAX && self.y + 1 == other.y);

        if !(x_adjacent ^ y_adjacent) {
            None
        } else if self.x < other.x {
            Some(Direction::Right)
        } else if self.x > other.x {
            Some(Direction::Left)
        } else if self.y < other.y {
            Some(Direction::Up)
        } else {
            Some(Direction::Down)
        }
    }
}

/// Collection of tiles and valid relationships to each other.
pub struct TileSet {
    /// Visual reference for what each tile is.
    pub tiles: Vec<char>,

    // Tile index relation to other tile indexes by direction. E.g. `tiles`
    // index 0 is relations index 0, and has `[up, down, left, right]` where `up`
    // is a vec of indexes into `tiles`.
    pub relations: Vec<[Vec<usize>; 4]>,
}

/// Builds a list of bidirectional arcs between coordinates in a 2d grid.
#[must_use]
pub fn build_arcs(x_lim: usize, y_lim: usize) -> Vec<(Coordinate, Coordinate)> {
    let mut arcs = vec![];
    for x in 0..x_lim {
        for y in 0..y_lim {
            let base = Coordinate::new(x, y);
            if x < x_lim - 1 {
                arcs.push((base, Coordinate::new(x + 1, y)));
                arcs.push((Coordinate::new(x + 1, y), base));
            }
            if y < y_lim - 1 {
                arcs.push((base, Coordinate::new(x, y + 1)));
                arcs.push((Coordinate::new(x, y + 1), base));
            }
        }
    }

    arcs
}

fn new_relation(
    up: Vec<usize>,
    down: Vec<usize>,
    left: Vec<usize>,
    right: Vec<usize>,
) -> [Vec<usize>; 4] {
    [up, down, left, right]
}

impl Default for TileSet {
    fn default() -> Self {
        Self::new()
    }
}

impl TileSet {
    #[must_use]
    pub fn new() -> Self {
        // Tiles:
        // ░
        // ┌─┐
        // │▓│
        // └─┘
        //                           0    1    2    3    4    5    6    7     8    9
        let tiles: Vec<char> = vec!['░', '┌', '─', '┐', '│', '▓', '└', '┘', '─', '│'];
        // up down left right (where "up" means "this tile is below the provided one")
        let relations = tiles
            .iter()
            .map(|_| new_relation(vec![], vec![], vec![], vec![]))
            .collect();
        let mut new_tileset = Self { tiles, relations };

        new_tileset.make_box_only();

        new_tileset
    }

    /// Add a bi-directional relationship between from and to elements.
    fn add_relation(&mut self, dir: Direction, from: usize, to: &[usize]) -> &mut Self {
        let reverse = dir.reverse();
        for to in to {
            self.relations[from][get_relation_index(dir)].push(*to);
            self.relations[*to][get_relation_index(reverse)].push(from);
        }
        self
    }

    /// Set the tileset with basically a bunch of box drawing relationships.
    fn make_box_only(&mut self) {
        self.add_relation(Direction::Up, 0, &[0, 8, 6, 7])
            .add_relation(Direction::Left, 0, &[0, 3, 9, 7])
            .add_relation(Direction::Up, 1, &[0, 6, 7, 8])
            .add_relation(Direction::Left, 1, &[0, 3, 9, 7])
            .add_relation(Direction::Up, 2, &[0, 6, 7, 8])
            .add_relation(Direction::Left, 2, &[1, 2])
            .add_relation(Direction::Up, 3, &[0, 6, 7, 8])
            .add_relation(Direction::Left, 3, &[2])
            .add_relation(Direction::Up, 4, &[1, 4])
            .add_relation(Direction::Left, 4, &[0, 3, 9, 7])
            .add_relation(Direction::Up, 5, &[5, 2])
            .add_relation(Direction::Left, 5, &[5, 4])
            .add_relation(Direction::Up, 6, &[4])
            .add_relation(Direction::Left, 6, &[0, 3, 9, 7])
            .add_relation(Direction::Up, 7, &[9])
            .add_relation(Direction::Left, 7, &[8])
            .add_relation(Direction::Up, 8, &[5])
            .add_relation(Direction::Left, 8, &[8, 6])
            .add_relation(Direction::Up, 9, &[9, 3])
            .add_relation(Direction::Left, 9, &[5]);
    }
}

/// Utility for indexing into relations.
fn get_relation_index(dir: Direction) -> usize {
    match dir {
        Direction::Up => 0,
        Direction::Down => 1,
        Direction::Left => 2,
        Direction::Right => 3,
    }
}

impl ConstraintProvider<Coordinate, usize> for TileSet {
    fn check(&self, a: Coordinate, av: &usize, b: Coordinate, bv: &usize) -> bool {
        if let Some(dir) = a.is_adjacent(&b) {
            // TODO: Should consider making this safer.
            self.relations[*av][get_relation_index(dir)].contains(bv)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::ac3::ConstraintProvider;

    use super::{Coordinate, Direction, TileSet};

    #[test]
    fn cmp_coordinates() {
        assert_eq!(
            Ordering::Less,
            Coordinate::new(1, 1).cmp(&Coordinate::new(2, 2))
        );
    }

    #[test]
    fn coordinate_is_adjacent() {
        let c0_0 = Coordinate::new(0, 0);
        let c1_1 = Coordinate::new(1, 1);
        let c0_1 = Coordinate::new(0, 1);
        assert_eq!(c0_0.is_adjacent(&c1_1), None);
        assert_eq!(c0_0.is_adjacent(&c0_1), Some(Direction::Up));
        assert_eq!(c0_1.is_adjacent(&c0_0), Some(Direction::Down));
        assert_eq!(c0_1.is_adjacent(&c1_1), Some(Direction::Right));
        assert_eq!(c1_1.is_adjacent(&c0_1), Some(Direction::Left));
    }

    #[test]
    fn new_tileset() {
        let t = TileSet::default();
        assert_eq!(t.tiles.len(), 10);
    }

    #[test]
    fn check_ignores_unrelated_coordinates() {
        let t = TileSet::default();
        assert_eq!(t.tiles.len(), 10);
        assert!(!t.check(Coordinate::new(0, 0), &0, Coordinate::new(2, 0), &0));
    }

    #[test]
    fn check_related() {
        let t = TileSet::default();
        assert_eq!(t.tiles.len(), 10);
        assert!(t.check(Coordinate::new(0, 0), &0, Coordinate::new(1, 0), &0));
    }

    #[test]
    fn check_for_non_related() {
        let t = TileSet::default();
        assert_eq!(t.tiles.len(), 10);
        assert!(!t.check(Coordinate::new(0, 0), &0, Coordinate::new(1, 0), &5));
    }

    #[test]
    fn direction_reverse() {
        assert_eq!(Direction::Up.reverse(), Direction::Down);
        assert_eq!(Direction::Down.reverse(), Direction::Up);
        assert_eq!(Direction::Left.reverse(), Direction::Right);
        assert_eq!(Direction::Right.reverse(), Direction::Left);
    }
}
