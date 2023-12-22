use crate::appstate::Risk;
use std::option::Option;

#[derive(FromForm, Debug, Clone)]
pub struct TaskForm {
    pub add: Option<bool>,
    pub uid: String,
    pub id: String,
    pub title: String,
    pub estimate: String,
    pub risk: Option<Risk>,
}
