use crate::config::Config;
use crate::utils;
use crate::GenericResult;
use protocol::{Manifest, MediaType};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::path::Path;

type Registry = HashMap<Uuid, Image>;

#[derive(Debug)]
pub struct Image
{
    pub manifest: Manifest,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageSerialized
{
    pub manifest: String,
    pub content: String,
}

pub fn deserialize() -> GenericResult<Vec<ImageSerialized>>
{
    let yaml = utils::file_read(&Config::readu().images)?;
    let reg: Vec<ImageSerialized> = serde_yaml::from_slice(&yaml)?;

    Ok(reg)
}

pub fn serialize(reg: Vec<ImageSerialized>) -> GenericResult<()>
{
    let yaml = serde_yaml::to_string(&reg)?;
    utils::file_write(&Config::readu().images, yaml.as_bytes())?;

    Ok(())
}

// read the yaml, load json manifests and construct uuid keyed hashmap
pub fn parse_to_hashmap(reg: Vec<ImageSerialized>) -> GenericResult<Registry>
{
    let mut ret = Registry::new();

    for img in reg {
        let path = format!("{}/{}", Config::readu().server, img.manifest);
        let json = utils::file_read(&path)?;
        let manifest: Manifest = serde_json::from_slice(&json)?;
        let uuid = manifest.uuid;
        let image = Image {
            manifest,
            content: img.content,
        };

        ret.insert(uuid, image);
    }

    Ok(ret)
}

pub fn generate_registry() -> GenericResult<()>
{
    let server = Config::readu().server.clone();
    let path = Path::new(&server);
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    std::fs::create_dir(path)?;

    let mut vm = Vec::new();
    for name in ["application1", "application2", "application3"] {
        let manifest = Manifest::new(name.to_string(), "Samsung".to_string(), MediaType::Docker);
        vm.push(manifest);
    }

    let mut vj = Vec::new();
    for elem in &vm {
        vj.push(serde_json::to_string(elem)?);
    }

    let mut vi = Vec::new();
    for (m, j) in std::iter::zip(vm, vj) {
        utils::file_write(&format!("{}/{}.json", Config::readu().server, m.uuid), j.as_bytes())?;
        let image = ImageSerialized {
            manifest: format!("{}.json", m.uuid),
            content: format!("{}.tgz", m.uuid),
        };
        vi.push(image);
    }

    serialize(vi)?;

    Ok(())
}
