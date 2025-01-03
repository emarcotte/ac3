use std::fmt::Display;

use ac3::{
    ac3::{ConstraintProvider, DomainType, IdentifierType},
    backtrack,
    variable_provider::{Variable, VariableID, VariableProvider},
};
use rand::prelude::{SeedableRng, SmallRng};
use rand_seeder::Seeder;

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

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum Tile {
    Inside,
    Outside,
    VWall,
    HWall,
    BLCorner,
    BRCorner,
    TLCorner,
    TRCorner,
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ch())
    }
}

impl DomainType for Tile {}
impl IdentifierType for Coordinate {}

// Tiles:
// ░
// ┌─┐
// │▓│
// └─┘
//  0    1    2    3    4    5    6    7     8    9
// '░', '┌', '─', '┐', '│', '▓', '└', '┘', '─', '│'
impl Tile {
    fn ch(&self) -> char {
        match self {
            Tile::Inside => '▓',
            Tile::Outside => '░',
            Tile::HWall => '─',
            Tile::VWall => '│',
            Tile::BLCorner => '└',
            Tile::BRCorner => '┘',
            Tile::TLCorner => '┌',
            Tile::TRCorner => '┐',
        }
    }

    // TODO: This sucks.
    fn len() -> usize {
        8
    }

    fn idx(self) -> usize {
        // TODO: Must be kept in sync with [`TileSet::new`].
        match self {
            Tile::Outside => 0,
            Tile::TLCorner => 1,
            Tile::HWall => 2,
            Tile::TRCorner => 3,
            Tile::VWall => 4,
            Tile::Inside => 5,
            Tile::BLCorner => 6,
            Tile::BRCorner => 7,
        }
    }
}

/// Collection of tiles and valid relationships to each other.
pub(crate) struct TileSet {
    /// Visual reference for what each tile is.
    pub tiles: Vec<Tile>,

    /// Tile index relation to other tile indexes by direction. E.g. `tiles`
    /// index 0 is relations index 0, and has `[up, down, left, right]` where `up`
    /// is a vec of indexes into `tiles`.
    pub relations: Vec<[Vec<bool>; 4]>,
}

/// Builds a list of bidirectional arcs between coordinates in a 2d grid.
#[must_use]
fn build_arcs(
    variables: &VariableProvider<Tile, Coordinate>,
    x_lim: usize,
    y_lim: usize,
) -> Vec<(VariableID, VariableID)> {
    let mut arcs = vec![];
    for x in 0..x_lim {
        for y in 0..y_lim {
            let base = variables.find_id(Coordinate::new(x, y)).unwrap();
            if x < x_lim - 1 {
                let other = variables.find_id(Coordinate::new(x + 1, y)).unwrap();
                arcs.push((base, other));
                arcs.push((other, base));
            }
            if y < y_lim - 1 {
                let other = variables.find_id(Coordinate::new(x, y + 1)).unwrap();
                arcs.push((base, other));
                arcs.push((other, base));
            }
        }
    }

    arcs
}

fn new_relation() -> [Vec<bool>; 4] {
    let tile_count = Tile::len();
    let relations = vec![false; tile_count];
    [
        relations.clone(),
        relations.clone(),
        relations.clone(),
        relations.clone(),
    ]
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
        //   0    1    2    3    4    5    6    7    8    9
        // ['░', '┌', '─', '┐', '│', '▓', '└', '┘', '─', '│'];
        let tiles = vec![
            Tile::Outside,
            Tile::TLCorner,
            Tile::HWall,
            Tile::TRCorner,
            Tile::VWall,
            Tile::Inside,
            Tile::BLCorner,
            Tile::BRCorner,
        ];

        // up down left right (where "up" means "this tile is below the provided one")
        let relations = tiles.iter().map(|_| new_relation()).collect();
        let mut new_tileset = Self { tiles, relations };

        new_tileset.add_relations();

        new_tileset
    }

    /// Add a bi-directional relationship between from and to elements.
    fn add_relation(&mut self, dir: Direction, from: Tile, to: &[Tile]) -> &mut Self {
        let reverse = dir.reverse();

        for to_tile in to {
            self.relations[from.idx()][get_relation_index(dir)][to_tile.idx()] = true;
            self.relations[to_tile.idx()][get_relation_index(reverse)][from.idx()] = true;
        }
        self
    }

    /// Set the tileset with basically a bunch of box drawing relationships.
    fn add_relations(&mut self) {
        self.add_relation(
            Direction::Left,
            Tile::BLCorner,
            &[Tile::Outside, Tile::Inside],
        )
        .add_relation(
            Direction::Left,
            Tile::BRCorner,
            &[Tile::HWall, Tile::TLCorner],
        )
        .add_relation(
            Direction::Left,
            Tile::HWall,
            &[Tile::HWall, Tile::BLCorner, Tile::TLCorner, Tile::HWall],
        )
        .add_relation(
            Direction::Left,
            Tile::Inside,
            &[Tile::Inside, Tile::VWall, Tile::TRCorner, Tile::BRCorner],
        )
        .add_relation(
            Direction::Left,
            Tile::Outside,
            &[Tile::Outside, Tile::TRCorner, Tile::VWall, Tile::BRCorner],
        )
        .add_relation(
            Direction::Left,
            Tile::TLCorner,
            &[Tile::Outside, Tile::Inside],
        )
        .add_relation(Direction::Left, Tile::TRCorner, &[Tile::HWall])
        .add_relation(Direction::Left, Tile::VWall, &[Tile::Inside, Tile::Outside])
        .add_relation(
            Direction::Up,
            Tile::BLCorner,
            &[Tile::VWall, Tile::TRCorner],
        )
        .add_relation(Direction::Up, Tile::BRCorner, &[Tile::VWall])
        .add_relation(Direction::Up, Tile::HWall, &[Tile::Inside, Tile::Outside])
        .add_relation(Direction::Up, Tile::Inside, &[Tile::Inside, Tile::HWall])
        .add_relation(
            Direction::Up,
            Tile::Outside,
            &[Tile::Outside, Tile::HWall, Tile::BLCorner, Tile::BRCorner],
        )
        .add_relation(
            Direction::Up,
            Tile::TLCorner,
            &[Tile::Outside, Tile::Inside],
        )
        .add_relation(
            Direction::Up,
            Tile::TRCorner,
            &[Tile::Outside, Tile::Inside],
        )
        .add_relation(
            Direction::Up,
            Tile::VWall,
            &[Tile::TLCorner, Tile::VWall, Tile::TRCorner],
        );
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

impl ConstraintProvider<Tile, Coordinate> for TileSet {
    fn check(
        &self,
        a: &Variable<Tile, Coordinate>,
        av: &Tile,
        b: &Variable<Tile, Coordinate>,
        bv: &Tile,
    ) -> bool {
        if let Some(dir) = a.identifier.is_adjacent(&b.identifier) {
            // TODO: Should consider making this safer.
            self.relations[av.idx()][get_relation_index(dir)][bv.idx()]
        } else {
            false
        }
    }
}

/// Inserts some data into the map to pre-seed some interesting shapes.
fn insert(
    variables: &mut VariableProvider<Tile, Coordinate>,
    bottom_left: Coordinate,
    tiles: Vec<Vec<Tile>>,
) {
    for (i_y, row) in tiles.iter().enumerate() {
        for (i_x, cell) in row.iter().enumerate() {
            if let Some(var) =
                variables.find_id(Coordinate::new(bottom_left.x + i_x, bottom_left.y + i_y))
            {
                variables.update_var(var, vec![*cell])
            }
        }
    }
}

fn main() {
    // TODO: The hashmap next domain value is not stable/doesn't use RNG.
    let mut rng = simple_rng("hello world o");

    let tiles = TileSet::new();
    let starting_domain = tiles.tiles.clone();
    let x_lim = 80;
    let y_lim = 20;

    let mut variables = VariableProvider::default();
    for x in 0..x_lim {
        for y in 0..y_lim {
            variables
                .add_var(Coordinate::new(x, y), starting_domain.clone())
                .unwrap();
        }
    }
    //
    // insert(
    //     &mut domains,
    //     Coordinate::new(5, 5),
    //     vec![
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //     ],
    // );
    // insert(
    //     &mut domains,
    //     Coordinate::new(15, 5),
    //     vec![
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //     ],
    // );
    // insert(
    //     &mut domains,
    //     Coordinate::new(25, 5),
    //     vec![
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //     ],
    // );
    // insert(
    //     &mut domains,
    //     Coordinate::new(30, 5),
    //     vec![
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //         vec![0, 0, 0, 0, 0],
    //     ],
    // );
    //
    // insert(
    //     &mut domains,
    //     Coordinate::new(5, 10),
    //     vec![
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //         vec![
    //             0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         ],
    //     ],
    // );
    //
    insert(
        &mut variables,
        Coordinate::new(28, 11),
        vec![
            vec![
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
            ],
            vec![
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
            ],
            vec![
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
            ],
            vec![
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
            ],
            vec![
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
            ],
            vec![
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
            ],
            vec![
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
                Tile::Inside,
            ],
        ],
    );
    //
    let mut arcs = build_arcs(&variables, x_lim, y_lim);

    let happy = backtrack::reduce(&mut variables, &mut arcs, &tiles, &mut rng);
    match happy {
        backtrack::Solution::Consistent => {
            print_domains(&variables, y_lim, x_lim);
        }
        backtrack::Solution::NoSolution => {
            println!("No solution found");
        }
    };
}

fn print_domains(variables: &VariableProvider<Tile, Coordinate>, y_lim: usize, x_lim: usize) {
    println!("Solution:");
    for y in (0..y_lim).rev() {
        print!("{y:>3} ");
        for x in 0..x_lim {
            let v = variables.find_var(Coordinate::new(x, y)).unwrap();
            if let Some(v) = v.possible_values().first() {
                print!("{}", v);
            } else {
                print!("x");
            }
        }
        println!();
    }
}

fn simple_rng(seed_str: &str) -> SmallRng {
    if seed_str.is_empty() {
        SmallRng::from_entropy()
    } else {
        Seeder::from(&seed_str).make_rng()
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use ac3::{ac3::ConstraintProvider, variable_provider::VariableProvider};

    use crate::Tile;

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
        // TODO: Bad test.
        assert!(t.relations.len() > 4);
    }

    #[test]
    fn check_ignores_unrelated_coordinates() {
        let t = TileSet::default();
        let mut vars = VariableProvider::<Tile, Coordinate>::default();
        let a = vars.add_var(Coordinate::new(0, 0), vec![]).unwrap();
        let b = vars.add_var(Coordinate::new(2, 0), vec![]).unwrap();
        assert!(!t.check(
            vars.get_var(a).unwrap(),
            &Tile::Inside,
            vars.get_var(b).unwrap(),
            &Tile::Inside
        ));
    }

    #[test]
    fn check_related() {
        let t = TileSet::default();
        let mut vars = VariableProvider::<Tile, Coordinate>::default();
        let a = vars
            .add_var(Coordinate::new(0, 0), vec![Tile::VWall, Tile::Outside])
            .unwrap();
        let b = vars
            .add_var(Coordinate::new(1, 0), vec![Tile::VWall, Tile::Outside])
            .unwrap();
        assert!(t.check(
            vars.get_var(a).unwrap(),
            &Tile::Outside,
            vars.get_var(b).unwrap(),
            &Tile::Outside
        ));
    }

    #[test]
    fn check_for_non_related() {
        let t = TileSet::default();
        let mut vars = VariableProvider::<Tile, Coordinate>::default();
        let a = vars.add_var(Coordinate::new(0, 0), vec![]).unwrap();
        let b = vars.add_var(Coordinate::new(1, 0), vec![]).unwrap();
        assert!(!t.check(
            vars.get_var(a).unwrap(),
            &Tile::Outside,
            vars.get_var(b).unwrap(),
            &Tile::Inside
        ));
    }

    #[test]
    fn direction_reverse() {
        assert_eq!(Direction::Up.reverse(), Direction::Down);
        assert_eq!(Direction::Down.reverse(), Direction::Up);
        assert_eq!(Direction::Left.reverse(), Direction::Right);
        assert_eq!(Direction::Right.reverse(), Direction::Left);
    }

    #[test]
    fn coordinate_display() {
        assert_eq!(format!("{}", Coordinate::new(0, 3)), "(0, 3)".to_string());
    }

    #[test]
    fn partial_cmp_coordinate() {
        assert_eq!(
            Coordinate::new(0, 1).partial_cmp(&Coordinate::new(1, 0)),
            Some(Ordering::Less)
        );
    }

    #[test]
    fn display_tile() {
        assert_eq!("▓".to_string(), format!("{}", Tile::Inside));
    }

    #[test]
    fn ch_tile() {
        for t in &[
            Tile::Outside,
            Tile::TLCorner,
            Tile::HWall,
            Tile::TRCorner,
            Tile::VWall,
            Tile::Inside,
            Tile::BLCorner,
            Tile::BRCorner,
        ] {
            // TODO: Bad test.
            t.ch();
        }
    }

    #[test]
    fn build_arcs() {
        let mut vars = VariableProvider::<Tile, Coordinate>::default();
        let x_lim = 3;
        let y_lim = 2;

        for x in 0..x_lim {
            for y in 0..y_lim {
                vars.add_var(Coordinate::new(x, y), vec![]).unwrap();
            }
        }

        let arcs = super::build_arcs(&vars, x_lim, y_lim);
        assert_eq!(arcs.len(), 14);
    }
}
