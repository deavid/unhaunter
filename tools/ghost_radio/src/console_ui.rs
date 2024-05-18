// tools/ghost_radio/src/console_ui.rs

use std::io::{self, Write};

use crate::data::GhostResponse;

pub fn display_ghost_options(ghost_responses: &[&str]) {
    println!("Available Ghosts:");
    for (i, key) in ghost_responses.iter().enumerate() {
        println!("{}. {}", i + 1, key);
    }

    print!("Select a ghost (enter number): ");
    io::stdout().flush().unwrap();
}

pub fn get_player_phrase(phrases: &[String]) -> String {
    print!("Enter a phrase (enter number 1-{}): ", phrases.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let idx: usize = input.trim().parse().unwrap();
    phrases[idx - 1].to_owned()
}

pub fn display_ghost_response(response: &GhostResponse) {
    println!("Ghost response: {}", response.phrase);
}
