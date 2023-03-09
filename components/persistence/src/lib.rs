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

#[cfg(feature = "tikv")]
pub mod persistence {
    use tikv_client::RawClient;
    use std::path::PathBuf;
    use futures::executor::block_on;

    pub fn get(key: PathBuf) -> tikv_client::Result<String> {
        let v = async {
            let client = RawClient::new(vec!["127.0.0.1:2379"]).await?;
            let value = client.get(format!("{:?}",key)).await?;
            let v = String::from_utf8(value.unwrap()).unwrap();
            Ok(v)
        };
        block_on(v)
    }
    pub fn put(key: PathBuf, value: &str) -> tikv_client::Result<()> {
        let v = async {
            let client = RawClient::new(vec!["127.0.0.1:2379"]).await?;
            client.put(format!("{:?}",key), value).await?;
            Ok(())
        };
        block_on(v)
    }
}

#[cfg(not(feature = "tikv"))]
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
    fn main() -> tikv_client::Result<()> {
        let hello = Path::new("Hello").to_path_buf();
        let _ = persistence::put(hello.clone(), "World");
        let value = persistence::get(hello).unwrap();
        assert_eq!(format!("Hello, {value}!"), "Hello, World!");
        Ok(())
    }
}
