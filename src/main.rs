mod app;
mod cli;
mod manager;
mod output;
mod paths;
mod plugin;

use std::process;

const APP_NAME: &str = "yuntuns";

fn main() {
    // Lấy các command-line arguments kiểu Args
    // yuntuns plugin install copyast
    // arg[0] = "yuntuns"
    // arg[1] = "plugin"
    // arg[2] = "install"
    // arg[3] = "copyast"
    // Bỏ qua một phần tử đầu tiên: tức bỏ yuntuns
    // -> Gom vào một collection
    // -> Turbofish syntax
    // vec![
    //     "plugin".to_string(),
    //     "install".to_string(),
    //     "copyast".to_string(),
    // ]
    // ~~ let args: Vec<String> = std::env::args()
    //     .skip(1)
    //     .collect();
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let exit_code = match app::run(&args) {
        Ok(exit_code) => exit_code,
        // Expression-oriented language
        Err(message) => {
            eprintln!("lỗi: {message}");
            1
        }
    };

    process::exit(exit_code);
}
