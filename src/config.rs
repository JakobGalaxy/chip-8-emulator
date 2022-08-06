use std::path::Path;
use std::io;
use std::io::{Error};
use std::str::FromStr;
use confy::ConfyError;
use serde::{Serialize, Deserialize};

const CONFIG_PATH: &str = "./config/chip8-emulator.toml";

#[derive(Serialize, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub screen_scale: u32,
    pub font_path: String,
    pub program_path: String,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        return Self {
            screen_scale: 20,
            font_path: String::from("./fonts/chip48.font"),
            program_path: String::from("./programs/welcome.ch8"),
        };
    }
}

pub fn load_config() -> Result<ApplicationConfig, ConfyError> {
    let path = Path::new(CONFIG_PATH);
    return if path.exists() && {
        get_decision_input("continue with config?")
    } {
        Ok(confy::load_path(path)?)
    } else {
        let config: ApplicationConfig = run_application_config_dialog();

        // store config
        confy::store_path(path, config.clone())?;

        Ok(config)
    };
}

pub fn run_application_config_dialog() -> ApplicationConfig {
    println!("==== CHIP-8 EMULATOR CONFIG ====");

    // get user input for screen_scale
    let screen_scale: u32 = get_parsed_input::<u32>("screen_scale");

    // get user input for font_path
    let font_path: String = get_path_input("font_path");

    // get user input for program_path
    let program_path: String = get_path_input("program_path");

    return ApplicationConfig {
        screen_scale,
        font_path,
        program_path,
    };
}

fn get_decision_input(message: &str) -> bool {
    loop {
        println!("{} (y/n)", message);

        if let Ok(input) = get_user_input() {
            let input = input.trim();
            match input {
                "y" => return true,
                "n" => return false,
                _ => println!("invalid input!"),
            }
        }
    }
}

fn get_user_input() -> Result<String, Error> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)?;
    return Ok(input);
}

fn get_path_input(value_description: &str) -> String {
    loop {
        println!("{}: ", value_description);

        if let Ok(input) = get_user_input() {
            let input = String::from(input.trim());
            let path = Path::new(&input);
            if path.exists() {
                return input;
            }

            println!("invalid path!");
        }
    }
}

fn get_parsed_input<T: FromStr>(value_description: &str) -> T {
    loop {
        println!("{}: ", value_description);

        if let Ok(input) = get_user_input() {
            let result = input.trim().parse::<T>();
            if let Ok(number) = result {
                return number;
            }
            // if let Err(number) = result {
            //     println!("{}", number);
            // }

            println!("invalid input!");
        }
    }
}