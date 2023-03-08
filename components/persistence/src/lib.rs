#[cfg(feature = "redis")]
pub mod persistence {
    extern crate redis;
    use std::path::PathBuf;

    use redis::Commands;
    pub fn get(key: PathBuf) -> redis::RedisResult<String> {
        let client = redis::Client::open("redis://127.0.0.1/")?;
        let mut con = client.get_connection()?;
        let v: String = con.get::<String, _>(format!("{:?}",key))?;
        Ok(v)
    }
    pub fn put(key: PathBuf, value: &str) -> redis::RedisResult<()> {
        let client = redis::Client::open("redis://127.0.0.1/")?;
        let mut con = client.get_connection()?;
        con.del(format!("{:?}",key))?;
        con.set(format!("{:?}",key), value)?;
        Ok(())
    }
}

#[cfg(not(feature = "redis"))]
pub mod persistence {
    use std::{result::Result, io::Error, path::PathBuf, fs::read_to_string};
    pub fn get(key: PathBuf) -> Result<String, Error> {
        read_to_string(key)
    }
    pub fn put(key: PathBuf, value: &str) -> Result<(), Error> {
            if let Some(p) = key.parent() {
                if p.to_str().unwrap() != "" && ! p.exists() {
                    std::fs::create_dir(p)?;
                }
            }
            std::fs::write(&key, value)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::persistence;

    #[test]
    fn main() {
        let hello = Path::new("Hello").to_path_buf();
        let _ = persistence::put(hello.clone(), "World");
        if let Ok(value) = persistence::get(hello) {
            assert_eq!(format!("Hello, {value}!"), "Hello, World!");
        }
    }
}
