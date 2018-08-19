use std::{env, error, ffi::OsStr, fs, io, path::PathBuf};

lazy_static! {
    static ref EXE_PATH: PathBuf = env::current_exe().unwrap();
}

/// Represents a type that can be loaded from a file or byte sequence.
pub trait Loadable
where
    Self: Sized,
{
    /// Type of an additional argument that can be supplied to the `load` methods.
    ///
    /// Can use () if additional options are not needed.
    type LoadOptions;

    /// Error type that can be by the `load` methods.
    ///
    /// Use io::Error if loading from bytes cannot fail.
    type LoadError: error::Error + From<io::Error>;

    /// Loads the object from a byte sequence.
    fn load_from_bytes(data: &[u8], options: Self::LoadOptions) -> Result<Self, Self::LoadError>;

    /// Loads the object from a file.
    ///
    /// Supplied `path` is relative to the game executable
    /// directory.
    fn load_from_file<P: AsRef<OsStr>>(
        path: P,
        options: Self::LoadOptions,
    ) -> Result<Self, Self::LoadError> {
        let mut full_path = EXE_PATH.clone();
        full_path.set_file_name(path);

        Self::load_from_bytes(&fs::read(&full_path)?, options)
    }
}
