use std::env;
use std::fmt::Debug;
use std::fs::{create_dir_all, File};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::str::FromStr;

use tempfile::TempDir;
use url::Url;

use crate::MadaraArgs;

const MIN_PORT: u16 = 49_152;
const MAX_PORT: u16 = 65_535;

#[derive(Debug)]
/// A helper struct for creating temporary Madara folders
///
/// The temporary directory is deleted when instance is dropped.
pub struct MadaraTempDir {
    temp_dir: TempDir,
}

impl MadaraTempDir {
    pub fn base_path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    pub fn data_path(&self) -> PathBuf {
        self.temp_dir.path().join("chains/dev")
    }

    pub fn clear(self) {
        self.temp_dir.close().expect("Failed to clean up");
    }
}

impl Default for MadaraTempDir {
    fn default() -> Self {
        let temp_dir = TempDir::with_prefix("madara").expect("Failed to create Madara path");
        let data_path = temp_dir.path().join("chains/dev");
        create_dir_all(data_path).expect("Failed to create data path");
        Self { temp_dir }
    }
}

#[derive(Debug)]
/// A wrapper over the Madara process handle, reqwest client and request counter
///
/// When this struct goes out of scope, it's `Drop` impl
/// will take care of killing the Madara process.
pub struct MadaraNode {
    process: Child,
    port: u16,
}

impl Drop for MadaraNode {
    fn drop(&mut self) {
        if let Err(e) = self.process.kill() {
            eprintln!("Could not kill Madara process: {}", e)
        }
    }
}

fn get_free_port() -> u16 {
    for port in MIN_PORT..=MAX_PORT {
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
            return listener.local_addr().expect("No local addr").port();
        }
        // otherwise port is occupied
    }
    panic!("No free ports available");
}

fn get_repository_root() -> PathBuf {
    let manifest_path = Path::new(&env!("CARGO_MANIFEST_DIR"));
    let repository_root = manifest_path.parent().expect("Failed to get parent directory of CARGO_MANIFEST_DIR");
    repository_root.to_path_buf()
}

impl MadaraNode {
    /// Run the Madara node
    ///
    /// The node is run in `release` mode.
    /// Parameters to the node can be passed with the `params` argument.
    fn cargo_run(root_dir: &Path, params: Vec<&str>) -> Child {
        let arguments = [vec!["run", "--release", "--"], params].concat();

        let (stdout, stderr) = if env::var("MADARA_LOG").is_ok() {
            let logs_dir = Path::join(root_dir, Path::new("target/madara-log"));
            create_dir_all(logs_dir.clone()).expect("Failed to create logs dir");
            (
                Stdio::from(File::create(Path::join(&logs_dir, Path::new("madara-stdout-log.txt"))).unwrap()),
                Stdio::from(File::create(Path::join(&logs_dir, Path::new("madara-stderr-log.txt"))).unwrap()),
            )
        } else {
            (Stdio::null(), Stdio::null())
        };

        Command::new("cargo").stdout(stdout).stderr(stderr).args(arguments).spawn().expect("Could not run Madara node")
    }

    pub fn run(args: MadaraArgs) -> Self {
        let port = get_free_port();
        let repository_root = &get_repository_root();

        std::env::set_current_dir(repository_root).expect("Failed to change working directory");

        let base_path_arg = args.base_path.map(|arg| format!("--base-path={}", arg.display()));
        let settlement_arg = args.settlement.map(|arg| format!("--settlement={arg}"));
        let settlement_conf_arg = args.settlement_conf.map(|arg| format!("--settlement-conf={}", arg.display()));
        let rpc_port_arg = format!("--rpc-port={port}");
        let chain_arg = "--chain=dev";
        let from_local_arg = format!("--from-local={}", repository_root.join("configs").display());

        // Codeblock to drop `setup_args` and be able to borrow again for `run_args`
        {
            let mut setup_args = vec!["setup", &chain_arg, &from_local_arg];
            if let Some(bp) = &base_path_arg {
                setup_args.push(bp);
            };

            let setup_res =
                Self::cargo_run(repository_root.as_path(), setup_args).wait().expect("Failed to setup Madara node");

            if !setup_res.success() {
                panic!("Madara setup failed with {} (check out stderr logs)", setup_res);
            }
        }

        let mut run_args = vec!["--alice", "--sealing=manual", &chain_arg, &rpc_port_arg];
        if let Some(bp) = &base_path_arg {
            run_args.push(bp);
        };
        if let Some(s) = &settlement_arg {
            run_args.push(s);
        };
        if let Some(s) = &settlement_conf_arg {
            run_args.push(s);
        }

        let process = Self::cargo_run(repository_root.as_path(), run_args);

        Self { process, port }
    }

    pub fn url(&self) -> Url {
        Url::from_str(&format!("http://127.0.0.1:{}", self.port)).unwrap()
    }

    pub fn has_exited(&mut self) -> Option<ExitStatus> {
        self.process.try_wait().expect("Failed to get Madara node exit status")
    }
}
