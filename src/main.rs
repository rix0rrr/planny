#[macro_use]
extern crate rocket;

use std::{
    cmp::{max, min},
    collections::HashMap,
    ops::Range,
};

use datamodel::{roughly_sort_tasks, Task, TaskUpdate};
use db::Database;
use forecast::simulate_tasks;
use ids::unique_id;
use itertools::Itertools;
use rocket::{
    form::Form,
    fs::{relative, FileServer},
    response::Redirect,
    State,
};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use viewmodel::{Choice, ProjectNameForm, ProjectPeopleForm, TaskDependencyView, TaskView};

use crate::{
    forecast::{convert_rng, query_minmax},
    render_forecast::render_dist,
    svg::RenderedSvg,
    viewmodel::TaskForm,
};

mod datamodel;
mod db;
mod forecast;
mod hstable;
mod ids;
mod render_forecast;
mod svg;
mod topo_queue;
mod viewmodel;

// Not sure why I need a wrapper type, but if I use Database directly
// Rocket thinks I didn't add it
struct Db(Database);

// A variant of anyhow::Result that has responable errors
type AnyResult<T> = Result<T, rocket::response::Debug<anyhow::Error>>;

#[get("/")]
fn index(db: &State<Db>) -> AnyResult<Template> {
    let projects = db.0.projects()?;
    Ok(Template::render(
        "index",
        context! {
            projects,
        },
    ))
}

#[get("/project/<project_uid>")]
fn get_project(project_uid: &str, db: &State<Db>) -> AnyResult<Template> {
    let project = db.0.project(project_uid)?;
    Ok(Template::render(
        "project",
        context! {
            project,
        },
    ))
}

#[post("/projects/create")]
fn create_project(db: &State<Db>) -> AnyResult<Redirect> {
    let uid = db.0.new_project()?;
    Ok(Redirect::moved(format!("/project/{}", uid)))
}

#[post("/project/<project_uid>/name", data = "<form>")]
fn post_project_name(
    project_uid: &str,
    form: Form<ProjectNameForm>,
    db: &State<Db>,
) -> AnyResult<Template> {
    if !form.project_name.is_empty() {
        db.0.with_project(project_uid, |proj| {
            proj.name = form.project_name.clone();
        })?;
    }
    get_project(project_uid, db)
}

#[get("/project/<project_uid>/tasks")]
fn get_tasks(project_uid: &str, db: &State<Db>) -> AnyResult<Template> {
    let project = db.0.project(project_uid)?;

    let task_map: HashMap<String, Task> = HashMap::from_iter(
        db.0.tasks()?
            .into_many(project_uid)
            .map(|t| (t.uid.clone(), t)),
    );

    let sorted_tasks = roughly_sort_tasks(task_map.values());

    // The list of elements in the dependency dropdown
    let task_list = task_map
        .values()
        .filter(|t| !t.id.is_empty())
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
            typ: t.r#type,
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
            project_uid: project_uid,
            fresh_id: unique_id(),
            project,
            tasks,
            task_list,
            warnings,
        },
    ))
}

#[get("/project/<project_uid>/people")]
fn get_people(project_uid: &str, db: &State<Db>) -> AnyResult<Template> {
    let project = db.0.project(project_uid)?.unwrap();
    Ok(Template::render(
        "partials/people",
        context! {
            project,
        },
    ))
}

#[post("/project/<project_uid>/people", data = "<form>")]
fn post_people(
    project_uid: &str,
    form: Form<ProjectPeopleForm>,
    db: &State<Db>,
) -> AnyResult<Template> {
    db.0.with_project(project_uid, |project| {
        if form.people > 0 {
            project.people = form.people;
        }
    })?;
    get_people(project_uid, db)
}

#[get("/project/<project_uid>/forecast")]
fn get_forecast(project_uid: &str, db: &State<Db>) -> AnyResult<Option<Template>> {
    let Some(project) = db.0.project(project_uid)? else {
        return Ok(None);
    };

    let tasks = db.0.tasks()?.into_many(project_uid).collect_vec();
    let sorted_tasks = roughly_sort_tasks(tasks.iter());
    if !sorted_tasks.cycles.is_empty() {
        return Ok(Some(Template::render(
            "partials/forecast",
            context! {
                cycles: sorted_tasks.cycles
            },
        )));
    }

    let rs = simulate_tasks(tasks.into_iter(), project.people);
    #[derive(Serialize)]
    struct TaskPrediction {
        task: Task,
        full_rng: Range<u32>,
        full_svg: RenderedSvg,
    }
    let task_timeline: Vec<TaskPrediction> = sorted_tasks
        .sorted_tasks
        .iter()
        .map(|task| {
            let rng = rs.task_stats.get(&task.uid).unwrap();

            let start_rng = convert_rng(&query_minmax(&rng.start, 0.01));
            let end_rng = convert_rng(&query_minmax(&rng.end, 0.01));

            let full_rng = start_rng.start..end_rng.end;

            println!("Task {} {:?}", task.id, full_rng);

            TaskPrediction {
                task: task.clone(),
                full_svg: render_dist(&rng.start, &rng.end, &full_rng).render(),
                full_rng,
            }
        })
        .collect();

    let time_range = task_timeline.iter().fold(0..0, |r, x| {
        min(r.start, x.full_rng.start)..max(r.end, x.full_rng.end)
    });

    Ok(Some(Template::render(
        "partials/forecast",
        context! {
            task_timeline,
            time_range: time_range.collect_vec(),
        },
    )))
}

#[post("/project/<project_uid>/tasks", data = "<form>")]
fn post_tasks(project_uid: &str, form: Form<TaskForm>, db: &State<Db>) -> AnyResult<Template> {
    // Convert add_dependency input (id) into a uid
    let mut add_dependencies: Vec<String> = vec![];
    if let Some(add) = &form.add_dependency {
        if !add.is_empty() {
            if let Some(t) = db.0.tasks()?.into_many(project_uid).find(|t| &t.id == add) {
                add_dependencies.push(t.uid)
            }
        }
    }

    {
        db.0.upsert_task(TaskUpdate {
            project_uid: project_uid.to_owned(),
            uid: form.uid.clone(),
            r#type: Some(form.r#type.clone()),
            id: Some(form.id.clone()),
            title: Some(form.title.clone()),
            estimate: form.estimate.as_ref().map(|x| x.parse::<f64>().ok()),
            risk: form.risk.as_ref().map(|risk| Some(*risk)),
            add_dependencies,
            remove_dependencies: vec![],
        })?;
    }
    get_tasks(project_uid, db)
}

#[delete("/project/<project_uid>/tasks/<uid>")]
fn delete_task(project_uid: &str, uid: &str, db: &State<Db>) -> AnyResult<Template> {
    {
        db.0.delete_task(project_uid, uid)?;
    }
    get_tasks(project_uid, db)
}

#[delete("/project/<project_uid>/tasks/<task_uid>/dep/<dep_uid>")]
fn delete_dep(
    project_uid: &str,
    task_uid: &str,
    dep_uid: &str,
    db: &State<Db>,
) -> AnyResult<Template> {
    db.0.with_task(project_uid, task_uid, |task| {
        task.dependencies.remove(dep_uid);
    })?;
    get_tasks(project_uid, db)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                get_tasks,
                post_tasks,
                delete_task,
                delete_dep,
                get_project,
                post_project_name,
                create_project,
                get_people,
                post_people,
                get_forecast,
            ],
        )
        .mount("/s", FileServer::from(relative!("/static")))
        .attach(Template::fairing())
        .manage(Db(Database::new("db.json".to_string()).unwrap()))
}
