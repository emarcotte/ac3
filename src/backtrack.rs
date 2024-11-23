use crate::ac3::{ConstraintProvider, DomainType, IdentifierType};
use crate::variable_provider::{VariableID, VariableProvider};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

/// Tracking state for the search algorithm to undo itself.
#[derive(Debug)]
struct State<D, K> {
    variable_id: VariableID,
    variables: VariableProvider<D, K>,
    untested: Vec<D>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Solution {
    Consistent,
    NoSolution,
}

// TODO: Should this be "externalized"? Like is there any reason to run CSP outside?
// TODO: Ideally we'd only store the diff in the state for pop/push, not a whole copy.
#[must_use]
pub fn reduce<K, D, CP>(
    variables: &mut VariableProvider<D, K>,
    arcs: &mut [(VariableID, VariableID)],
    constraints: &CP,
    rng: &mut SmallRng,
) -> Solution
where
    K: IdentifierType,
    D: DomainType,
    CP: ConstraintProvider<D, K>,
{
    let mut backtrack: Vec<State<D, K>> = vec![];

    loop {
        // First, make domains consistent.
        crate::ac3::ac3(variables, arcs, constraints);

        // Second, search for the most constrained unsolved variable and try to choose a value for it.
        if let Some(v) = variables.next_reducable_variable() {
            let variables_clone = variables.clone();
            if let Some(reducable) = variables.get_var(v) {
                // Use a random index for selecting the answer, and store the selection.
                if let Some(selected) = reducable.choose(rng) {
                    let untested = reducable
                        .possible_values()
                        .iter()
                        .filter(|dv| selected != **dv)
                        .copied()
                        .collect();
                    backtrack.push(State {
                        variable_id: reducable.index,
                        variables: variables_clone,
                        untested,
                    });
                    reducable.retain(|dv| *dv == selected);
                } else {
                    println!("This should never happen");
                }
            } else {
                println!("This should not happen?");
            }
        }
        // Alternatively, if there are no viable selections to be made
        else if !variables.is_consistent() {
            let len = backtrack.len();
            let Some(mut prev) = backtrack.pop() else {
                return Solution::NoSolution;
            };

            // Choose from the untested values, if there are any.
            if let Some(selected) = prev.untested.choose(rng).copied() {
                prev.untested.retain(|dv| *dv != selected);
                if let Some(v) = variables.get_var(prev.variable_id) {
                    v.replace_possible_values(vec![selected]);
                }
                backtrack.push(prev);
            }
            // If there are no untested values, go back to the top of the backtrack and reset the domain.
            else if len > 1 {
                // TODO: setup top level as unfiltered so this is easier to steal prev
                // values without cloning whole solution space?
                variables.clone_from(&prev.variables);
            }
            // If we're popping the top of the backtrack stack, we're done. Cooked.
            else {
                return Solution::NoSolution;
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
    use rand::rngs::SmallRng;
    use rand_seeder::Seeder;

    use crate::{
        ac3::ConstraintProvider,
        backtrack::Solution,
        variable_provider::{Variable, VariableID, VariableProvider},
    };

    use super::reduce;

    fn simple_rng(seed_str: &str) -> SmallRng {
        Seeder::from(seed_str).make_rng()
    }

    struct TestCase {
        arcs: Vec<(VariableID, VariableID)>,
        variables: VariableProvider<char, i32>,
    }

    struct StaticConstraints {}
    impl ConstraintProvider<char, i32> for StaticConstraints {
        fn check(
            &self,
            a: &Variable<char, i32>,
            a_value: &char,
            b: &Variable<char, i32>,
            b_value: &char,
        ) -> bool {
            let m = if a.identifier % 2 == 0 && b.identifier % 2 == 0 {
                a_value == b_value
            } else {
                a_value != b_value
            };
            println!(
                "Does {}/{} vs {}/{} match? {}",
                a.identifier, a_value, b.identifier, b_value, m
            );
            m
        }
    }

    fn build_test_case() -> TestCase {
        let tiles = ('a'..='z').collect::<Vec<char>>();

        let mut variables = VariableProvider::default();
        let v1 = variables.add_var(0, tiles.clone()).unwrap();
        let v2 = variables.add_var(1, tiles.clone()).unwrap();
        let v3 = variables.add_var(2, tiles.clone()).unwrap();
        let v4 = variables.add_var(3, tiles.clone()).unwrap();
        let v5 = variables.add_var(4, tiles.clone()).unwrap();
        let v6 = variables.add_var(5, tiles.clone()).unwrap();

        // assume its adjacency like:
        // v1 v2
        // v3 v4
        // v5 v6
        let arcs = vec![
            (v1, v2),
            (v1, v3),
            (v2, v1),
            (v2, v4),
            (v3, v1),
            (v3, v4),
            (v3, v5),
            (v4, v2),
            (v4, v3),
            (v4, v6),
            (v5, v3),
            (v5, v6),
            (v6, v4),
            (v6, v5),
        ];
        TestCase { arcs, variables }
    }

    #[test]
    fn backtrack_1() {
        let mut rng = simple_rng("hello ");
        let TestCase {
            mut arcs,
            mut variables,
        } = build_test_case();

        assert_eq!(
            Solution::Consistent,
            reduce(&mut variables, &mut arcs, &StaticConstraints {}, &mut rng,)
        );
        println!(
            "{}, {}",
            variables.find_var(0).unwrap().possible_values()[0],
            variables.find_var(1).unwrap().possible_values()[0]
        );
        println!(
            "{}, {}",
            variables.find_var(2).unwrap().possible_values()[0],
            variables.find_var(3).unwrap().possible_values()[0]
        );
        println!(
            "{}, {}",
            variables.find_var(4).unwrap().possible_values()[0],
            variables.find_var(5).unwrap().possible_values()[0]
        );
    }

    /*
    #[test]
    fn backtrack_inconsistent() {
        // TODO: This test makes wild assumptions about the tileset. It's basic
        // intention is to exercise a bit of the backtracking logic since I am
        // not sure the best way to exercise it programmatically...
        let mut rng = simple_rng("hello world oasdf");
        let TestCase {
            tiles,
            mut arcs,
            mut domains,
        } = build_domain(5, 5);

        let reduction = reduce(&mut domains, &mut arcs, &tiles, &mut rng);
        assert_eq!(reduction, Solution::Consistent);
        assert_eq!(
            vec!(3, 8, 5, 2, 8, 0, 8, 5, 2, 8, 1, 7, 9, 3, 7, 2, 0, 6, 4, 1, 3, 0, 8, 5, 2),
            reduced_domains(domains),
        );
    }
    */
}
