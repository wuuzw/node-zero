use curl::easy::Easy;
use flate2::read::GzDecoder;
use nix::sys::utsname;
use std::fs::{create_dir_all, remove_dir_all, remove_file, File};
use std::io::Write;
use std::os::unix;
use std::path::{Component::Normal, Path, PathBuf};
use tar::Archive;
use tempfile::TempDir;

pub struct Installer {
    base_url: String,
    system: String,
    machine: String,
    install_path: PathBuf,
}

impl Installer {
    pub fn new() -> Self {
        let base_url = "https://unofficial-builds.nodejs.org/download/release".to_string();
        let system = utsname::uname().sysname().to_lowercase();
        let machine = utsname::uname().machine().to_lowercase();
        let install_path = Path::new("/opt/nodejs").to_owned();

        Self {
            base_url,
            system,
            machine,
            install_path,
        }
    }

    pub fn print_sys_info(&self) {
        println!("Host: {}", utsname::uname().nodename());
        println!("System: {}", &self.system);
        println!("Machine: {}", &self.machine);
    }

    fn download(&self, version: &String, file: &str, dir: &Path) -> PathBuf {
        let file_path = dir.join(&file);
        let download_url = format!("{}/{}/{}", &self.base_url, version, &file);

        println!("Downloading {}...", file);

        let mut destination = File::create(&file_path).unwrap();

        let mut easy = Easy::new();
        easy.url(download_url.as_str()).unwrap();
        {
            let mut transfer = easy.transfer();
            transfer
                .write_function(|new_data| {
                    destination.write_all(new_data).unwrap();
                    Ok(new_data.len())
                })
                .unwrap();
            transfer.perform().unwrap();
        }

        assert_eq!(200, easy.response_code().unwrap());

        file_path
    }

    fn unpack(&self, src: PathBuf) {
        println!("Unpacking...");
        let tar = GzDecoder::new(File::open(src).unwrap());
        let mut archive = Archive::new(tar);
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let path: PathBuf = entry
                .path()
                .unwrap()
                .components()
                .skip(1)
                .filter(|c| matches!(c, Normal(_)))
                .collect();
            entry
                .unpack(Path::new(&self.install_path).join(path))
                .unwrap();
        }
    }

    fn make_symlinks(&self) {
        println!("Creating symlinks...");

        unix::fs::symlink(&self.install_path.join("bin/node"), "/usr/bin/node").unwrap();
        unix::fs::symlink(&self.install_path.join("bin/npm"), "/usr/bin/npm").unwrap();
        unix::fs::symlink(&self.install_path.join("bin/npx"), "/usr/bin/npx").unwrap();

        unix::fs::symlink(&self.install_path.join("bin/node"), "/usr/sbin/node").unwrap();
        unix::fs::symlink(&self.install_path.join("bin/npm"), "/usr/sbin/npm").unwrap();
        unix::fs::symlink(&self.install_path.join("bin/npx"), "/usr/sbin/npx").unwrap();

        unix::fs::symlink(&self.install_path.join("bin/node"), "/usr/local/bin/node").unwrap();
        unix::fs::symlink(&self.install_path.join("bin/npm"), "/usr/local/bin/npm").unwrap();
        unix::fs::symlink(&self.install_path.join("bin/npx"), "/usr/local/bin/npx").unwrap();
    }

    fn uninstall(&self) {
        println!("Removing old nodejs directory...");
        let symlinks = [
            "/usr/bin/node",
            "/usr/bin/npm",
            "/usr/bin/npx",
            "/usr/sbin/node",
            "/usr/sbin/npm",
            "/usr/sbin/npx",
            "/usr/local/bin/node",
            "/usr/local/bin/npm",
            "/usr/local/bin/npx",
        ];
        for symlink in symlinks.iter() {
            remove_file(symlink).unwrap();
        }

        remove_dir_all(&self.install_path).unwrap();
    }

    pub fn install(&self, version: String) {
        if self.install_path.exists() {
            self.uninstall();
        }

        create_dir_all(&self.install_path).unwrap();

        let file_name = format!("node-{}-{}-{}.tar.gz", &version, self.system, self.machine);
        let temp_dir = TempDir::new().unwrap();

        let node_file = self.download(&version, &file_name, &temp_dir.path());
        self.unpack(node_file);
        self.make_symlinks();
        println!("Done");
    }
}
