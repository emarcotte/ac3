use std::collections::HashMap;

use csp::ConstraintProvider;
use rand::{prelude::SmallRng, seq::SliceRandom};
use rand_seeder::Seeder;
use tile_matcher::{Coordinate, TileSet};

mod tile_matcher;

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

struct BacktrackState<V, D> {
    variable: V,
    domains: HashMap<Coordinate, Vec<D>>,
    untested: Vec<D>,
}

fn main() {
    let mut rng = simple_rng("hello world oasdf");

    let tiles = TileSet::new();
    let starting_domain = (0..tiles.tiles.len()).collect::<Vec<_>>();
    let x_lim = 80;
    let y_lim = 20;
    tiles.debug();

    let mut domains = HashMap::new();
    for x in 0..x_lim {
        for y in 0..y_lim {
            domains.insert(Coordinate::new(x, y), starting_domain.clone());
        }
    }

    let arcs = build_arcs(x_lim, y_lim);
    let mut backtrack: Vec<BacktrackState<Coordinate, usize>> = vec![];

    loop {
        // First, make domains consistent.
        csp::ac3(&mut domains, &arcs, &tiles);

        // Second, search for the most constrained unsolved variable and try to choose a value for it.
        if let Some(v) = domains
            .iter_mut()
            .filter(|(_, v)| v.len() > 1)
            .min_by(|a, b| a.1.len().cmp(&b.1.len()))
            .map(|min| *min.0)
        {
            let domains_clone = domains.clone();
            // TODO: Unwrap :((
            let reducable = domains.get_mut(&v).unwrap();
            // Use a random index for selecting the answer, and store the selection.
            if let Some(selected) = reducable.choose(&mut rng).copied() {
                println!("Selecting and recording {} (depth {})", v, backtrack.len());
                backtrack.push(BacktrackState {
                    variable: v,
                    domains: domains_clone,
                    untested: reducable
                        .iter()
                        .copied()
                        .filter(|v| selected == *v)
                        .collect(),
                });
                reducable.retain(|dv| *dv == selected);
            }

            // print_domain_counts(&domains, y_lim, x_lim);
        }
        // Alternatively, if there are no viable selections to be made
        else if domains.iter().any(|(_, tiles)| tiles.is_empty()) {
            println!("Testing alternative..");
            let Some(mut prev) = backtrack.pop() else {
                panic!("No solution");
            };

            // Choose from the untested values, if there are any.
            if let Some(selected) = prev.untested.choose(&mut rng).copied() {
                prev.untested.retain(|v| *v != selected);
                println!("Choosing {selected} leaving {:?}", prev.untested);
                domains.clone_from(&prev.domains);
                domains
                    .entry(prev.variable)
                    .and_modify(|d| d.retain(|v| *v == selected));
                backtrack.push(prev);
            }
            // If there are no untested values, go back to the top of the backtrack and reset the domain.
            else {
                println!("Bubbling up");
                domains.clone_from(&prev.domains);
            }
        } else {
            println!("All done.");
            break;
        }
    }

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
