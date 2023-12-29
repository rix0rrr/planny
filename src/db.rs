use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    datamodel::{Task, TaskUpdate},
    hstable::HSTable,
};

pub struct Database {
    filename: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct FullDatabase {
    pub tasks: HSTable<Task>,
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

    pub fn tasks(&self) -> Result<HSTable<Task>> {
        let db = self.load()?;
        Ok(db.tasks)
    }

    pub fn upsert_task(&self, update: TaskUpdate) -> Result<()> {
        assert!(!update.uid.is_empty());

        let mut db = self.load()?;
        let task_count = db.tasks.get_many(&update.project_uid).count();

        let uid = update.uid.clone();
        if let Some(existing) = db.tasks.get(&update.project_uid, &uid) {
            db.tasks.insert(update.apply(existing));
        } else {
            let mut task = update.apply(&Default::default());
            // Make sure every task has an ID
            task.ensure_defaults(task_count);
            db.tasks.insert(task);
        }
        self.save(&db)
    }

    pub fn delete_task(&self, project_uid: &str, uid: &str) -> Result<()> {
        let mut db = self.load()?;
        db.tasks.remove(project_uid, uid);
        self.save(&db)
    }

    pub fn with_task(
        &self,
        project_uid: &str,
        uid: &str,
        block: impl FnMut(&mut Task),
    ) -> Result<()> {
        let mut db = self.load()?;
        let task = db.tasks.get_mut(project_uid, uid);
        task.map(block);
        self.save(&db)
    }
}
