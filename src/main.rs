mod solver;
mod tile_matcher;

use solver::Solver;
use tile_matcher::TileMatchBuilder;

use rand_seeder::Seeder;
use rand::prelude::SmallRng;
use rand::Rng;

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

    let width = 10u32;
    let height = 10u32;

    for x in 0..width {
        for y in 0..height {
            solver.add_variable(x + y * width);
        }
    }

    for x in 0..width {
        for y in 0..height {
            let index = x + y * width;
            if x > 0 {
                println!("l");
                solver.add_binary_constraint(index, index - 1, rules.left())?;
            }
            if x < width - 1 {
                println!("r");
                solver.add_binary_constraint(index, index + 1, rules.right())?;
            }
            if y > 0 {
                println!("d");
                solver.add_binary_constraint(index, index - width, rules.down())?;
            }
            if y < height - 1{
                println!("u");
                solver.add_binary_constraint(index, index + width, rules.up())?;
            }

        }
    }

    println!("Initial state:");
    println!("{}", solver);

    let mut rng = simple_rng("hello world bilbo");

    loop {
        if solver.solve() {
            println!("Done, but still have options!");
            println!("{}", solver);
            if ! select_random_variable_domain_value(&mut rng, &mut solver) {
                println!("No more unresolved variables");
                break;
            }
        }
        else {
            println!("No options left");
            break;
        }
    }

    println!("{}", solver);
    Ok(())
}

fn simple_rng(seed_str: &str) -> SmallRng {
    Seeder::from(seed_str).make_rng()
}

fn select_random_variable_domain_value(r: &mut SmallRng, solver: &mut Solver<u32, i32>) -> bool {
    let remaining = solver.unresolved_variables().collect::<Vec<_>>();
    if remaining.len() > 0 {
        let (v, domain) = remaining[r.gen_range(0..remaining.len())].clone();
        let selected = domain[r.gen_range(0..domain.len())];
        println!("Reducing domain of {v} to {selected} from {domain:?}");
        solver.set_domain(*v, selected);
        true
    }
    else {
        false
    }
}
