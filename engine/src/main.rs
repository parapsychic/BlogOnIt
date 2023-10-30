use std::env;

use file_builder::start;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <folder_path>", args[0]);
        std::process::exit(1);
    }

    let folder_path = &args[1];
    
    match start(folder_path) {
        Ok(_) => {
            println!("Completed Successfully")
        },
        Err(x) => {
            println!("ERR: {}", x)
        },
    }
}
