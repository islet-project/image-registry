use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum MediaType {
    Docker,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub uuid: Uuid,
    pub name: String,
    pub vendor: String,
    pub media_type: MediaType,
}

impl Manifest {
    pub fn new(name: String, vendor: String, media_type: MediaType) -> Manifest {
        Manifest {
            uuid: Uuid::new_v4(),
            name,
            vendor,
            media_type,
        }
    }
}
