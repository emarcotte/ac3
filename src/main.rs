mod solver;
mod tile_matcher;

use std::collections::VecDeque;

use solver::Solver;
use tile_matcher::TileMatchBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules = TileMatchBuilder::new()
        // 0
        .up_down(0, 0)
        .up_down(0, 4)
        .up_down(0, 6)
        .up_down(0, 2)
        .left_right(0, 0)
        .left_right(0, 4)
        .left_right(0, 5)
        .left_right(0, 9)
        // 1
        .up_down(1, 0)
        .left_right(1, 7)
        // 2
        .up_down(2, 0)
        .left_right(2, 1)
        .left_right(2, 2)
        // 3
        .up_down(3, 3)
        .up_down(3, 2)
        .left_right(3, 3)
        .left_right(3, 7)
        // 4
        .up_down(4, 9)
        .left_right(4, 6)
        // 5
        .up_down(5, 0)
        .left_right(5, 2)
        // 6
        .up_down(6, 3)
        .left_right(6, 6)
        .left_right(6, 8)
        // 7
        .up_down(7, 1)
        .left_right(7, 0)
        // 8
        .up_down(8, 7)
        .left_right(8, 0)
        // 9
        .up_down(9, 9)
        .up_down(9, 5)
        .left_right(9, 3)
        // the end
        .build();

    let mut solver = Solver::new(
        vec!(0, 1, 2, 3),
    );

    // pretend:
    // a,b
    // c,d
    let a_id = solver.add_variable('a');
    let b_id = solver.add_variable('b');
    let c_id = solver.add_variable('c');
    let d_id = solver.add_variable('d');

    // a constraints
    solver.add_binary_constraint('a', 'b', rules.right())?;
    solver.add_binary_constraint('a', 'c', rules.down())?;
    solver.add_binary_constraint('b', 'a', rules.left())?;
    solver.add_binary_constraint('b', 'd', rules.down())?;
    solver.add_binary_constraint('c', 'd', rules.right())?;
    solver.add_binary_constraint('c', 'a', rules.up())?;
    solver.add_binary_constraint('d', 'c', rules.left())?;
    solver.add_binary_constraint('d', 'b', rules.up())?;

    println!("Constrainted");

    // TODO: Can these be discovered from constraints?
    let mut arcs = VecDeque::new();
    arcs.push_back((a_id, b_id));
    arcs.push_back((a_id, c_id));
    arcs.push_back((b_id, a_id));
    arcs.push_back((b_id, d_id));
    arcs.push_back((c_id, a_id));
    arcs.push_back((c_id, d_id));
    arcs.push_back((d_id, b_id));
    arcs.push_back((d_id, c_id));

    solver.solve(arcs);

    Ok(())
}
