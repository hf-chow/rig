use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct State {
    pub instance_id: u64,
    pub created_at: u64,
    pub ip: Option<String>,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
}

impl State {
    pub fn load() -> Result<Option<Self>> {
        let path = state_path();

        Self::load_from(&path)
    }

    pub fn save(&self) -> Result<()> {
        let path = state_path();

        self.save_to(&path)
    }

    pub fn clear() -> Result<()> {
        let path = state_path();

        Self::clear_at(&path)
    }

    fn load_from(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read state at {}", path.display()))?;

        let state = serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse state at {}", path.display()))?;

        Ok(Some(state))
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create config dir at {}", parent.display()))?;
        }

        let contents = serde_json::to_string_pretty(&self)?;
        std::fs::write(&path, contents)
            .with_context(|| format!("failed to write state at {}", &path.display()))?;

        Ok(())
    }

    fn clear_at(path: &Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

pub fn state_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rig")
        .join("state.json")
}

#[cfg(test)]
mod tests {
    use super::State;
    use std::path::Path;

    #[test]
    fn load_returns_none_when_no_file() {
        let fake_path = Path::new("/tmp/rig_nonexistent.json");
        assert!(State::load_from(&fake_path).unwrap().is_none())
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp_path = Path::new("/tmp/rig_test_roundtrip.json");

        let test_state = State {
            instance_id: 1,
            created_at: 2,
            ip: Some("1.2.3.4".to_string()),
            ssh_host: Some("host".to_string()),
            ssh_port: Some(22),
        };

        test_state.save_to(tmp_path).unwrap();
        let result = State::load_from(tmp_path).unwrap();

        assert!(result.is_some());

        let state = result.unwrap();
        assert_eq!(state.instance_id, 1);
        assert_eq!(state.created_at, 2);
        assert_eq!(state.ip, Some("1.2.3.4".to_string()));
        assert_eq!(state.ssh_host, Some("host".to_string()));
        assert_eq!(state.ssh_port, Some(22));

        State::clear_at(tmp_path).unwrap();
    }
}
