use failure::Error;
use std::{env, fs, path::PathBuf};

lazy_static! {
    static ref EXE_PATH: PathBuf = env::current_exe().unwrap();
}

#[derive(Debug, Fail)]
pub enum AssetError {
    /// The requested asset name was not found.
    #[fail(display = "No asset named '{}' found.", _0)]
    NameNotFound(String),
}

pub trait Asset<TOptions>
where
    Self: Sized,
{
    fn load_from_bytes(data: &[u8], options: TOptions) -> Result<Self, Error>;
}

pub struct AssetHandle {
    name: String,
    path: PathBuf,
}

pub struct AssetDatabase {
    assets: Vec<AssetHandle>,
}

impl AssetDatabase {
    pub fn get<T: Asset<TOptions>, TOptions>(&self, name: &str) -> Result<T, Error> {
        match self.get_handle(name) {
            None => Err(AssetError::NameNotFound(name.to_owned()).into()),
            Some(handle) => {
                let mut full_path = EXE_PATH.clone();
                full_path.set_file_name(handle.path);

                T::load_from_bytes(fs::read(full_path));
            }
        }
    }

    fn get_handle(&self, name: &str) -> Option<&AssetHandle> {
        self.assets.iter().find(|handle| handle.name == name)
    }
}
