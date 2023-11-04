use std::env;

use file_builder::start;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <folder_path> <website_name>", args[0]);
        std::process::exit(1);
    }

    let folder_path = &args[1];
    let website_name = &args[2];
    
    match start(folder_path, website_name) {
        Ok(_) => {
            println!("Completed Successfully")
        },
        Err(x) => {
            println!("ERR: {}", x)
        },
    }
}
