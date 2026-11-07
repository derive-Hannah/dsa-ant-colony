mod hex_grid;
use hex_grid::HexGrid;

fn main() {
    println!("==PARALLELAGRAM (0, 10, -4, 4)==");
    println!("{}", HexGrid::from_bounds_parallelagram('P', 0, 10, -4, 4));
    println!("==REVERSE PARALLELAGRAM (0, 10, -4, 4)==");
    println!("{}", HexGrid::from_bounds_rparallelagram('R', 0, 10, -4, 4));
    println!("==HEXAGON (0, 5, 4)==");
    println!("{}", HexGrid::from_bounds_hexagon('H', 0, 5, 4));
}
