#[macro_use] extern crate rocket;

use appstate::AppState;
use rocket_dyn_templates::Template;
use rocket::{fs::{FileServer, relative}, State};

mod appstate;

#[get("/")]
fn index(state: &State<AppState>) -> Template {
    Template::render("index", state.inner())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/s", FileServer::from(relative!("/static")))
        .attach(Template::fairing())
        .manage(AppState::new())
}