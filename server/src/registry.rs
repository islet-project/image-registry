use crate::config::Config;
use crate::utils;
use crate::GenericResult;
use protocol::{Manifest, MediaType};
use serde::{Serialize, Deserialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Image
{
    pub manifest: String,
    pub image: String,
}

pub fn load() -> GenericResult<Vec<Image>>
{
    let file = utils::file_read(&Config::readu().images)?;
    let reg: Vec<Image> = serde_yaml::from_str(&String::from_utf8(file)?)?;

    Ok(reg)
}

pub fn save(reg: Vec<Image>) -> GenericResult<()>
{
    let yaml = serde_yaml::to_string(&reg)?;
    utils::file_write(&Config::readu().images, yaml.as_bytes())?;

    Ok(())
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
        let image = Image {
            manifest: format!("{}.json", m.uuid),
            image: format!("{}.tgz", m.uuid),
        };
        vi.push(image);
    }

    save(vi)?;

    Ok(())
}
