use serde::Serialize;

use crate::datamodel::{Risk, TaskType};
use std::option::Option;

#[derive(FromForm, Debug, Clone)]
pub struct TaskForm {
    pub add: Option<bool>,

    pub uid: String,
    pub id: String,
    pub r#type: TaskType,
    pub title: String,

    // The following 2 do not exist for milestones
    pub estimate: Option<String>,
    pub risk: Option<Risk>,

    #[field(name = "add-dependency")]
    pub add_dependency: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Choice {
    pub value: String,
    pub caption: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct TaskView {
    pub uid: String,
    pub typ: TaskType,
    pub id: String,
    pub title: String,
    pub estimate: Option<u32>,
    pub risk: Option<Risk>,
    pub dependencies: Vec<TaskDependencyView>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TaskDependencyView {
    pub uid: String,
    pub id: String,
    pub title: String,
}
