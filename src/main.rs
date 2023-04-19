use sudoku::Puzzle;

fn main() {
    let mut puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");
    puzzle.solve();

    println!("{}", puzzle.readout());
}
