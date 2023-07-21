use csp::{ConstraintProvider, DomainProvider};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

/// Tracking state for the search algorithm to undo itself.
struct BacktrackState<V, D, DP> {
    variable: V,
    domains: DP,
    untested: Vec<D>,
}

// TODO: Should this be "externalized"? Like is there any reason to run CSP outside?
pub fn backtrack_reduce<V, D, DP, CP>(
    domains: &mut DP,
    arcs: &mut [(V, V)],
    constraints: &CP,
    rng: &mut SmallRng,
) where
    V: Eq + Copy,
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
            println!("Testing alternative..");
            let Some(mut prev) = backtrack.pop() else {
                panic!("No solution");
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
                println!("Bubbling up");
                domains.clone_from(&prev.domains);
            }
        } else {
            println!("All done.");
            break;
        }
    }
}
