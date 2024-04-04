use crate::config::Config;
use crate::utils;
use crate::GenericResult;
use protocol::{Manifest, MediaType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;
use tokio::fs;
use tokio_util::io;
use uuid::Uuid;


#[derive(Debug)]
pub struct Image
{
    pub manifest: Manifest,
    pub image: String,
}

pub type RegistryMap = HashMap<Uuid, Image>;

#[derive(Debug)]
pub struct Registry
{
    content: RegistryMap,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageSerialized
{
    pub manifest: String,
    pub image: String,
}

impl Deref for Registry
{
    type Target = RegistryMap;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl DerefMut for Registry
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl Registry
{
    fn deserialize() -> GenericResult<Vec<ImageSerialized>>
    {
        let yaml = utils::file_read(&Config::readu().database)?;
        let reg: Vec<ImageSerialized> = serde_yaml::from_slice(&yaml)?;

        Ok(reg)
    }

    fn serialize(reg: Vec<ImageSerialized>) -> GenericResult<()>
    {
        let yaml = serde_yaml::to_string(&reg)?;
        utils::file_write(&Config::readu().database, yaml.as_bytes())?;

        Ok(())
    }

    // read the yaml, load json manifests and construct uuid keyed hashmap
    pub fn import() -> GenericResult<Self>
    {
        let reg = Registry::deserialize()?;

        let mut content = RegistryMap::new();

        for img in reg {
            let path = format!("{}/{}", Config::readu().server, img.manifest);
            let json = utils::file_read(&path)?;
            let manifest: Manifest = serde_json::from_slice(&json)?;
            let uuid = manifest.uuid;
            let image = Image {
                manifest,
                image: img.image,
            };

            content.insert(uuid, image);
        }

        Ok(Self { content })
    }

    pub fn get_manifest(&self, uuid: &Uuid) -> Option<&Manifest>
    {
        if self.contains_key(uuid) {
            Some(&self.content[uuid].manifest)
        } else {
            None
        }
    }

    pub async fn get_image(&self, uuid: &Uuid) -> Option<io::ReaderStream<fs::File>>
    {
        if self.contains_key(uuid) {
            let path = format!("{}/{}", Config::readu().server, &self.content[uuid].image);
            let file = match fs::File::open(&path).await {
                Ok(file) => file,
                Err(_err) => return None,
            };

            let stream = io::ReaderStream::new(file);
            Some(stream)
        } else {
            None
        }
    }

    pub fn generate_example() -> GenericResult<()>
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
                image: format!("{}.tgz", m.uuid),
            };
            vi.push(image);
        }

        Registry::serialize(vi)?;

        Ok(())
    }
}
