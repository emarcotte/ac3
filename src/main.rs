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
        vec!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9),
    );

    // pretend:
    // a,b,c
    // d,e,f
    // g,h,i
    let a_id = solver.add_variable('a');
    let b_id = solver.add_variable('b');
    let c_id = solver.add_variable('c');
    let d_id = solver.add_variable('d');
    let e_id = solver.add_variable('e');
    let f_id = solver.add_variable('f');
    let g_id = solver.add_variable('g');
    let h_id = solver.add_variable('h');
    let i_id = solver.add_variable('i');

    // add constraints
    solver.add_binary_constraint('a', 'b', rules.right())?;
    solver.add_binary_constraint('a', 'd', rules.down())?;

    solver.add_binary_constraint('b', 'a', rules.left())?;
    solver.add_binary_constraint('b', 'c', rules.right())?;
    solver.add_binary_constraint('b', 'e', rules.down())?;

    solver.add_binary_constraint('c', 'b', rules.left())?;
    solver.add_binary_constraint('c', 'f', rules.down())?;

    solver.add_binary_constraint('d', 'a', rules.up())?;
    solver.add_binary_constraint('d', 'e', rules.right())?;
    solver.add_binary_constraint('d', 'g', rules.down())?;

    solver.add_binary_constraint('e', 'b', rules.up())?;
    solver.add_binary_constraint('e', 'd', rules.left())?;
    solver.add_binary_constraint('e', 'f', rules.right())?;
    solver.add_binary_constraint('e', 'h', rules.down())?;

    solver.add_binary_constraint('f', 'c', rules.up())?;
    solver.add_binary_constraint('f', 'e', rules.left())?;
    solver.add_binary_constraint('f', 'i', rules.down())?;

    solver.add_binary_constraint('g', 'd', rules.up())?;
    solver.add_binary_constraint('g', 'h', rules.right())?;

    solver.add_binary_constraint('h', 'e', rules.up())?;
    solver.add_binary_constraint('h', 'g', rules.left())?;
    solver.add_binary_constraint('h', 'i', rules.right())?;

    solver.add_binary_constraint('i', 'f', rules.up())?;
    solver.add_binary_constraint('i', 'h', rules.left())?;

    println!("Constrainted");

    // TODO: Can these be discovered from constraints?
    let mut arcs = VecDeque::new();
    arcs.push_back((a_id, b_id));
    arcs.push_back((a_id, d_id));

    arcs.push_back((b_id, a_id));
    arcs.push_back((b_id, c_id));
    arcs.push_back((b_id, e_id));

    arcs.push_back((c_id, b_id));
    arcs.push_back((c_id, f_id));

    arcs.push_back((d_id, a_id));
    arcs.push_back((d_id, e_id));
    arcs.push_back((d_id, g_id));

    arcs.push_back((e_id, b_id));
    arcs.push_back((e_id, d_id));
    arcs.push_back((e_id, f_id));
    arcs.push_back((e_id, h_id));

    arcs.push_back((f_id, c_id));
    arcs.push_back((f_id, e_id));
    arcs.push_back((f_id, i_id));

    arcs.push_back((g_id, d_id));
    arcs.push_back((g_id, h_id));

    arcs.push_back((h_id, e_id));
    arcs.push_back((h_id, g_id));
    arcs.push_back((h_id, i_id));

    arcs.push_back((i_id, f_id));
    arcs.push_back((i_id, h_id));

    println!("Initial state:");
    println!("{}", solver);

    loop {
        if solver.solve(&arcs) {
            println!("Done, but still have options!");
        }
        else {
            println!("No options left");
        }

        if ! select_random_variable_domain_value(&mut solver) {
            break;
        }
    }

    println!("{}", solver);
    Ok(())
}

fn select_random_variable_domain_value(solver: &mut Solver<char, i32>) -> bool {
    let remaining = solver.unresolved_variables().collect::<Vec<_>>();
    if remaining.len() > 0 {
        let (v, domain) = remaining[0].clone();
        let v = *v;
        let domain = domain.clone();
        drop(remaining);
        println!("Reducing domain of {v} to {:}", domain[0]);
        solver.set_domain(v, domain[0]);
        true
    }
    else {
        false
    }
}
