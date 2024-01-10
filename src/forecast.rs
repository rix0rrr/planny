use itertools::Itertools;
use rand::seq::IteratorRandom; // 0.7.2
use std::{cmp::max, collections::HashMap, ops::Range};

use quantiles::ckms::CKMS;

use crate::{datamodel::Task, topo_queue::TopoQueue};

const N: u32 = 1000;
const ERROR: f64 = 0.001;

pub struct SimulationResults {
    pub task_stats: HashMap<String, Range<CKMS<f64>>>,
}

pub fn simulate_tasks(tasks: impl Iterator<Item = Task>, people: u32) -> SimulationResults {
    let tasks: HashMap<String, Task> = HashMap::from_iter(tasks.map(|t| (t.uid.clone(), t)));
    let mut stats: HashMap<String, Range<CKMS<f64>>> = HashMap::from_iter(
        tasks
            .keys()
            .map(|k| (k.clone(), CKMS::new(ERROR)..CKMS::new(ERROR))),
    );

    let mut rng = rand::thread_rng();

    let start_queue = queue_from_tasks(tasks.values());
    for _ in 0..N {
        // FIXME: Parallellism between peeps

        let mut plan: HashMap<String, Range<u32>> = HashMap::new();
        let mut queue = start_queue.clone();
        let mut people = PeopleAllocation::new(people);

        let mut t = 0;

        while !queue.is_empty() {
            let Some(task) = queue
                .available()
                .choose(&mut rng)
                .and_then(|t| tasks.get(t))
            else {
                break;
            };

            // FIXME: Draw duration from distribution
            let duration = task.estimate.unwrap_or(1);

            // Find personnel to carry out this task (FIXME: randomize?)
            let (person, start) = people.availabilities(t).next().unwrap();

            let end = t + duration;

            people.book(person, start..end);
            plan.insert(task.uid.clone(), start..end);
            t = people.earliest_availability();

            queue.remove(&task.uid);

            let stats = stats.get_mut(&task.uid).unwrap();
            stats.start.insert(start as f64);
            stats.end.insert(end as f64);
        }
    }

    SimulationResults { task_stats: stats }
}

pub fn queue_from_tasks<'a>(tasks: impl Iterator<Item = &'a Task>) -> TopoQueue {
    TopoQueue::from_iter(tasks.map(|t| (t.uid.clone(), t.dependencies.clone())))
}

type Interval = Range<u32>;

type Schedule = Vec<Interval>;

#[derive(Debug, Clone)]
struct PeopleAllocation {
    schedules: Vec<Schedule>,
    booked_until: Vec<u32>,
}

impl PeopleAllocation {
    pub fn new(n: u32) -> Self {
        PeopleAllocation {
            schedules: (0..n).map(|_| vec![]).collect(),
            booked_until: (0..n).map(|_| 0).collect(),
        }
    }

    /// Return the availabilities for all people starting at a given t, earliest available first
    pub fn availabilities(&self, start: u32) -> impl Iterator<Item = (usize, u32)> + '_ {
        self.booked_until
            .iter()
            .map(move |b| max(*b, start))
            .enumerate()
            .sorted_by_key(|(_, v)| *v)
    }

    pub fn earliest_availability(&self) -> u32 {
        self.booked_until.iter().min().copied().unwrap()
    }

    pub fn book(&mut self, i: usize, interval: Interval) {
        if self.schedules[i].iter().any(|v| overlaps(&interval, v)) {
            panic!("Oops, double booking!");
        }
        self.booked_until[i] = max(self.booked_until[i], interval.end);
        self.schedules[i].push(interval);
    }
}

fn overlaps<A: PartialOrd>(a: &Range<A>, b: &Range<A>) -> bool {
    a.start < b.end && b.start < a.end
}
