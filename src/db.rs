use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Debug,
    fs::{File, OpenOptions},
    hash::Hash,
    io::Write,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Db<K, V> {
    path: PathBuf,
    db: HashMap<K, V>,
}

impl<K, V> Db<K, V>
where
    K: Hash + Eq + Serialize + for<'de> Deserialize<'de> + Debug,
    V: Serialize + for<'de> Deserialize<'de> + Debug,
{
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = PathBuf::from(path.as_ref());
        let db = if path.exists() {
            let file = File::open(&path)?;
            bincode::deserialize_from(file)?
        } else {
            HashMap::new()
        };

        Ok(Self { path, db })
    }

    pub fn flush(&self) -> anyhow::Result<()> {
        let data = bincode::serialize(&self.db)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl<K, V> Deref for Db<K, V> {
    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl<K, V> DerefMut for Db<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}
