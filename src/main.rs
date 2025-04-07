use controller::Controller;
use cli::Cli;

mod music_player;
mod file_utils;
mod controller;
mod cli;
mod settings_changed;

fn main() {
    println!("Application starting...");

    // Get a listing of all music files available
    // let starting_dir : &str = "/home/jonathan/tmp";
    let starting_dir : &str = "/home/jonathan/Music";

    let ctrl: Controller = Controller::init(starting_dir.to_string());  

    // get a list of all subdirectories
    let sub_dirs_res: Result<Vec<String>, std::io::Error> = ctrl.get_available_dirs();
    match sub_dirs_res {
        Ok(sub_dirs) => {
            for sub_dir in sub_dirs {
                println!("Sub directory found {}", sub_dir);
            }
        },
        Err(_) => println!("No sub directories found"),
    }

    let cli: Cli = Cli::init(ctrl);
    loop {
        if cli.is_done() {
            println!("Closing gracefully");
            break;
        }
        else {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    println!("All Done!");
}
