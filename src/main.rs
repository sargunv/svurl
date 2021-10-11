#[macro_use]
extern crate rocket;

mod command_handler;
mod config;

use rocket::{response::Redirect, State};

use crate::{command_handler::CommandHandler, config::load_config};

#[launch]
fn rocket() -> _ {
    let config = load_config().unwrap();
    let command_handler = CommandHandler::new(&config);

    rocket::build()
        .manage(config)
        .manage(command_handler)
        .mount("/", routes![get_search])
}

#[get("/search?<cmd>")]
fn get_search(cmd: &str, command_handler: &State<CommandHandler>) -> Option<Redirect> {
    let url = command_handler.handle(cmd.trim())?;
    Some(Redirect::temporary(url))
}
