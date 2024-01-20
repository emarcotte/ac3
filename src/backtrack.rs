use crate::ac3::{ConstraintProvider, DomainProvider};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

/// Tracking state for the search algorithm to undo itself.
struct State<V, D, DP> {
    variable: V,
    domains: DP,
    untested: Vec<D>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Solution {
    Consistent,
    NoSolution,
}

// TODO: Should this be "externalized"? Like is there any reason to run CSP outside?
#[must_use]
pub fn reduce<V, D, DP, CP>(
    domains: &mut DP,
    arcs: &mut [(V, V)],
    constraints: &CP,
    rng: &mut SmallRng,
) -> Solution
where
    V: PartialEq + Copy,
    D: PartialEq + Copy,
    DP: DomainProvider<V, D>,
    CP: ConstraintProvider<V, D>,
{
    let mut backtrack: Vec<State<V, D, DP>> = vec![];

    loop {
        // First, make domains consistent.
        crate::ac3::ac3(domains, arcs, constraints);

        // Second, search for the most constrained unsolved variable and try to choose a value for it.
        if let Some(v) = domains.next_reducable_variable() {
            let domains_clone = domains.clone();
            if let Some(mut reducable) = domains.take_domain(&v) {
                // Use a random index for selecting the answer, and store the selection.
                if let Some(selected) = reducable.choose(rng).copied() {
                    backtrack.push(State {
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
        }
        // Alternatively, if there are no viable selections to be made
        else if !domains.is_consistent() {
            let Some(mut prev) = backtrack.pop() else {
                return Solution::NoSolution;
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
            return Solution::Consistent;
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, iter::once};

    use rand::rngs::SmallRng;
    use rand_seeder::Seeder;

    use crate::{ac3::ConstraintProvider, backtrack::Solution};

    use super::reduce;

    fn simple_rng(seed_str: &str) -> SmallRng {
        Seeder::from(seed_str).make_rng()
    }

    /// PigPen is an array, contraints are going to be:
    /// - 3 may be followed by 1 or 3.
    /// - 1 may be followed by 1, 3, or 5
    /// - 5 may be followed by 3 or 6.
    /// V = index
    /// D = u32
    struct PigPen(Vec<u32>);

    fn one_way_checks(a: usize, a_value: &u32, b: usize, b_value: &u32) -> bool {
        let a = a as isize;
        let b = b as isize;
        *a_value == 3 && b - a == 1 && [1, 3].contains(b_value)
            || *a_value == 1 && b - a == 1 && [1, 3, 5].contains(b_value)
            || *a_value == 5 && b - a == 1 && [3, 6].contains(b_value)
    }

    impl ConstraintProvider<usize, u32> for () {
        fn check(&self, a: usize, a_value: &u32, b: usize, b_value: &u32) -> bool {
            one_way_checks(a, a_value, b, b_value) || one_way_checks(b, b_value, a, a_value)
        }
    }

    impl From<HashMap<usize, Vec<u32>>> for PigPen {
        fn from(domains: HashMap<usize, Vec<u32>>) -> Self {
            let mut pen = PigPen(vec!(0; domains.len()));
            for (k, v) in domains {
                assert!(v.len() == 1);
                pen.0[k] = v[0];
            }
            pen
        }
    }

    struct TestCase {
        // relationships between items, by index.
        arcs: Vec<(usize, usize)>,
        // possible values for all items by index.
        domains: HashMap<usize, Vec<u32>>,
    }

    fn new_testcase(size: usize, values: std::ops::Range<u32>) -> TestCase {
        let a = (1..size).into_iter();
        let b = (0..size).into_iter();
        let arcs = a.zip(b)
            .flat_map(|(a, b)| once((a, b)).chain(once((b, a))))
            .collect();

        let starting_domain: Vec<u32> = values.collect();
        let mut domains = HashMap::new();
        for x in 0..size {
            domains.insert(x, starting_domain.clone());
        }

        TestCase { arcs, domains }
    }

    #[test]
    fn backtrack_1() {
        let mut rng = simple_rng("hello world oasdf");

        let TestCase {
            mut arcs,
            mut domains,
        } = new_testcase(4, 1..8);

        assert_eq!(
            Solution::Consistent,
            reduce(&mut domains, &mut &mut arcs, &(), &mut rng)
        );
        let pen = PigPen::from(domains);
        assert_eq!(
            vec!(5, 3, 1, 3),
            pen.0,
        );

    }

    #[test]
    fn backtrack_inconsistent() {
        let mut rng = simple_rng("test332");
        let TestCase {
            mut arcs,
            mut domains,
        } = new_testcase(5, 1..8);

        let reduction = reduce(&mut domains, &mut arcs, &(), &mut rng);
        assert_eq!(reduction, Solution::Consistent);
        let pen = PigPen::from(domains);
        assert_eq!(
            vec!(1, 1, 1, 1, 3),
            pen.0,
        );
    }
}
