use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::Arc;

pub type ClonableBinaryConstraintFunction<D> = Arc<dyn Send + Sync + Fn(D, D) -> bool + 'static>;

/// A function that can determine if two variables are valid compared to each other.
#[derive(Clone)]
pub enum Constraint<D>
{
    // Global(fn() -> bool),
    // Unary(fn(&D) -> bool),
    Binary {
        scope: (usize, usize),
        rule: ClonableBinaryConstraintFunction<D>,
    },
}

/// Constraint solver over the domain of type `D` using AC-3 as described by Arificial Intelligent A Modern Approach (Ch. 6).
///
/// The solver will evaluate all constraints. This will leave it in either a partially assigned or completely assigned state. From there further refinements can be done externally, and the solver can be re-started.
pub struct Solver<V, D>
where
    V: Display + Copy + Clone,
    D: Clone + Copy + std::fmt::Debug + 'static,
{
    variables: Vec<V>,
    domains: Vec<Vec<D>>,
    domain_values: Vec<D>,
    constraints: Vec<Constraint<D>>,
}

impl <V, D> Solver<V, D>
where
    V: Display + Copy + Clone + PartialEq,
    D: PartialEq + Clone + Copy + std::fmt::Debug,
{
    pub fn new(values: Vec<D>) -> Self {
        Self {
            variables: vec!(),
            constraints: vec!(),
            domain_values: values,
            domains: vec!(),
        }
    }

    // TODO: Stop returning indexes.
    pub fn add_variable(&mut self, v: V) -> usize {
        self.variables.push(v);
        self.domains.push(self.domain_values.clone());
        self.variables.len() - 1
    }

    pub fn add_binary_constraint(
        &mut self,
        from: V,
        to: V,
        rules: &Vec<ClonableBinaryConstraintFunction<D>>,
    ) -> Result<(), String> {
        let from = self.variables.iter().position(|v| *v == from)
            .ok_or_else(|| format!("From value `{from}` not found"))?;

        let to = self.variables.iter().position(|v| *v == to)
            .ok_or_else(|| format!("To value `{to}` not found"))?;

        for rule in rules {
            self.constraints.push(Constraint::Binary {
                scope: (from, to),
                rule: rule.clone(),
            });
        }

        Ok(())
    }

    // TODO: look up arcs from constraints.
    pub fn solve(&mut self, arcs: &VecDeque<(usize, usize)>) -> bool {
        let mut arc_queue = arcs.clone();

        loop {
            if let Some(next) = arc_queue.pop_front() {
                let revised = self.revise(next);

                if revised {
                    if self.domains[next.0].len() == 0 {
                        return false;
                    }

                    // TODO enqueue any neighbors that need to propagate the revision.
                }
            }
            else {
                break;
            }
        }

        // TODO: This is insufficient.
        self.domains.iter().next().unwrap().len() != 0
    }

    fn revise(&mut self, (from, to): (usize, usize)) -> bool {
        let mut revised = false;

        let new_domain = self.domains[from]
            .iter()
            .copied()
            .filter_map(|from_v| {
                let satisfied = self.constraints.iter().any(|constraint| {
                    let Constraint::Binary { scope, rule } = constraint;
                    if *scope == (from, to) {
                        self.domains[to].iter().any(|to_v| rule(from_v, *to_v))
                    }
                    else {
                        false
                    }
                });

                if !satisfied {
                    revised = true;
                    None
                }
                else {
                    Some(from_v)
                }
            })
            .collect();

        if revised {
            println!("Domain of {from} reduced from {:?} to {:?}", self.domains[from], new_domain);
        }
        self.domains[from] = new_domain;

        revised
    }

    pub fn set_domain(&mut self, v: V, d: D) {
        if let Some(index) = self.variables.iter().position(|stored| *stored == v) {
            if self.domains[index].contains(&d) {
                self.domains[index] = vec!(d);
            }
        }
    }

    pub fn unresolved_variables(&self) -> impl Iterator<Item = (&V, &Vec<D>)> {
        self.variables.iter()
            .zip(&self.domains)
            .filter_map(|(var, domain)| {
                (domain.len() > 1).then_some((var, domain))
            })
    }
}

impl <V, D> Display for Solver<V, D>
where
    V: Display + Copy + Clone + PartialEq,
    D: Clone + Copy + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Variables:")?;
        for var in self.variables.iter().enumerate() {
            writeln!(f, "  {}: {} => {:?}", var.0, var.1, self.domains[var.0])?;
        }
        Ok(())
    }
}
