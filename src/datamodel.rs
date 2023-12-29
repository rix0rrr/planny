use std::collections::{HashMap, HashSet};

use petgraph::{algo::tarjan_scc, graph::DiGraph, matrix_graph::NodeIndex, Graph};
use serde::{Deserialize, Serialize};

use crate::hstable::HashSortable;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Task {
    pub project_uid: String,
    pub uid: String,
    #[serde(default)]
    pub r#type: TaskType,
    pub id: String,
    pub title: String,
    pub estimate: Option<u32>,
    pub risk: Option<Risk>,
    pub dependencies: HashSet<String>,
}

impl HashSortable for Task {
    type HashKey = String;
    type SortKey = String;

    fn key(&self) -> (&String, &String) {
        (&self.project_uid, &self.uid)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Project {
    pub uid: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, FromFormField)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    #[default]
    Task,
    Milestone,
}

impl Task {
    pub fn ensure_defaults(&mut self, task_count: usize) {
        if self.id.is_empty() {
            self.id = format!("T{}", task_count + 1);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, FromFormField, Default)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Low,
    #[default]
    Medium,
    High,
}

pub struct TaskUpdate {
    pub project_uid: String,
    pub uid: String,
    pub r#type: Option<TaskType>,
    pub id: Option<String>,
    pub title: Option<String>,
    pub estimate: Option<Option<u32>>,
    pub risk: Option<Option<Risk>>,
    pub add_dependencies: Vec<String>,
    pub remove_dependencies: Vec<String>,
}

impl TaskUpdate {
    pub fn apply(self, task: &Task) -> Task {
        Task {
            project_uid: self.project_uid.clone(),
            uid: self.uid.clone(),
            r#type: self.r#type.unwrap_or(task.r#type.clone()),
            id: self.id.unwrap_or(task.id.clone()),
            title: self.title.unwrap_or(task.title.clone()),
            estimate: self.estimate.unwrap_or(task.estimate),
            risk: self.risk.unwrap_or(task.risk.clone()),
            dependencies: {
                let mut deps = task.dependencies.clone();
                for d in self.add_dependencies {
                    if d != self.uid {
                        deps.insert(d);
                    }
                }
                for d in self.remove_dependencies {
                    deps.remove(&d);
                }
                deps
            },
        }
    }
}

/// Make a dependency graph from a set of tasks
fn make_dependency_graph<'a>(tasks: impl Iterator<Item = &'a Task> + Clone) -> Graph<Task, ()> {
    let tasks_copy = tasks.clone();
    let mut g = DiGraph::<Task, (), u32>::new();
    let nodes: HashMap<String, NodeIndex<u32>> =
        HashMap::from_iter(tasks.map(|t| (t.uid.clone(), g.add_node(t.clone()))));
    for task in tasks_copy {
        if let Some(task_node) = nodes.get(&task.uid) {
            for d in &task.dependencies {
                if let Some(dep_node) = nodes.get(d) {
                    g.add_edge(*task_node, *dep_node, ());
                }
            }
        }
    }
    g
}

/// Sort tasks ~ using a Strongly Connected Components algorithm
///
/// This will not fail if there are cycles.
pub fn roughly_sort_tasks<'a>(tasks: impl Iterator<Item = &'a Task> + Clone) -> SortedTasks {
    let g = make_dependency_graph(tasks);
    let indexes = tarjan_scc(&g);

    SortedTasks {
        cycles: indexes
            .iter()
            .filter(|vs| vs.len() > 1)
            .map(|vs| {
                vs.iter()
                    .filter_map(|v| g.node_weight(*v))
                    .map(|w| w.id.clone())
                    .collect()
            })
            .collect(),
        sorted_tasks: indexes
            .iter()
            .flat_map(|ts| ts.iter())
            .filter_map(|id| g.node_weight(*id))
            .cloned()
            .collect(),
    }
}

pub struct SortedTasks {
    pub sorted_tasks: Vec<Task>,
    pub cycles: Vec<Vec<String>>,
}
