mod noter;

use noter::Noter;

use std::process;

fn main() {
    let (mut rl, thread) = raylib::init()
        .title("noter")
        .size(800, 600)
        .build();

    let mut noter = match Noter::new(&mut rl, thread) {
        Ok(noter) => noter,
        Err(err) => {
            println!("[ERROR] failed to initialize noter: {}", err.to_string());
            process::exit(1);
        },
    };

    if let Err(err) = noter.run() {
        println!("[ERROR] failed to run noter: {}", err.to_string());
        process::exit(1);
    }
}

