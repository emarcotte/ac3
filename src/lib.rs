#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
// Disallow mod.rs, its too confusing to see a bunch of mod.rs files in various tools.
#![forbid(clippy::mod_module_files)]

mod tile_matcher;

use std::{
    collections::{HashMap, VecDeque},
    hash::{BuildHasher, Hash},
};

/// Allows users to provide a mechanism to validate binary constraints between
/// two variables and their value.
pub trait ConstraintProvider<V, D> {
    /// Determine if variable a has a valid relationship with b based on their
    /// identity and value.
    fn check(&self, a: V, a_value: &D, b: V, b_value: &D) -> bool;
}

/// Utility type for making boxes a little simpler. Probably should be removed
/// from public API as it is only really needed for the [`HashMap`] implementation
/// of [`ConstraintProvider`].
pub type Constraint<D> = Box<dyn Fn(&D, &D) -> bool>;

/// Utility function for making [`Constraint`]s.
pub fn new_constraint<D>(f: impl Fn(&D, &D) -> bool + 'static) -> Constraint<D> {
    Box::new(f)
}

impl<V, D, S1> ConstraintProvider<V, D> for HashMap<(V, V), Constraint<D>, S1>
where
    V: Eq + PartialEq + Hash + Copy,
    S1: BuildHasher,
{
    fn check(&self, a: V, av: &D, b: V, bv: &D) -> bool {
        // TODO: Default is to be unconstrained, i guess.
        self.get(&(a, b))
            .map_or(true, |checker: &Constraint<D>| checker(av, bv))
    }
}

pub trait DomainProvider<V, D> {
    fn get_domain(&self, var: &V) -> Option<&Vec<D>>;
    fn take_domain(&mut self, var: &V) -> Option<Vec<D>>;
    fn update_domain(&mut self, var: &V, d: Vec<D>);
}

impl<K, D, S1> DomainProvider<K, D> for HashMap<K, Vec<D>, S1>
where
    K: Eq + PartialEq + Hash + Copy,
    S1: BuildHasher,
{
    fn get_domain(&self, var: &K) -> Option<&Vec<D>> {
        self.get(var)
    }

    fn take_domain(&mut self, var: &K) -> Option<Vec<D>> {
        self.remove(var)
    }

    fn update_domain(&mut self, var: &K, d: Vec<D>) {
        self.insert(*var, d);
    }
}

fn revise<V, D>(
    domains: &mut dyn DomainProvider<V, D>,
    constraints: &dyn ConstraintProvider<V, D>,
    x: V,
    y: V,
) -> bool
where
    V: Copy,
{
    let mut revised = false;

    // TODO: We could probably avoid the "safe" lookup and panic on invalid / unknown vars...
    if let Some(mut x_domain) = domains.take_domain(&x) {
        if let Some(y_domain) = domains.get_domain(&y) {
            x_domain.retain(|x_value| {
                let satisfies = y_domain
                    .iter()
                    .any(|y_value| constraints.check(x, x_value, y, y_value));
                if !satisfies {
                    revised = true;
                }
                satisfies
            });

            domains.update_domain(&x, x_domain);
        }
    }

    revised
}

/// Entrypoint for a very basic version of [AC-3](https://en.wikipedia.org/wiki/AC-3_algorithm).
///
/// Callers must provide:
///
/// - a [`DomainProvider`] that will be mutated, reducing the possible values for each variable.
/// - a [`ConstraintProvider`] that provides the rules for validating the relationships between variables
/// - a collection of pairs of variables that are used to indicate which variables are related to each other.
///
/// # Example
///
/// NOTE: This example uses the
///
/// ```
/// # use std::collections::HashMap;
/// let mut domains = HashMap::from([('a', vec![1, 2, 3]), ('b', vec![1, 2, 3])]);
/// let mut constraints = HashMap::from([
///     (('a', 'b'), csp::new_constraint(|a, b| a == b && *a < 2)),
///     (('b', 'a'), csp::new_constraint(|a, b| a == b && *b < 2)),
/// ]);
/// let arcs = vec![('a', 'b'), ('b', 'a')];
/// csp::ac3(&mut domains, &arcs, &constraints);
/// assert_eq!(domains.get(&'a'), Some(&vec!(1)));
/// assert_eq!(domains.get(&'b'), Some(&vec!(1)));
/// ```
pub fn ac3<V, D>(
    domains: &mut dyn DomainProvider<V, D>,
    arcs: &[(V, V)],
    constraints: &dyn ConstraintProvider<V, D>,
) where
    V: PartialEq + Eq + Copy,
{
    let mut queue = arcs.iter().copied().collect::<VecDeque<_>>();

    while let Some((x, y)) = queue.pop_front() {
        let revised = revise(domains, constraints, x, y);

        if revised {
            queue.extend(arcs.iter().filter(|(_, b)| b.eq(&x)));
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

        let constraints = HashMap::from([(('a', 'b'), new_constraint(|a, _| *a < 3))]);

        assert!(revise(&mut domains, &constraints, 'a', 'b'));
        assert_eq!(domains.get(&'a'), Some(&vec!(1, 2)));
        assert_eq!(domains.get(&'b'), Some(&vec!(2, 3)));
    }

    #[test]
    fn revise_leaves_domain_unmodified_if_all_constraints_valid() {
        let mut domains = HashMap::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);

        let constraints = HashMap::from([(('a', 'b'), new_constraint(|a, _| *a < 5))]);

        assert!(!revise(&mut domains, &constraints, 'a', 'b'));
        assert_eq!(domains.get(&'a'), Some(&vec!(1, 2, 3)));
        assert_eq!(domains.get(&'b'), Some(&vec!(2, 3)));
    }

    #[test]
    fn revise_does_not_change_domain_without_constraints() {
        let mut domains = HashMap::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);
        let constraints = HashMap::new();
        assert!(!revise(&mut domains, &constraints, 'x', 'y'));
        assert_eq!(domains.get(&'a'), Some(&vec!(1, 2, 3)));
        assert_eq!(domains.get(&'b'), Some(&vec!(2, 3)));
    }

    #[test]
    fn revise_can_empty_domain_values() {
        let mut domains = HashMap::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);
        let constraints = HashMap::from([(('a', 'b'), new_constraint(|_, _| false))]);

        assert!(revise(&mut domains, &constraints, 'a', 'b'));
    }
}
