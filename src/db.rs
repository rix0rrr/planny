use std::{collections::HashMap, fs};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::datamodel::{Task, TaskUpdate};

pub struct Database {
    filename: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct FullDatabase {
    pub tasks: HashMap<String, Task>,
}

impl Database {
    pub fn new(filename: String) -> Result<Self> {
        Ok(Self { filename })
    }

    fn load(&self) -> Result<FullDatabase> {
        match fs::File::open(&self.filename) {
            Ok(f) => Ok(serde_json::from_reader(f)?),
            Err(_) => Ok(FullDatabase::default()),
        }
    }

    fn save(&self, db: &FullDatabase) -> Result<()> {
        let f = fs::File::create(&self.filename)?;
        serde_json::to_writer_pretty(f, &db)?;
        Ok(())
    }

    pub fn tasks(&self) -> Result<HashMap<String, Task>> {
        let db = self.load()?;
        Ok(db.tasks)
    }

    pub fn upsert_task(&self, update: TaskUpdate) -> Result<()> {
        assert!(update.uid != "");

        let mut db = self.load()?;
        let task_count = db.tasks.len();

        let uid = update.uid.clone();
        if let Some(existing) = db.tasks.get(&uid) {
            db.tasks.insert(uid, update.apply(&existing));
        } else {
            let mut task = update.apply(&Default::default());
            // Make sure every task has an ID
            task.ensure_defaults(task_count);
            db.tasks.insert(uid, task);
        }
        self.save(&db)
    }

    pub fn delete_task(&self, uid: &str) -> Result<()> {
        let mut db = self.load()?;
        db.tasks.remove(uid);
        self.save(&db)
    }

    pub fn with_task(&self, uid: &str, block: impl FnMut(&mut Task)) -> Result<()> {
        let mut db = self.load()?;
        let task = db.tasks.get_mut(uid);
        task.map(block);
        self.save(&db)
    }
}
