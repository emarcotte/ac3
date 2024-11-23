use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::ac3::{Constraint, ConstraintProvider, DomainType, IdentifierType};
use crate::variable_provider::{Variable, VariableID};

impl DomainType for char {}
impl DomainType for u8 {}
impl DomainType for u16 {}
impl DomainType for u32 {}
impl DomainType for u64 {}
impl DomainType for i8 {}
impl DomainType for i16 {}
impl DomainType for i32 {}
impl DomainType for i64 {}
impl DomainType for usize {}

impl IdentifierType for char {}
impl IdentifierType for u8 {}
impl IdentifierType for u16 {}
impl IdentifierType for u32 {}
impl IdentifierType for u64 {}
impl IdentifierType for i8 {}
impl IdentifierType for i16 {}
impl IdentifierType for i32 {}
impl IdentifierType for i64 {}
impl IdentifierType for usize {}

// TODO: A constraint database could probably avoid the hash structure by scanning relationships
// more efficient... maybe tied directly to arc list?
impl<K, D, S1> ConstraintProvider<D, K> for HashMap<(VariableID, VariableID), Constraint<D>, S1>
where
    S1: BuildHasher,
    D: DomainType,
    K: IdentifierType,
{
    fn check(&self, a: &Variable<D, K>, av: &D, b: &Variable<D, K>, bv: &D) -> bool {
        // TODO: Default is to be unconstrained, i guess.
        self.get(&(a.index, b.index))
            .map_or(true, |checker: &Constraint<D>| checker(av, bv))
    }
}
