use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub(crate) struct TestEnv {
    root: TempDir,
    config_root: PathBuf,
    home: PathBuf,
}

impl TestEnv {
    pub(crate) fn new() -> Self {
        let root = tempfile::tempdir().expect("failed to create temp dir");
        let home = root.path().join("home");
        let cache_dir = home.join(".cache");
        let config_root = home.join(".config");

        fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
        fs::create_dir_all(&config_root).expect("failed to create config dir");
        fs::create_dir_all(&home).expect("failed to create home dir");

        Self {
            root,
            config_root,
            home,
        }
    }

    pub(crate) fn root(&self) -> &Path {
        self.root.path()
    }

    pub(crate) fn cache_dir(&self) -> PathBuf {
        self.home.join(".cache")
    }

    pub(crate) fn global_config_dir(&self) -> PathBuf {
        self.config_root.join("skillscripts")
    }

    pub(crate) fn global_config_file(&self) -> PathBuf {
        self.global_config_dir().join("skillscripts.yaml")
    }

    pub(crate) fn command_envs(&self) -> Vec<(&'static str, &Path)> {
        vec![("HOME", &self.home), ("USERPROFILE", &self.home)]
    }

    pub(crate) fn command(&self, workspace: &Path) -> Command {
        let mut cmd = Command::cargo_bin("skillscripts").expect("binary should build");
        let _ = self.cache_dir();
        let _ = self.global_config_dir();
        let _ = self.global_config_file();
        cmd.current_dir(workspace);
        for (key, value) in self.command_envs() {
            cmd.env(key, value);
        }
        cmd.env("LANG", "en_US.UTF-8");
        cmd.env("LC_ALL", "en_US.UTF-8");
        cmd
    }
}
