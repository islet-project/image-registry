use crate::config::Config;
use crate::utils::{self, file_write};
use crate::GenericResult;
use protocol::{Manifest, MediaType};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Image
{
    pub manifest: String,
    pub image: String,
}

pub fn load() -> GenericResult<Vec<Image>>
{
    Ok(Vec::new())
}

pub fn save(reg: Vec<Image>) -> GenericResult<()>
{
    let yaml = serde_yaml::to_string(&reg)?;
    file_write(&Config::readu().images, yaml.as_bytes())?;

    Ok(())
}

#[allow(dead_code)]
pub fn generate_registry() -> GenericResult<()>
{
    let mut vm = Vec::new();
    let mut vj = Vec::new();
    let mut vi = Vec::new();

    for name in ["application1", "application2", "application3"] {
        let manifest = Manifest::new(name.to_string(), "Samsung".to_string(), MediaType::Docker);
        vm.push(manifest);
    }

    for elem in &vm {
        vj.push(serde_json::to_string(elem)?);
    }

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
