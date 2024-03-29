use std::collections::HashMap;

use csp::backtrack;
use csp::tile_matcher::{build_arcs, Coordinate, TileSet};
use rand::prelude::SmallRng;
use rand_seeder::Seeder;

/// Inserts some data into the map to pre-seed some interesting shapes.
pub fn insert(
    domain: &mut HashMap<Coordinate, Vec<usize>>,
    bottom_left: Coordinate,
    tiles: Vec<Vec<usize>>,
) {
    for (i_y, row) in tiles.iter().enumerate() {
        for (i_x, cell) in row.iter().enumerate() {
            domain.insert(
                Coordinate::new(bottom_left.x + i_x, bottom_left.y + i_y),
                vec![*cell],
            );
        }
    }
}

fn main() {
    // TODO: The hashmap next domain value is not stable/doesn't use RNG.
    let mut rng = simple_rng("hello world oasdf");

    let tiles = TileSet::new();
    let starting_domain = (0..tiles.tiles.len()).collect::<Vec<_>>();
    let x_lim = 80;
    let y_lim = 20;

    let mut domains = HashMap::new();
    for x in 0..x_lim {
        for y in 0..y_lim {
            domains.insert(Coordinate::new(x, y), starting_domain.clone());
        }
    }

    insert(
        &mut domains,
        Coordinate::new(5, 5),
        vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
        ],
    );
    insert(
        &mut domains,
        Coordinate::new(15, 5),
        vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
        ],
    );
    insert(
        &mut domains,
        Coordinate::new(25, 5),
        vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
        ],
    );
    insert(
        &mut domains,
        Coordinate::new(30, 5),
        vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
        ],
    );

    insert(
        &mut domains,
        Coordinate::new(5, 10),
        vec![
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        ],
    );

    insert(
        &mut domains,
        Coordinate::new(28, 11),
        vec![
            vec![
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            ],
            vec![
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            ],
            vec![
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            ],
            vec![
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            ],
            vec![
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            ],
            vec![
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            ],
            vec![
                5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            ],
        ],
    );

    let mut arcs = build_arcs(x_lim, y_lim);

    let happy = backtrack::reduce(&mut domains, &mut arcs, &tiles, &mut rng);
    match happy {
        backtrack::Solution::Consistent => {
            print_domains(&domains, y_lim, x_lim, tiles);
        }
        backtrack::Solution::NoSolution => {
            println!("No solution found");
        }
    };
}

fn print_domains(
    domains: &HashMap<Coordinate, Vec<usize>>,
    y_lim: usize,
    x_lim: usize,
    tiles: TileSet,
) {
    if domains.iter().all(|(_, tiles)| tiles.len() == 1) {
        println!("Solution:");
        for y in (0..y_lim).rev() {
            print!("{y:>3} ");
            for x in 0..x_lim {
                if let Some(v) = domains[&Coordinate::new(x, y)].first() {
                    print!("{}", tiles.tiles[*v]);
                } else {
                    print!("x");
                }
            }
            println!();
        }
    } else {
        println!("Sad face");
    }
}

fn simple_rng(seed_str: &str) -> SmallRng {
    Seeder::from(seed_str).make_rng()
}
