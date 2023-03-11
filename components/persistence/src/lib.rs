use std::env;
pub fn config(var: String, default: String) -> String {
    let name = var;
    match env::var(name) {
        Ok(v) => v,
        Err(_) => default,
    }
}

#[cfg(feature = "redis")]
pub mod persistence {
    extern crate redis;
    use std::{path::PathBuf, fs::File};
    use std::io::{prelude::*, BufWriter};

    use flate2::Compression;
    use flate2::write::GzEncoder;
    use redis::Commands;
    use tar::{Builder, Header};
    use indicatif::{ProgressBar, ProgressStyle};

    pub fn get(key: PathBuf) -> redis::RedisResult<String> {
        let namespace = crate::config("NAMESPACE".to_string(), "".to_string());
        let key = format!("{namespace}{}", key.as_path().display());
        let client = redis::Client::open("redis://127.0.0.1/")?;
        let mut con = client.get_connection()?;
        let v: String = con.get::<String, _>(key)?;
        Ok(v)
    }
    pub fn get_raw(key: PathBuf) -> redis::RedisResult<String> {
        let key = format!("{}", key.as_path().display());
        let client = redis::Client::open("redis://127.0.0.1/")?;
        let mut con = client.get_connection()?;
        let v: String = con.get::<String, _>(key)?;
        Ok(v)
    }
    pub fn put(key: PathBuf, value: &str) -> redis::RedisResult<()> {
        let namespace = crate::config("NAMESPACE".to_string(), "".to_string());
        let key = format!("{namespace}{}", key.as_path().display());
        let client = redis::Client::open("redis://127.0.0.1/")?;
        let mut con = client.get_connection()?;
        con.del(key.clone())?;
        con.set(key, value)?;
        Ok(())
    }
    pub fn save_table_as_files(prefix: String) {
        let mut safe_dir = PathBuf::new();
        safe_dir.push(prefix.clone());
        if ! safe_dir.exists() {
            std::fs::create_dir(prefix.clone()).unwrap();
        }
        if let Ok(client) = redis::Client::open("redis://127.0.0.1/") {
            if let Ok(mut con) = client.get_connection() {
                let files: Vec<String> = con.keys(format!("{prefix}/*")).unwrap();
                files.iter().for_each(|file| {
                    let content = get(PathBuf::from(file)).unwrap();
                    let f = File::create(file);
                    if let Ok(mut f) = f {
                        let _ = f.write_all(content.as_bytes());
                    }
                });
            }
        }
    }
    pub fn save_tables_as_gzip(tables: Vec<&str>) {
        let namespace = crate::config("NAMESPACE".to_string(), "".to_string());
        let output_file = File::create(format!("{namespace}.tar.gz")).unwrap();
        let encoder = GzEncoder::new(BufWriter::new(output_file), Compression::default());
        let mut tar = Builder::new(encoder);
        let mut header = Header::new_gnu();
        for table in tables {
            let prefix = format!("{namespace}{table}");
            if let Ok(client) = redis::Client::open("redis://127.0.0.1/") {
                if let Ok(mut con) = client.get_connection() {
                    let files: Vec<String> = con.keys(format!("{prefix}/*")).unwrap();
                    let pb = ProgressBar::new(files.len() as u64);
                    pb.set_style(
                        ProgressStyle::default_bar()
                            .template("{bar:40.cyan/blue} {percent}%")
                            .progress_chars("##-"),
                    );
                    let mut i = 0;
                    files.iter().for_each(|file| {
                        if let Ok(contents) = get_raw(PathBuf::from(file)) {
                            let contents = contents.as_bytes();
                            header.set_size(contents.len() as u64);
                            header.set_mode(0o644);
                            header.set_cksum();
                            tar.append_data(&mut header, file, contents).unwrap();
                        }
                        pb.set_position(i);
                        i = i + 1;
                    });
                    pb.finish();
                }
            }
        }
    }


}

#[cfg(feature = "tikv")]
pub mod persistence {
    use tikv_client::RawClient;
    use std::path::{PathBuf, Path};
    use futures::executor::block_on;
    pub async fn async_get(key: PathBuf) -> tikv_client::Result<String> {
        let namespace = crate::config("NAMESPACE".to_string(), "".to_string());
        let key = format!("{namespace}{}", key.as_path().display());
        let client = RawClient::new(vec!["127.0.0.1:2379"]).await?;
        if let Some(value) = client.get(key).await? {
            let v = String::from_utf8(value).unwrap(); 
            Ok(v)
        } else {
            Ok("".to_string())
        }
    }
    pub async fn async_put(key: PathBuf, value: &str) -> tikv_client::Result<()> {
        let namespace = crate::config("NAMESPACE".to_string(), "".to_string());
        let key = format!("{namespace}{}", key.as_path().display());
        let client = RawClient::new(vec!["127.0.0.1:2379"]).await?;
        client.put(key, value).await?;
        Ok(())
    }

    pub fn get(key: PathBuf) -> tikv_client::Result<String> {
        let v = async_get(key);
        block_on(v)
    }
    pub fn put(key: PathBuf, value: &str) -> tikv_client::Result<()> {
        let v = async_put(key, value);
        block_on(v)
    }
    #[tokio::main]
    async fn _main() -> tikv_client::Result<()> {
        let hello = Path::new("Hello").to_path_buf();
        async_put(hello.clone(), "World").await?;
        let value = async_get(hello).await?;
        assert_eq!(format!("Hello, {value}!"), "Hello, World!");
        Ok(())
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
    fn main() {
        let hello = Path::new("Hello").to_path_buf();
        let _ = persistence::put(hello.clone(), "World");
        if let Ok(value) = persistence::get(hello) {
            assert_eq!(format!("Hello, {value}!"), "Hello, World!");
        }
    }

}

#[allow(dead_code)]
fn main() {
    persistence::save_tables_as_gzip(vec!["safe", "unsafe"]);
}
