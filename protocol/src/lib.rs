use serde::{Serialize, Serializer, Deserialize, Deserializer};
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
    #[serde(serialize_with = "to_hex", deserialize_with = "from_hex")]
    pub root_of_trust: Vec<u8>,
}

impl Manifest {
    pub fn new(name: String, vendor: String, media_type: MediaType) -> Manifest {
        Manifest {
            uuid: Uuid::new_v4(),
            name,
            vendor,
            media_type,
            root_of_trust: vec![0; 32],
        }
    }
}

pub fn to_hex<T, S>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error>
where T: AsRef<[u8]>,
      S: Serializer
{
    serializer.serialize_str(&hex::encode(&buffer))
}

pub fn from_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where D: Deserializer<'de>
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| hex::decode(&string).map_err(|err| Error::custom(err.to_string())))
}
