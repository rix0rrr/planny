use std::collections::{hash_map::Entry, HashMap, HashSet};

use itertools::Itertools;

type IdSet = HashSet<String>;
type TaskMap = HashMap<String, IdSet>;

#[derive(Clone, Debug, Default)]
pub struct TopoQueue {
    current_deps: TaskMap,
    unblocks: TaskMap,
    available: HashSet<String>,
}

impl TopoQueue {
    pub fn from_iter(iter: impl IntoIterator<Item = (String, HashSet<String>)>) -> Self {
        let deps = HashMap::from_iter(iter);
        let mut ret = TopoQueue {
            current_deps: deps.clone(),
            ..Default::default()
        };

        // Invert the dependencies
        for (key, deps) in &ret.current_deps {
            for dep in deps {
                match ret.unblocks.entry(dep.clone()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().insert(key.clone());
                    }
                    Entry::Vacant(e) => {
                        e.insert(HashSet::from_iter([key.clone()]));
                    }
                }
            }
        }
        ret.update_available();

        ret
    }

    pub fn is_empty(&self) -> bool {
        self.available.len() + self.current_deps.len() == 0
    }

    pub fn available(&self) -> impl Iterator<Item = &String> {
        self.available.iter()
    }

    pub fn remove(&mut self, x: &str) {
        assert!(self.available.contains(x));
        self.available.remove(x);
        if let Some(unblocks) = self.unblocks.remove(x) {
            for t in unblocks {
                self.current_deps.get_mut(&t).unwrap().remove(x);
            }
        }
        self.update_available();
    }

    fn update_available(&mut self) {
        let available = self
            .current_deps
            .iter()
            .filter(|(_, d)| d.is_empty())
            .map(|(k, _)| k)
            .cloned()
            .collect_vec();
        for k in &available {
            self.current_deps.remove(k);
        }
        self.available.extend(available);
    }
}
