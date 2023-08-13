use csp::{ConstraintProvider, DomainProvider};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

/// Tracking state for the search algorithm to undo itself.
struct BacktrackState<V, D, DP> {
    variable: V,
    domains: DP,
    untested: Vec<D>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BacktrackResult {
    Consistent,
    NoSolution,
}

// TODO: Should this be "externalized"? Like is there any reason to run CSP outside?
#[must_use]
pub fn backtrack_reduce<V, D, DP, CP>(
    domains: &mut DP,
    arcs: &mut [(V, V)],
    constraints: &CP,
    rng: &mut SmallRng,
) -> BacktrackResult
where
    V: PartialEq + Copy,
    D: PartialEq + Copy,
    DP: DomainProvider<V, D>,
    CP: ConstraintProvider<V, D>,
{
    let mut backtrack: Vec<BacktrackState<V, D, DP>> = vec![];

    loop {
        // First, make domains consistent.
        csp::ac3(domains, arcs, constraints);

        // Second, search for the most constrained unsolved variable and try to choose a value for it.
        if let Some(v) = domains.next_reducable_variable() {
            let domains_clone = domains.clone();
            // TODO: Unwrap :((
            if let Some(mut reducable) = domains.take_domain(&v) {
                // Use a random index for selecting the answer, and store the selection.
                if let Some(selected) = reducable.choose(rng).copied() {
                    backtrack.push(BacktrackState {
                        variable: v,
                        domains: domains_clone,
                        untested: reducable
                            .iter()
                            .filter(|dv| selected == **dv)
                            .copied()
                            .collect(),
                    });
                    reducable.retain(|dv| *dv == selected);
                    domains.update_domain(&v, reducable);
                }
            }
            // print_domain_counts(&domains, y_lim, x_lim);
        }
        // Alternatively, if there are no viable selections to be made
        else if !domains.is_consistent() {
            let Some(mut prev) = backtrack.pop() else {
                return BacktrackResult::NoSolution;
            };

            // Choose from the untested values, if there are any.
            if let Some(selected) = prev.untested.choose(rng).copied() {
                prev.untested.retain(|dv| *dv != selected);
                domains.clone_from(&prev.domains);
                if let Some(mut domain_values) = domains.take_domain(&prev.variable) {
                    domain_values.retain(|dv| *dv == selected);
                    domains.update_domain(&prev.variable, domain_values);
                }
                backtrack.push(prev);
            }
            // If there are no untested values, go back to the top of the backtrack and reset the domain.
            else {
                domains.clone_from(&prev.domains);
            }
        }
        // Finally, if we've exhausted all reducable variables, and everything is consistent, we must have found a solution.
        else {
            return BacktrackResult::Consistent;
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use rand::rngs::SmallRng;
    use rand_seeder::Seeder;

    use crate::{
        backtrack::BacktrackResult,
        tile_matcher::{Coordinate, TileSet},
    };

    use super::backtrack_reduce;

    fn simple_rng(seed_str: &str) -> SmallRng {
        Seeder::from(seed_str).make_rng()
    }

    struct TestCase {
        tiles: TileSet,
        arcs: Vec<(Coordinate, Coordinate)>,
        domains: HashMap<Coordinate, Vec<usize>>,
    }

    fn build_domain(x_lim: usize, y_lim: usize) -> TestCase {
        let arcs = crate::build_arcs(x_lim, y_lim);
        let tiles = TileSet::new();
        let starting_domain = (0..tiles.tiles.len()).collect::<Vec<_>>();
        let mut domains = HashMap::new();
        for x in 0..x_lim {
            for y in 0..y_lim {
                domains.insert(Coordinate::new(x, y), starting_domain.clone());
            }
        }

        TestCase {
            tiles,
            arcs,
            domains,
        }
    }

    /// Turn 2d coordinates into a 1d space. Assumes a reduced domain set, will panic if inconsistent or give pointless results if not reduced...
    fn reduced_domains(domains: HashMap<Coordinate, Vec<usize>>) -> Vec<usize> {
        let mut domains = domains.iter().collect::<Vec<_>>();
        domains.sort_by(|a, b| a.0.x.cmp(&b.0.x).then_with(|| a.0.y.cmp(&b.0.y)));
        domains.into_iter().map(|(_, v)| v[0]).collect()
    }

    #[test]
    fn backtrack_1() {
        let mut rng = simple_rng("hello world oasdf");
        let TestCase {
            tiles,
            mut arcs,
            mut domains,
        } = build_domain(5, 5);

        assert_eq!(
            BacktrackResult::Consistent,
            backtrack_reduce(&mut domains, &mut arcs, &tiles, &mut rng)
        );
    }

    #[test]
    fn backtrack_inconsistent() {
        // TODO: This test makes wild assumptions about the tileset. It's basic
        // intention is to exercise a bit of the backtracking logic since I am
        // not sure the best way to exercise it programatically...
        let mut rng = simple_rng("hello world oasdf");
        let TestCase {
            tiles,
            mut arcs,
            mut domains,
        } = build_domain(5, 5);

        let reduction = backtrack_reduce(&mut domains, &mut arcs, &tiles, &mut rng);
        assert_eq!(reduction, BacktrackResult::Consistent);
        assert_eq!(
            vec!(3, 8, 5, 2, 8, 0, 8, 5, 2, 8, 1, 7, 9, 3, 7, 2, 0, 6, 4, 1, 3, 0, 8, 5, 2),
            reduced_domains(domains),
        );
    }
}
