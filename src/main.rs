use clap::Parser;

mod options;

fn main() {
    let options = options::Options::parse();
    println!("{:?}", options);
}
