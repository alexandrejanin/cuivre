use std::{
    env, error,
    fmt::{self, Display, Formatter},
    fs, io,
    path::{Path, PathBuf},
};

///Errors related to resource loading.
#[derive(Debug)]
pub enum ResourceError {
    ///The ResourceLoader could not find the path to the current executable.
    ExecutablePathNotFound,
    ///Error related to IO operations.
    Io(io::Error),
}

impl Display for ResourceError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ResourceError::ExecutablePathNotFound => {
                write!(f, "Error: Could not locate executable path.")
            }
            ResourceError::Io(error) => write!(f, "{}", error),
        }
    }
}

impl error::Error for ResourceError {
    fn cause(&self) -> Option<&error::Error> {
        match self {
            ResourceError::Io(error) => Some(error),

            _ => None,
        }
    }
}

/// Represents a type that can be loaded from a byte sequence
/// (often from a file).
pub trait Loadable
where
    Self: Sized,
{
    /// Type of an additional argument that can be supplied to the `load` method.
    ///
    /// Can use () if additional options are not needed for the type.
    type LoadOptions;

    /// Type returned by the `load` method.
    ///
    /// Will usually be either `Self` if the loading cannot fail,
    /// or some form of `Result<Self, Error>` if the loading can fail.
    type LoadResult;

    /// Loads the object from a byte sequence.
    fn load(data: &[u8], options: Self::LoadOptions) -> Self::LoadResult;
}

///Loads and manages resource files: text, images etc.
pub struct ResourceLoader {
    res_root: PathBuf,
}

impl ResourceLoader {
    /// Attempts to create a new ResourceLoader with `root` as the root directory for all resources.
    /// Can fail if the executable directory cannot be acquired.
    pub fn new(root: &Path) -> Result<ResourceLoader, ResourceError> {
        //Get path to executable
        let executable_name = env::current_exe().map_err(ResourceError::Io)?;

        //Get parent dir
        let executable_dir = executable_name
            .parent()
            .ok_or(ResourceError::ExecutablePathNotFound)?;

        //Get resources dir
        let res_root = executable_dir.join(root);

        Ok(ResourceLoader { res_root })
    }

    ///Deserializes a config object from a file
    pub fn load<T: Loadable>(
        &self,
        path: &Path,
        options: T::LoadOptions,
    ) -> Result<T::LoadResult, ResourceError> {
        Ok(T::load(&self.load_bytes(path)?, options))
    }

    fn load_bytes(&self, path: &Path) -> Result<Vec<u8>, ResourceError> {
        let path = self.get_path(path);

        fs::read(&path).map_err(ResourceError::Io)
    }

    ///Returns absolute path when provided with a path relative to the resources root directory.
    fn get_path(&self, path: &Path) -> PathBuf {
        self.res_root.join(path)
    }
}
