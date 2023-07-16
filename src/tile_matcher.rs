use crate::ConstraintProvider;
use core::fmt::Display;



/// Direction tracks relationships between tiles.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Direction {
    Down,
    Left,
    Right,
    Up,
}

/// Define a position on the 2d map.
/// Assume that **Down the screen** is lower Y -- The bottom of the screen y = 0.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Coordinate(usize, usize);

impl Coordinate {
    /// Make a coordinate
    pub fn new(x: usize, y: usize) -> Self {
        Self(x, y)
    }

    /// Returns the direction `other` is in relation to `self`.
    pub fn is_adjacent(&self, other: &Self) -> Option<Direction> {
        let x_adjacent =
            (self.0 > 0 && self.0 - 1 == other.0) || (self.0 < usize::MAX && self.0 + 1 == other.0);
        let y_adjacent =
            (self.1 > 0 && self.1 - 1 == other.1) || (self.1 < usize::MAX && self.1 + 1 == other.1);

        if !(x_adjacent ^ y_adjacent) {
            None
        } else if self.0 < other.0 {
            Some(Direction::Right)
        } else if self.0 > other.0 {
            Some(Direction::Left)
        } else if self.1 < other.1 {
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
    pub fn new() -> Self {
        // Tiles:
        // ░
        // ┌─┐
        // │▓│
        // └─┘
        //                           0    1    2    3    4    5    6    7
        let tiles: Vec<char> = vec!['░', '┌', '─', '┐', '│', '▓', '└', '┘'];
        // up down left right (where "up" means "this tile is below the provided one")
        let relations = vec![
            // 0
            new_relation(
                vec![0, 2, 6, 7],
                vec![0, 1, 2, 3],
                vec![0, 3, 4, 7],
                vec![0, 1, 4, 6],
            ),
            // 1 ┌
            new_relation(vec![0, 2, 6, 7], vec![4], vec![0, 4, 3, 7], vec![2]),
            // 2 ─
            new_relation(vec![0, 2, 5, 6, 7], vec![0, 1, 2, 3, 5,], vec![1, 6, 2], vec![2, 3, 7]),
            // 3 ┐
            new_relation(vec![0, 2, 6, 7], vec![4], vec![2], vec![0, 1, 4, 6]),
            // 4 │
            new_relation(vec![1, 3, 4], vec![4, 6, 7], vec![0, 5], vec![0, 5]),
            // 5 ▓
            new_relation(vec![2, 5], vec![2, 5], vec![4, 5], vec![4, 5]),
            // 6 └
            new_relation(vec![4], vec![0, 1, 2, 3], vec![0, 3, 4, 7], vec![2]),
            // 7 ┘
            new_relation(vec![4], vec![0, 1, 2, 3], vec![2], vec![]),
        ];

        Self { tiles, relations }
    }

    pub fn debug(&self) {
        for (i, t) in self.tiles.iter().enumerate() {
            let rels = &self.relations[i];
            println!("Tile {i}: {t}");
            let up = &rels[0];
            let down = &rels[1];
            let left = &rels[2];
            let right = &rels[3];
            for o in up.iter() {
                println!(" up    {}", &self.tiles[*o]);
                println!("       {t}");
                println!("     xxx");
            }
            for o in down.iter() {
                println!("       {t}");
                println!(" down  {}", &self.tiles[*o]);
                println!("     xxx");
            }
            for o in left.iter() {
                println!(" left  {}{t}", &self.tiles[*o]);
                println!("     xxx");
            }
            for o in right.iter() {
                println!(" right {t}{}", &self.tiles[*o]);
                println!("     xxx");
            }
        }
    }
}

// up down left right
fn get_relation_index(dir: Direction) -> usize {
    match dir {
        Direction::Up => 0,
        Direction::Down => 1,
        Direction::Left => 2,
        Direction::Right => 3,
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {})", self.0, self.1))
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Up => f.write_str("up"),
            Direction::Down => f.write_str("down"),
            Direction::Left => f.write_str("left"),
            Direction::Right => f.write_str("right"),
        }
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
    use super::{Coordinate, Direction};

    #[test]
    fn coordinate_is_adjacent() {
        let c0_0 = Coordinate(0, 0);
        let c1_1 = Coordinate(1, 1);
        let c0_1 = Coordinate(0, 1);
        assert_eq!(c0_0.is_adjacent(&c1_1), None);
        assert_eq!(c0_0.is_adjacent(&c0_1), Some(Direction::Up));
        assert_eq!(c0_1.is_adjacent(&c0_0), Some(Direction::Down));
        assert_eq!(c0_1.is_adjacent(&c1_1), Some(Direction::Right));
        assert_eq!(c1_1.is_adjacent(&c0_1), Some(Direction::Left));
    }

    #[test]
    fn generate() {}
}
