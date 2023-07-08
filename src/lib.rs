#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
// Disallow mod.rs, its too confusing to see a bunch of mod.rs files in various tools.
#![forbid(clippy::mod_module_files)]

mod tile_matcher;

use std::{
    collections::{HashMap, VecDeque},
    hash::{BuildHasher, Hash},
};

type Constraint<D> = fn(D, D) -> bool;

fn revise<V, D, S1, S2>(
    domains: &mut HashMap<V, Vec<D>, S1>,
    constraints: &HashMap<(V, V), Constraint<D>, S2>,
    x: V,
    y: V,
) -> bool
where
    S1: BuildHasher,
    S2: BuildHasher,
    V: Hash + PartialEq + Eq + Copy,
    D: Copy,
{
    let mut revised = false;

    if let Some(constraints) = constraints.get(&(x, y)) {
        // Take x out of the map so we own it separately from a ref to
        // `domains`. This lets us mutate it while looking into another part of
        // `domains.
        let mut x_domain = domains.remove(&x).unwrap();
        let y_domain = domains.get(&y).unwrap();

        x_domain.retain(|x_value| {
            let satisfies = y_domain
                .iter()
                .any(|y_value| constraints(*x_value, *y_value));
            if !satisfies {
                revised = true;
            }
            satisfies
        });

        domains.insert(x, x_domain);
    }

    revised
}

pub fn ac3<V, D, S1, S2>(
    domains: &mut HashMap<V, Vec<D>, S1>,
    arcs: &VecDeque<(V, V)>,
    constraints: &HashMap<(V, V), Constraint<D>, S2>,
) where
    S1: BuildHasher,
    S2: BuildHasher,
    V: Hash + PartialEq + Eq + Copy,
    D: Copy,
{
    let mut queue = arcs.clone();

    while let Some((x, y)) = queue.pop_front() {
        let revised = revise(domains, constraints, x, y);

        if revised {
            queue.extend(arcs.iter().filter(|(_, b)| *b == x));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn revise_shrinks_domain_based_on_constraints() {
        let mut domains = HashMap::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);

        let mut constraints = HashMap::<(char, char), Constraint<i32>>::new();
        constraints.insert(('a', 'b'), |a: i32, _: i32| a < 3);

        assert!(revise(&mut domains, &constraints, 'a', 'b'));
        assert_eq!(domains.get(&'a'), Some(&vec!(1, 2)));
        assert_eq!(domains.get(&'b'), Some(&vec!(2, 3)));
    }

    #[test]
    fn revise_leaves_domain_unmodified_if_all_constraints_valid() {
        let mut domains = HashMap::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);

        let mut constraints = HashMap::<(char, char), Constraint<i32>>::new();
        constraints.insert(('a', 'b'), |a: i32, _: i32| a < 5);

        assert!(!revise(&mut domains, &constraints, 'a', 'b'));
        assert_eq!(domains.get(&'a'), Some(&vec!(1, 2, 3)));
        assert_eq!(domains.get(&'b'), Some(&vec!(2, 3)));
    }

    #[test]
    fn revise_does_not_change_domain_without_constraints() {
        let mut domains = HashMap::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);
        let constraints = HashMap::from([]);
        assert!(!revise(&mut domains, &constraints, 'x', 'y'));
        assert_eq!(domains.get(&'a'), Some(&vec!(1, 2, 3)));
        assert_eq!(domains.get(&'b'), Some(&vec!(2, 3)));
    }

    #[test]
    fn revise_can_empty_domain_values() {
        let mut domains = HashMap::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);
        let mut constraints = HashMap::<(char, char), fn(u32, u32) -> bool>::new();
        constraints.insert(('a', 'b'), |_, _| false);

        assert!(revise(&mut domains, &constraints, 'a', 'b'));
    }

    #[test]
    fn ac3_refines_matches() {
        let mut domains = HashMap::from([('a', vec![1, 2, 3]), ('b', vec![1, 2, 3])]);
        let mut constraints = HashMap::<(char, char), fn(u32, u32) -> bool>::new();
        constraints.insert(('a', 'b'), |a, b| a == b && a < 2);
        constraints.insert(('b', 'a'), |a, b| a == b && b < 2);
        let arcs = VecDeque::from([('a', 'b'), ('b', 'a')]);
        ac3(&mut domains, &arcs, &constraints);
        assert_eq!(domains.get(&'a'), Some(&vec!(1)));
        assert_eq!(domains.get(&'b'), Some(&vec!(1)));
    }
}
