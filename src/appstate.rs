use native_db::{native_db, InnerKeyValue};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct Task {
    #[primary_key]
    pub uid: String,
    pub id: String,
    pub title: String,
    pub estimate: Option<u32>,
    pub risk: Option<Risk>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromFormField)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Low,
    Medium,
    High,
}
