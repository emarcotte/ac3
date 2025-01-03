use rand::Rng;

use crate::ac3::{DomainType, IdentifierType};
use rand::seq::SliceRandom;
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    hash::Hash,
};

#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub struct VariableID(usize);

/// A [`Variable`] is a node in the graph with a set of possible values. In a tile map it might be a
/// single tile, for example.
#[derive(Clone, Debug)]
pub struct Variable<D, K> {
    /// The possible values available to this variable.
    domain: RefCell<Vec<D>>,
    /// How external users refer to this variable.
    pub identifier: K,
    /// Internal tracker for the variable.
    pub(crate) index: VariableID,
}

impl<D, K> Variable<D, K>
where
    D: Copy,
{
    /// Construct A variable from a set of possible values.
    fn new(id: K, index: usize, values: Vec<D>) -> Self {
        Self {
            domain: RefCell::new(values),
            identifier: id,
            index: VariableID(index),
        }
    }

    // Check if the variable has any possible valid value remaining.
    pub fn is_consistent(&self) -> bool {
        return !self.possible_values().is_empty();
    }

    pub fn possible_values(&self) -> Ref<Vec<D>> {
        self.domain.borrow()
    }

    pub fn possible_values_mut(&self) -> RefMut<Vec<D>> {
        self.domain.borrow_mut()
    }

    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&D) -> bool,
    {
        self.domain.borrow_mut().retain(f);
    }

    /// Select a random possible value.
    pub fn choose<R>(&self, rng: &mut R) -> Option<D>
    where
        R: Rng + ?Sized,
    {
        self.possible_values().choose(rng).copied()
    }

    pub fn replace_possible_values(&self, values: Vec<D>) {
        self.domain.replace(values);
    }
}

/// A [`VariableProvider`] maintains the set of identifiers (type K) mapped to [`Variable`]s as
/// well as their possible values (subset of the full-domain, type D).
#[derive(Clone, Debug)]
pub struct VariableProvider<D, K> {
    identifiers: Vec<Variable<D, K>>,
}

impl<D, K> Default for VariableProvider<D, K> {
    fn default() -> Self {
        Self {
            identifiers: Vec::new(),
        }
    }
}

// TODO: Move to impls
impl<D, K, F> From<F> for VariableProvider<D, K>
where
    D: DomainType,
    K: IdentifierType,
    F: Into<HashMap<K, Vec<D>>>,
{
    fn from(value: F) -> Self {
        let items: HashMap<K, Vec<D>> = value.into();
        Self {
            identifiers: items
                .into_iter()
                .enumerate()
                .map(|(index, (id, values))| Variable::new(id, index, values))
                .collect(),
        }
    }
}

impl<D, K> VariableProvider<D, K>
where
    D: DomainType,
    K: IdentifierType,
{
    /// # Errors
    ///
    /// Fails if the identifier is already in use.
    pub fn add_var(&mut self, id: K, values: Vec<D>) -> Result<VariableID, String> {
        if self
            .identifiers
            .iter()
            .any(|existing| existing.identifier == id)
        {
            Err(format!("Identifier already in use {id}"))
        } else {
            let var = Variable::new(id, self.identifiers.len(), values);
            let var_id = var.index;
            self.identifiers.push(var);
            Ok(var_id)
        }
    }

    /// Replaces the possible values for the given identifier.
    pub fn update_var(&mut self, id: VariableID, values: Vec<D>) {
        if let Some(var) = self.identifiers.get_mut(id.0) {
            var.replace_possible_values(values);
        }
    }

    #[must_use]
    pub fn get_var(&self, index: VariableID) -> Option<&Variable<D, K>> {
        self.identifiers.get(index.0)
    }

    #[must_use]
    pub fn find_var(&self, var: K) -> Option<&Variable<D, K>> {
        self.identifiers.iter().find(|id| id.identifier == var)
    }

    #[must_use]
    pub fn find_id(&self, var: K) -> Option<VariableID> {
        self.identifiers
            .iter()
            .find(|id| id.identifier == var)
            .map(|id| id.index)
    }

    pub(crate) fn next_reducable_variable(&mut self) -> Option<VariableID> {
        self.identifiers
            .iter()
            .filter_map(|v| {
                let len = v.possible_values().len();
                if len > 1 {
                    Some((len, v))
                } else {
                    None
                }
            })
            .min_by(|(a_len, a), (b_len, b)| a_len.cmp(b_len).then(a.identifier.cmp(&b.identifier)))
            .map(|(_, min)| min.index)
    }

    pub(crate) fn is_consistent(&self) -> bool {
        self.identifiers.iter().all(Variable::is_consistent)
    }
}
