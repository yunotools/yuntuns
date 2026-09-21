mod app;
mod cli;
mod manager;
mod output;
mod paths;
mod plugin;

use std::process;

const APP_NAME: &str = "yuntuns";

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let exit_code = match app::run(&args) {
        Ok(exit_code) => exit_code,
        Err(message) => {
            eprintln!("lỗi: {message}");
            1
        }
    };

    process::exit(exit_code);
}
