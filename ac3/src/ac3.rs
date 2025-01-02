use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::variable_provider::{Variable, VariableID, VariableProvider};

pub trait DomainType: Clone + PartialEq + Copy + Debug + Display {}
pub trait IdentifierType: Eq + PartialEq + Hash + Ord + Copy + Clone + Display + Debug {}

/// Iterate `x`'s remaining domain values, and keep any that satisfy available constraints.
fn retain<D, K, CP>(x: &Variable<D, K>, y: &Variable<D, K>, constraints: &CP) -> bool
where
    D: DomainType,
    CP: ConstraintProvider<D, K>,
    K: IdentifierType,
{
    let mut revised = false;

    x.retain(|x_value| {
        let satisfies = y
            .possible_values()
            .iter()
            .any(|y_value| constraints.check(x, x_value, y, y_value));
        if !satisfies {
            revised = true;
        }
        satisfies
    });

    revised
}

/// Allows users to provide a mechanism to validate binary constraints between
/// two variables and their value.
pub trait ConstraintProvider<D, K>
where
    D: DomainType,
    K: Eq + PartialEq + Hash + Ord + Copy + Clone,
{
    /// Determine if variable a has a valid relationship with b based on their
    /// identity and value.
    fn check(&self, a: &Variable<D, K>, a_value: &D, b: &Variable<D, K>, b_value: &D) -> bool;
}

/// Utility type for making boxes a little simpler. Probably should be removed
/// from public API as it is only really needed for the [`HashMap`] implementation
/// of [`ConstraintProvider`].
pub type Constraint<D> = Box<dyn Fn(&D, &D) -> bool>;

/// Utility function for making [`Constraint`]s.
pub fn new_constraint<D>(f: impl Fn(&D, &D) -> bool + 'static) -> Constraint<D>
where
    D: DomainType,
{
    Box::new(f)
}

/// Removes invalid domain values from a given variable `x`, by verifying
/// constraints in relation to `y`.
fn revise<K, D, CP>(
    variables: &VariableProvider<D, K>,
    constraints: &CP,
    x: VariableID,
    y: VariableID,
) -> bool
where
    D: DomainType,
    CP: ConstraintProvider<D, K>,
    K: IdentifierType,
{
    let mut revised = false;

    if let Some(x_var) = variables.get_var(x) {
        if let Some(y_var) = variables.get_var(y) {
            if retain(x_var, y_var, constraints) {
                revised = true;
            }
        }
    }

    revised
}

/// Entrypoint for a very basic version of [AC-3](https://en.wikipedia.org/wiki/AC-3_algorithm).
///
/// Callers must provide:
///
/// - a [`VariableProvider`] that will be mutated, reducing the possible values for each variable.
/// - a [`ConstraintProvider`] that provides the rules for validating the relationships between variables
/// - a collection of pairs of variables that are used to indicate which variables are related to each other.
///
/// # Example
///
/// See [`test::validate_ac3`].
/// ```
pub fn ac3<K, D, CP>(
    variables: &mut VariableProvider<D, K>,
    arcs: &[(VariableID, VariableID)],
    constraints: &CP,
) where
    D: DomainType,
    K: IdentifierType,
    CP: ConstraintProvider<D, K>,
{
    let mut queue = arcs.iter().copied().collect::<VecDeque<_>>();

    while let Some((x, y)) = queue.pop_front() {
        let revised = revise(variables, constraints, x, y);

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
    fn validate_ac3() {
        let mut variables = VariableProvider::from([('a', vec![1, 2, 3]), ('b', vec![1, 2, 3])]);
        let a = variables.find_id('a').unwrap();
        let b = variables.find_id('b').unwrap();
        let constraints = HashMap::from([
            ((a, b), new_constraint(|a, b| a == b && *a < 2)),
            ((b, a), new_constraint(|a, b| a == b && *b < 2)),
        ]);
        let arcs = vec![(a, b), (b, a)];
        ac3(&mut variables, &arcs, &constraints);
        assert!(variables.get_var(a).unwrap().possible_values().eq(&vec!(1)));
        assert!(variables.get_var(b).unwrap().possible_values().eq(&vec!(1)));
    }

    #[test]
    fn revise_shrinks_domain_based_on_constraints() {
        let variables = VariableProvider::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);
        let a = variables.find_id('a').unwrap();
        let b = variables.find_id('b').unwrap();

        let constraints = HashMap::from([((a, b), new_constraint(|a, _| *a < 3))]);

        assert!(revise(&variables, &constraints, a, b));
        assert!(variables
            .get_var(a)
            .unwrap()
            .possible_values()
            .eq(&vec!(1, 2)));
        assert!(variables
            .get_var(b)
            .unwrap()
            .possible_values()
            .eq(&vec!(2, 3)));
    }

    #[test]
    fn revise_leaves_domain_unmodified_if_all_constraints_valid() {
        let variables = VariableProvider::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);

        let a = variables.find_id('a').unwrap();
        let b = variables.find_id('b').unwrap();

        let constraints = HashMap::from([((a, b), new_constraint(|a, _| *a < 5))]);

        assert!(!revise(&variables, &constraints, a, b));
        assert!(variables
            .get_var(a)
            .unwrap()
            .possible_values()
            .eq(&vec!(1, 2, 3)));
        assert!(variables
            .get_var(b)
            .unwrap()
            .possible_values()
            .eq(&vec!(2, 3)));
    }

    #[test]
    fn revise_does_not_change_domain_without_constraints() {
        let variables = VariableProvider::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);
        let constraints = HashMap::new();

        let a = variables.find_id('a').unwrap();
        let b = variables.find_id('b').unwrap();

        assert!(!revise(&variables, &constraints, a, b));
        assert!(variables
            .get_var(a)
            .unwrap()
            .possible_values()
            .eq(&vec!(1, 2, 3)));
        assert!(variables
            .get_var(b)
            .unwrap()
            .possible_values()
            .eq(&vec!(2, 3)));
    }

    #[test]
    fn revise_can_empty_domain_values() {
        let variables = VariableProvider::from([('a', vec![1, 2, 3]), ('b', vec![2, 3])]);

        let a = variables.find_id('a').unwrap();
        let b = variables.find_id('b').unwrap();

        let constraints = HashMap::from([((a, b), new_constraint(|_, _| false))]);

        assert!(revise(&variables, &constraints, a, b));
        assert!(
            !variables.get_var(a).unwrap().is_consistent(),
            "is consistent? {:#?}",
            variables.get_var(a)
        );
    }
}
