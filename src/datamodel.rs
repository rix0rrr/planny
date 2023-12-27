use std::collections::{HashMap, HashSet};

use petgraph::{algo::tarjan_scc, graph::DiGraph, matrix_graph::NodeIndex, Graph};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Task {
    pub uid: String,
    pub id: String,
    pub title: String,
    pub estimate: Option<u32>,
    pub risk: Risk,
    pub dependencies: HashSet<String>,
}

impl Task {
    pub fn ensure_defaults(&mut self, task_count: usize) {
        if self.id == "" {
            self.id = format!("T{}", task_count + 1);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, FromFormField)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Low,
    Medium,
    High,
}

impl Default for Risk {
    fn default() -> Self {
        Risk::Medium
    }
}

pub struct TaskUpdate {
    pub uid: String,
    pub id: Option<String>,
    pub title: Option<String>,
    pub estimate: Option<Option<u32>>,
    pub risk: Option<Risk>,
    pub add_dependencies: Vec<String>,
    pub remove_dependencies: Vec<String>,
}

impl TaskUpdate {
    pub fn apply(self, task: &Task) -> Task {
        Task {
            uid: self.uid.clone(),
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
                    .filter_map(|v| g.node_weight(*v).clone())
                    .map(|w| w.id.clone())
                    .collect()
            })
            .collect(),
        sorted_tasks: indexes
            .iter()
            .flat_map(|ts| ts.into_iter())
            .filter_map(|id| g.node_weight(*id))
            .cloned()
            .collect(),
    }
}

pub struct SortedTasks {
    pub sorted_tasks: Vec<Task>,
    pub cycles: Vec<Vec<String>>,
}
