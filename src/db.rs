use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    datamodel::{Project, Task, TaskUpdate},
    hstable::HSTable,
    ids::unique_id,
};

pub struct Database {
    filename: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct FullDatabase {
    pub tasks: HSTable<Task>,
    pub projects: HSTable<Project>,
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

    pub fn projects(&self) -> Result<HSTable<Project>> {
        let db = self.load()?;
        Ok(db.projects)
    }

    pub fn project(&self, uid: &str) -> Result<Option<Project>> {
        let db = self.load()?;
        Ok(db.projects.get1(uid).cloned())
    }

    pub fn new_project(&self) -> Result<String> {
        let uid = unique_id();

        let mut db = self.load()?;
        db.projects.insert(Project {
            uid: uid.clone(),
            name: "New project".to_owned(),
        });
        self.save(&db)?;
        Ok(uid)
    }

    pub fn upsert_task(&self, update: TaskUpdate) -> Result<()> {
        assert!(!update.uid.is_empty());

        let mut db = self.load()?;
        let task_count = db.tasks.get_many(&update.project_uid).count();

        let uid = update.uid.clone();
        if let Some(existing) = db.tasks.get2(&update.project_uid, &uid) {
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
        db.tasks.remove2(project_uid, uid);
        self.save(&db)
    }

    pub fn with_project(&self, project_uid: &str, block: impl FnMut(&mut Project)) -> Result<()> {
        let mut db = self.load()?;
        let project = db.projects.get1_mut(project_uid);
        project.map(block);
        self.save(&db)
    }

    pub fn with_task(
        &self,
        project_uid: &str,
        uid: &str,
        block: impl FnMut(&mut Task),
    ) -> Result<()> {
        let mut db = self.load()?;
        let task = db.tasks.get2_mut(project_uid, uid);
        task.map(block);
        self.save(&db)
    }
}
