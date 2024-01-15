use itertools::Itertools;
use rand::seq::IteratorRandom;
use rand_distr::{Distribution, LogNormal}; // 0.7.2
use std::{cmp::max, collections::HashMap, ops::Range};

use quantogram::Quantogram;

use crate::{
    datamodel::{Risk, Task, TaskType},
    topo_queue::TopoQueue,
};

const N: u32 = 1000;

pub struct SimulationResults {
    pub task_stats: HashMap<String, Range<Quantogram>>,
}

pub fn simulate_tasks(tasks: impl Iterator<Item = Task>, people: u32) -> SimulationResults {
    let tasks: HashMap<String, Task> = HashMap::from_iter(tasks.map(|t| (t.uid.clone(), t)));
    let mut stats: HashMap<String, Range<Quantogram>> = HashMap::from_iter(
        tasks
            .keys()
            .map(|k| (k.clone(), Quantogram::new()..Quantogram::new())),
    );

    let mut rng = rand::thread_rng();

    let start_queue = queue_from_tasks(tasks.values());
    for _ in 0..N {
        // FIXME: Parallellism between peeps

        let mut plan: HashMap<String, Range<u32>> = HashMap::new();
        let mut queue = start_queue.clone();
        let mut people = PeopleAllocation::new(people);

        while !queue.is_empty() {
            let Some(task) = queue
                .available()
                .choose(&mut rng)
                .and_then(|t| tasks.get(t))
            else {
                break;
            };

            let duration = if task.r#type == TaskType::Milestone {
                0
            } else {
                // Based in this guy's musings:
                // https://erikbern.com/2019/04/15/why-software-projects-take-longer-than-you-think-a-statistical-model.html
                let estimated_duration = task.estimate.unwrap_or(1.0);

                let sigma = match task.risk.as_ref().copied().unwrap_or_default() {
                    Risk::Low => 0.25,
                    Risk::Medium => 0.5,
                    Risk::High => 1.0,
                };

                let log_normal = LogNormal::from_mean_cv(1.0, sigma).unwrap();

                let blowup = log_normal.sample(&mut rand::thread_rng());
                (estimated_duration * blowup).ceil() as u32
            };

            // Start time is the max of the end time of all dependencies
            let dependencies_end = task
                .dependencies
                .iter()
                .map(|id| plan[id].end)
                .max()
                .unwrap_or_default();

            // Find personnel to carry out this task (FIXME: randomize?)
            let (person, start) = people.availabilities(dependencies_end).next().unwrap();

            let end = start + duration;

            people.book(person, start..end);
            plan.insert(task.uid.clone(), start..end);

            queue.remove(&task.uid);

            let stats = stats.get_mut(&task.uid).unwrap();
            stats.start.add(start as f64);
            stats.end.add(end as f64);
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

/**
 * Query the range of the CKMS
 *
 * Discard the outer quantiles by some fraction
 */
pub fn query_minmax(ckms: &Quantogram, margin: f64) -> Range<f64> {
    (ckms.quantile(margin).unwrap())..(ckms.quantile(1.0 - margin).unwrap())
}

pub fn convert_rng(rng: &Range<f64>) -> Range<u32> {
    (rng.start as u32)..(rng.end.ceil() as u32)
}

// P(working) = P(started) * (1 - P(stopped))
