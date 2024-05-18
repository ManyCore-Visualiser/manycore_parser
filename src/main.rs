use manycore_parser::ManycoreSystem;

fn main() {
    let path = "./tests/VisualiserOutput1.xml";

    let _ = ManycoreSystem::parse_file(path);
}