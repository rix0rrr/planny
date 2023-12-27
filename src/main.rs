#[macro_use]
extern crate rocket;

use std::collections::HashMap;

use datamodel::{roughly_sort_tasks, Task, TaskUpdate};
use db::Database;
use ids::unique_id;
use itertools::Itertools;
use rocket::{
    form::Form,
    fs::{relative, FileServer},
    State,
};
use rocket_dyn_templates::{context, Template};
use viewmodel::{Choice, TaskDependencyView, TaskView};

use crate::viewmodel::TaskForm;

mod datamodel;
mod db;
mod ids;
mod viewmodel;

// Not sure why I need a wrapper type, but if I use Database directly
// Rocket thinks I didn't add it
struct Db(Database);

// A variant of anyhow::Result that has responable errors
type AnyResult<T> = Result<T, rocket::response::Debug<anyhow::Error>>;

#[get("/")]
fn index() -> AnyResult<Template> {
    Ok(Template::render("index", context! {}))
}

#[get("/tasks")]
fn get_tasks(db: &State<Db>) -> AnyResult<Template> {
    let task_map: HashMap<String, Task> = db.0.tasks()?;
    let sorted_tasks = roughly_sort_tasks(
        task_map
            .values()
            .sorted_by(|a, b| human_sort::compare(&a.id, &b.id)),
    );

    // The list of elements in the dependency dropdown
    let task_list = task_map
        .values()
        .filter(|t| t.id != "")
        .map(|t| Choice {
            value: t.id.clone(),
            caption: t.title.clone(),
        })
        .sorted_by_key(|t| t.value.clone())
        .collect_vec();

    let tasks = sorted_tasks
        .sorted_tasks
        .into_iter()
        .map(|t| TaskView {
            uid: t.uid,
            id: t.id,
            title: t.title,
            estimate: t.estimate,
            risk: t.risk,
            dependencies: t
                .dependencies
                .iter()
                .filter_map(|d| task_map.get(d).cloned())
                .map(|t| TaskDependencyView {
                    uid: t.uid,
                    id: t.id,
                    title: t.title,
                })
                .sorted_by(|a, b| human_sort::compare(&a.id, &b.id))
                .collect(),
        })
        .collect_vec();

    let mut warnings: Vec<String> = vec![];
    for cycle in sorted_tasks.cycles {
        warnings.push(format!(
            "Tasks with a dependency cycle: {}",
            cycle.join(" → ")
        ));
    }

    Ok(Template::render(
        "partials/task-grid",
        context! {
            fresh_id: unique_id(),
            tasks,
            task_list,
            warnings,
        },
    ))
}

#[post("/tasks", data = "<form>")]
fn post_tasks(form: Form<TaskForm>, db: &State<Db>) -> AnyResult<Template> {
    // Convert add_dependency input (id) into a uid
    let mut add_dependencies: Vec<String> = vec![];
    if let Some(add) = &form.add_dependency {
        if add != "" {
            let tasks = db.0.tasks()?;
            tasks
                .into_values()
                .find(|t| &t.id == add)
                .map(|t| add_dependencies.push(t.uid));
        }
    }

    {
        db.0.upsert_task(TaskUpdate {
            uid: form.uid.clone(),
            id: Some(form.id.clone()),
            title: Some(form.title.clone()),
            estimate: Some(form.estimate.parse::<u32>().ok()),
            risk: form.risk.clone(),
            add_dependencies,
            remove_dependencies: vec![],
        })?;
    }
    get_tasks(db)
}

#[delete("/tasks/<uid>")]
fn delete_task(uid: &str, db: &State<Db>) -> AnyResult<Template> {
    {
        db.0.delete_task(uid)?;
    }
    get_tasks(db)
}

#[delete("/tasks/<task_uid>/dep/<dep_uid>")]
fn delete_dep(task_uid: &str, dep_uid: &str, db: &State<Db>) -> AnyResult<Template> {
    db.0.with_task(task_uid, |task| {
        task.dependencies.remove(dep_uid);
    })?;
    get_tasks(db)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![index, get_tasks, post_tasks, delete_task, delete_dep],
        )
        .mount("/s", FileServer::from(relative!("/static")))
        .attach(Template::fairing())
        .manage(Db(Database::new("db.json".to_string()).unwrap()))
}
