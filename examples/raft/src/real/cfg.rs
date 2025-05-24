use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    path::{Path, PathBuf},
};

use mc::Address;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
struct RawConfig {
    routes: Vec<(String, String)>,
    me: usize,
    dir: String,
}

impl RawConfig {
    pub fn from_file(filename: impl AsRef<Path>) -> Option<Self> {
        let mut cfg: PathBuf = MANIFEST_DIR.into();
        cfg.push(filename);
        std::fs::File::open(cfg)
            .ok()
            .and_then(|file| serde_json::from_reader(file).ok())
            .map(|mut c: RawConfig| {
                let mut cfg: PathBuf = MANIFEST_DIR.into();
                cfg.push(c.dir);
                c.dir = cfg.to_string_lossy().to_string();
                c
            })
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Config {
    pub routes: HashMap<Address, SocketAddr>,
    pub me: Address,
    pub my_id: usize,
    pub dir: String,
}

impl From<RawConfig> for Option<Config> {
    fn from(value: RawConfig) -> Option<Config> {
        let mut routes: HashMap<_, _> = Default::default();
        let my_id = value.me;
        for (addr, sock) in value.routes.iter() {
            let addr: Address = addr.into();
            let sock = sock.to_socket_addrs().ok()?.next()?;
            routes.insert(addr, sock);
        }
        let me: Address = value.routes[value.me].0.clone().into();
        Some(Config {
            routes,
            my_id,
            me,
            dir: value.dir,
        })
    }
}

impl Config {
    pub fn from_file(filename: impl AsRef<Path>) -> Option<Self> {
        RawConfig::from_file(filename).and_then(Option::<Config>::from)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::real::cfg::Config;

    use super::RawConfig;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn works() {
        let s = serde_json::to_string(&RawConfig {
            routes: vec![("node:proc".into(), "127.0.0.1:10094".into())],
            me: 0,
            dir: ".sys/n1".into(),
        })
        .unwrap();

        print!("{s}");

        let n1 = r#"{
            "routes": [
                [
                    "n1:raft",
                    "127.0.0.1:10094"
                ],
                [
                    "n2:raft",
                    "127.0.0.1:10095"
                ],
                [
                    "n3:raft",
                    "127.0.0.1:10096"
                ]
            ],
            "me": 0,
            "dir": ".sys/n1"
        }"#;

        let cfg: RawConfig = serde_json::from_str(n1).unwrap();
        assert_eq!(
            cfg.routes,
            vec![
                ("n1:raft".into(), "127.0.0.1:10094".into()),
                ("n2:raft".into(), "127.0.0.1:10095".into()),
                ("n3:raft".into(), "127.0.0.1:10096".into())
            ]
        );
        assert_eq!(cfg.me, 0);
        assert_eq!(cfg.dir, ".sys/n1");

        let cfg: Option<Config> = cfg.into();
        let cfg = cfg.unwrap();

        assert_eq!(cfg.me, "n1:raft".into());
        assert_eq!(cfg.dir, ".sys/n1");

        assert_eq!(
            cfg.routes
                .get(&mc::Address::new("n1", "raft"))
                .unwrap()
                .to_string(),
            "127.0.0.1:10094"
        );

        assert_eq!(
            cfg.routes
                .get(&mc::Address::new("n2", "raft"))
                .unwrap()
                .to_string(),
            "127.0.0.1:10095"
        );

        assert_eq!(
            cfg.routes
                .get(&mc::Address::new("n3", "raft"))
                .unwrap()
                .to_string(),
            "127.0.0.1:10096"
        );
    }
}
