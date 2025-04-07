use std::thread;
use std::io::{self, Write};
use crossbeam_channel::{Receiver, Sender};
use crate::controller::Controller;
use crate::settings_changed::SettingsChanged;

pub struct Cli {
    _thread: thread::JoinHandle<()>,
}

impl Cli {
    pub fn init(mut ctrl: Controller) -> Cli {
        // Get notified by the controller when settings have changed
        println!("CLI: about to register settings listener");
        let ctrl_settings_listener: Receiver<SettingsChanged> = ctrl.register_settings_listener();

        // Start playing the browsing directory
        println!("CLI: about to play dir");
        ctrl.play_browsing_dir().unwrap_or_else(|e| {
            eprintln!("Failed to play music files: {}", e);
        });

        // Spawn a thread for CLI interaction
        println!("CLI: about to spawn thread");
        let thread: thread::JoinHandle<()> = thread::spawn(move || {
            println!("CLI: before loop");
            loop {
                // Check for settings changes
                // match ctrl_settings_listener.try_recv() {
                //     Ok(s) => {
                //         println!("CLI: New settings {:?}", s);
                //     }
                //     Err(_) => {
                //         // No new settings, just sleep for a bit
                //         std::thread::sleep(std::time::Duration::from_secs(1));
                //     }
                // };

                // Display CLI menu
                println!("\nCLI Menu:");
                println!("1. Specify directory");
                println!("2. Play");
                println!("3. Pause");
                println!("4. Stop");
                println!("n - Next");
                println!("s - Show status");
                println!("x - Exit");
                print!("Enter your choice: ");
                io::stdout().flush().unwrap();

                // Read user input
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let choice = input.trim();

                match choice {
                    "1" => {
                        println!("Enter the directory path:");
                        let mut dir_input = String::new();
                        io::stdin().read_line(&mut dir_input).unwrap();
                        let dir_path = dir_input.trim();
                        // TODO:
                    }
                    "2" => {
                        ctrl.play();
                    }
                    "3" => {
                        ctrl.pause();
                    }
                    "4" => {
                        ctrl.stop();
                    }
                    "n" => {
                        ctrl.next();
                    }
                    "s" => {
                        match ctrl_settings_listener.try_recv() {
                            Ok(status) => {
                                println!("CLI: Playback status {:?}", status);
                            },
                            Err(_) => println!("No status updates available."),
                        }
                    }
                    "x" => {
                        println!("Exiting...");
                        break;
                    }
                    _ => {
                        println!("Invalid choice. Please try again.");
                    }
                }
            }
        });

        println!("CLI: after thread");
        Cli {
            _thread: thread,
        }
    }

    pub fn is_done(&self) -> bool {
        self._thread.is_finished()
    }
}