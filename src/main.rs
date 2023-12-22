#[macro_use]
extern crate rocket;

use db::Database;
use ids::unique_id;
use rocket::{
    form::Form,
    fs::{relative, FileServer},
    State,
};
use rocket_dyn_templates::{context, Template};

use crate::{appstate::Task, forms::TaskForm};

mod appstate;
mod data;
mod db;
mod forms;
mod ids;

// Not sure why I need a wrapper type, but if I use Database directly
// Rocket thinks I didn't add it
struct Db(Database);

// A variant of anyhow::Result that has responable errors
type AnyResult<T> = Result<T, rocket::response::Debug<anyhow::Error>>;

#[get("/")]
fn index(db: &State<Db>) -> AnyResult<Template> {
    Ok(Template::render(
        "index",
        context! {
            fresh_id: unique_id(),
            tasks: db.0.tasks()?,
        },
    ))
}

#[post("/tasks", data = "<form>")]
fn post_tasks(form: Form<TaskForm>, db: &State<Db>) -> AnyResult<Template> {
    {
        db.0.upsert_task(Task {
            uid: form.uid.clone(),
            id: form.id.clone(),
            title: form.title.clone(),
            estimate: form.estimate.parse::<u32>().ok(),
            risk: form.risk.clone(),
        })?;
    }
    render_task_grid(db)
}

#[delete("/tasks/<uid>")]
fn delete_task(uid: &str, db: &State<Db>) -> AnyResult<Template> {
    {
        db.0.delete_task(uid)?;
    }
    render_task_grid(db)
}

fn render_task_grid(db: &State<Db>) -> AnyResult<Template> {
    Ok(Template::render(
        "partials/task-grid",
        context! {
            fresh_id: unique_id(),
            tasks: db.0.tasks()?,
        },
    ))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, post_tasks, delete_task])
        .mount("/s", FileServer::from(relative!("/static")))
        .attach(Template::fairing())
        .manage(Db(Database::new().unwrap()))
}
