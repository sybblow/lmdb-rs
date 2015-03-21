//! Idiomatic and safe APIs for interacting with the
//! [Symas Lightning Memory-Mapped Database (LMDB)](http://symas.com/mdb/).

#![feature(core, libc, optin_builtin_traits, path, std_misc, unsafe_destructor)]
#![cfg_attr(test, feature(fs, io, tempdir, test))]

extern crate libc;
extern crate "lmdb-sys" as ffi;

#[cfg(test)] extern crate rand;
#[cfg(test)] extern crate test;
#[macro_use] extern crate bitflags;

pub use cursor::{
    Cursor,
    CursorExt,
    RoCursor,
    RwCursor
};
pub use database::Database;
pub use environment::{Environment, EnvironmentBuilder};
pub use error::{Error, Result};
pub use flags::*;
pub use transaction::{
    InactiveTransaction,
    RoTransaction,
    RwTransaction,
    Transaction,
    TransactionExt,
};

macro_rules! lmdb_try {
    ($expr:expr) => ({
        match $expr {
            ::ffi::MDB_SUCCESS => (),
            err_code => return Err(::std::error::FromError::from_error(::Error::from_err_code(err_code))),
        }
    })
}

macro_rules! lmdb_try_with_cleanup {
    ($expr:expr, $cleanup:expr) => ({
        match $expr {
            ::ffi::MDB_SUCCESS => (),
            err_code => {
                let _ = $cleanup;
                return Err(::std::error::FromError::from_error(::Error::from_err_code(err_code)))
            },
        }
    })
}

mod flags;
mod cursor;
mod database;
mod environment;
mod error;
mod transaction;

#[cfg(test)]
mod test_utils {
    use std;

    use std::fs;
    use std::fs::{create_dir, remove_dir_all};
    use std::path::{Path, PathBuf, AsPath};
    use rand::{thread_rng, Rng};

    use super::*;

    struct MyTempDir {
        path: Option<PathBuf>,
    }

    impl MyTempDir {
        pub fn new() -> std::io::Result<MyTempDir> {
            let mut path = PathBuf::new("/tmp/lmdb_tmpfs");
            let s: String = thread_rng().gen_ascii_chars().take(10).collect();
            path.push(s.as_slice());
            // println!("create dir: {:?}", path);
            match create_dir(&path) {
                Ok(_) => Ok(MyTempDir { path: Some(path) }),
                Err(e) => Err(e),
            }
        }

        pub fn path(&self) -> &Path {
            self.path.as_ref().unwrap()
        }

        fn cleanup_dir(&mut self) -> std::io::Result<()> {
            match self.path {
                Some(ref p) => fs::remove_dir_all(p),
                None => Ok(())
            }
        }
    }

    impl Drop for MyTempDir {
        fn drop(&mut self) {
            // println!("delete dir: {:?}", self.path());
            let _ = self.cleanup_dir();
        }
    }

    pub fn get_key(n: u32) -> String {
        format!("key{}", n)
    }

    pub fn get_data(n: u32) -> String {
        format!("data{}", n)
    }

    pub fn setup_bench_db<'a>(num_rows: u32) -> (MyTempDir, Environment) {
        let dir = MyTempDir::new().unwrap();
        let env = Environment::new().open(dir.path()).unwrap();

        {
            let db = env.open_db(None).unwrap();
            let mut txn = env.begin_rw_txn().unwrap();
            for i in range(0, num_rows) {
                txn.put(db,
                        get_key(i).as_bytes(),
                        get_data(i).as_bytes(),
                        WriteFlags::empty())
                    .unwrap();
            }
            txn.commit().unwrap();
        }
        (dir, env)
    }
}
