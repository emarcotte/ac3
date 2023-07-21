use std::collections::HashMap;

use csp::ConstraintProvider;
use rand::prelude::SmallRng;
use rand_seeder::Seeder;
use tile_matcher::{Coordinate, TileSet};

mod tile_matcher;
mod backtrack;

fn build_arcs(x_lim: usize, y_lim: usize) -> Vec<(Coordinate, Coordinate)> {
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

/*
fn insert(domain: &mut HashMap<Coordinate, Vec<usize>>, bottom_left: Coordinate, tiles: Vec<Vec<usize>>) {
    
}
 */

fn main() {
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

    let mut arcs = build_arcs(x_lim, y_lim);

    backtrack::backtrack_reduce(&mut domains, &mut arcs, &tiles, &mut rng);

    print_domains(&domains, y_lim, x_lim, tiles);
}

fn print_domain_counts(domains: &HashMap<Coordinate, Vec<usize>>, y_lim: usize, x_lim: usize) {
    for y in (0..y_lim).rev() {
        print!("{y:>3} ");
        for x in 0..x_lim {
            let c = domains[&Coordinate::new(x, y)].len();
            print!("{c:>3}");
        }
        println!();
    }
}

fn print_domains(
    domains: &HashMap<Coordinate, Vec<usize>>,
    y_lim: usize,
    x_lim: usize,
    tiles: TileSet,
) {
    if domains.iter().all(|(_, tiles)| tiles.len() == 1) {
        println!("Result:");
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
