use image;
use ron;
use serde::de::DeserializeOwned;
use std::{
    self,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

///Errors related to resource loading.
#[derive(Debug)]
pub enum ResourceError {
    ///The ResourceLoader could not find the path to the current executable.
    ExecutablePathNotFound,
    Io(PathBuf, io::Error),
    Image(PathBuf, image::ImageError),
    Ron(PathBuf, ron::de::Error),
}

impl Display for ResourceError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ResourceError::ExecutablePathNotFound => {
                write!(f, "Error: Could not locate executable path.")
            }
            ResourceError::Io(path, error) => write!(f, "{:?}: {}", path, error),
            ResourceError::Image(path, error) => write!(f, "{:?}: {}", path, error),
            ResourceError::Ron(path, error) => write!(f, "{:?}: {}", path, error),
        }
    }
}

impl std::error::Error for ResourceError {
    fn cause(&self) -> Option<&std::error::Error> {
        match self {
            ResourceError::Io(_, error) => Some(error),
            ResourceError::Image(_, error) => Some(error),
            ResourceError::Ron(_, error) => Some(error),

            _ => None,
        }
    }
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
        let executable_name =
            std::env::current_exe().map_err(|error| ResourceError::Io(root.to_owned(), error))?;

        //Get parent dir
        let executable_dir = executable_name
            .parent()
            .ok_or(ResourceError::ExecutablePathNotFound)?;

        //Get resources dir
        let res_root = executable_dir.join(root);

        Ok(ResourceLoader { res_root })
    }

    ///Deserializes a config object from a file
    pub fn load_object<T: DeserializeOwned>(&self, path: &Path) -> Result<T, ResourceError> {
        let text = self.load_string(path)?;

        ron::de::from_str::<T>(&text).map_err(|error| ResourceError::Ron(path.to_owned(), error))
    }

    ///Load an image from a PNG file.
    pub fn load_png(&self, path: &Path) -> Result<image::RgbaImage, ResourceError> {
        let path = self.get_path(path);

        Ok(image::open(&path)
            .map_err(|error| ResourceError::Image(path.to_owned(), error))?
            .to_rgba())
    }

    ///Load a String from a file.
    pub fn load_string(&self, path: &Path) -> Result<String, ResourceError> {
        //Open file
        let mut file = self
            .get_file(path)
            .map_err(|error| ResourceError::Io(path.to_owned(), error))?;

        //Allocate string
        let mut string = String::new();

        //Read file to string
        file.read_to_string(&mut string)
            .map_err(|error| ResourceError::Io(path.to_owned(), error))?;

        Ok(string)
    }

    ///Returns absolute path when provided with a path relative to the resources root directory.
    fn get_path(&self, path: &Path) -> PathBuf {
        self.res_root.join(path)
    }

    ///Opens file from resources root directory
    fn get_file(&self, path: &Path) -> io::Result<File> {
        let file_path = self.get_path(path);
        fs::File::open(file_path)
    }
}
