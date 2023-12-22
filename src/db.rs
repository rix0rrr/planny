use anyhow::Result;
use native_db::DatabaseBuilder;

use crate::appstate::Task;

pub struct Database {
    builder: DatabaseBuilder,
}

impl Database {
    pub fn new() -> Result<Self> {
        let mut builder = DatabaseBuilder::new();
        builder.define::<Task>()?;
        Ok(Self { builder })
    }

    pub fn open(&self) -> Result<native_db::Database<'_>> {
        let filename = "db.db";
        match self.builder.open(filename) {
            Ok(db) => Ok(db),
            Err(_) => Ok(self.builder.create(filename)?),
        }
    }

    pub fn tasks(&self) -> Result<Vec<Task>> {
        let db = self.open()?;
        let tx = db.r_transaction()?;
        let ts = tx.scan().primary::<Task>()?.all().collect();
        Ok(ts)
    }

    pub fn upsert_task(&self, task: Task) -> Result<()> {
        let db = self.open()?;
        let tx = db.rw_transaction()?;
        if let Some(existing) = tx.get().primary::<Task>(task.uid.clone())? {
            tx.update(existing, task)?;
        } else {
            tx.insert(task)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn delete_task(&self, uid: &str) -> Result<()> {
        let db = self.open()?;
        let tx = db.rw_transaction()?;
        if let Some(existing) = tx.get().primary::<Task>(uid)? {
            tx.remove(existing)?;
        }
        tx.commit()?;
        Ok(())
    }
}
