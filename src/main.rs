use parser::parse_top_level;

mod parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_name = &args[1];
    let input = std::fs::read_to_string(file_name).unwrap();
    let (_, top_level) = parse_top_level(&input).unwrap();
    println!("{:?}", top_level);
}
