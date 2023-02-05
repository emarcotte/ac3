use std::collections::HashSet;
use std::sync::Arc;
use crate::solver::ClonableBinaryConstraintFunction;

type TupleMatcher = ClonableBinaryConstraintFunction<i32>;

pub fn tuple_matchers(a: i32, b: i32) -> TupleMatcher
{
    Arc::new(move |av: i32, bv: i32|  av == a && bv == b)
}

pub struct TileMatchBuilder {
    up_down: HashSet<(i32, i32)>,
    left_right: HashSet<(i32, i32)>,
}

impl TileMatchBuilder {
    pub fn new() -> Self {
        Self {
            up_down: HashSet::new(),
            left_right: HashSet::new(),
        }
    }

    pub fn left_right(mut self, l: i32, r: i32) -> Self {
        self.left_right.insert((l, r));
        self
    }

    pub fn up_down(mut self, u: i32, d: i32) -> Self {
        self.up_down.insert((u, d));
        self
    }

    pub fn build(self) -> TileMatchSet {
        self.into()
    }
}

pub struct TileMatchSet
{
    up: Vec<TupleMatcher>,
    down: Vec<TupleMatcher>,
    right: Vec<TupleMatcher>,
    left: Vec<TupleMatcher>,
}

impl Into<TileMatchSet> for TileMatchBuilder {
    fn into(self) -> TileMatchSet {
        let up = self.up_down.iter()
            .map(|(u, d)| tuple_matchers(*u, *d))
            .collect();

        let down = self.up_down.iter()
            .map(|(u, d)| tuple_matchers(*d, *u))
            .collect();

        let left = self.left_right.iter()
            .map(|(l, r)| tuple_matchers(*l, *r))
            .collect();

        let right = self.left_right.iter()
            .map (|(l, r)| tuple_matchers(*r, *l))
            .collect();

        TileMatchSet {
            up,
            down,
            right,
            left,
        }
    }
}

impl TileMatchSet {
    pub fn left(&self) -> &Vec<TupleMatcher> {
        &self.left
    }

    pub fn right(&self) -> &Vec<TupleMatcher> {
        &self.right
    }

    pub fn up(&self) -> &Vec<TupleMatcher> {
        &self.up
    }

    pub fn down(&self) -> &Vec<TupleMatcher> {
        &self.down
    }
}
