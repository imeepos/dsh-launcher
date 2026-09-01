use super::registry_io::validate_id;
use super::registry_model::{RegResult, Registry, VersionEntry};

impl Registry {
    pub fn find_version(&self, id: &str) -> Option<&VersionEntry> {
        self.versions.iter().find(|v| v.id == id)
    }

    pub fn find_version_mut(&mut self, id: &str) -> Option<&mut VersionEntry> {
        self.versions.iter_mut().find(|v| v.id == id)
    }

    /// 同 id 视为整体替换(指纹写回场景);新 id 走合法性校验。
    pub fn upsert_version(&mut self, entry: VersionEntry) -> RegResult<()> {
        if let Some(slot) = self.find_version_mut(&entry.id) {
            *slot = entry;
            return Ok(());
        }
        validate_id(&entry.id)?;
        self.versions.push(entry);
        Ok(())
    }

    pub fn remove_version(&mut self, id: &str) -> bool {
        let before = self.versions.len();
        self.versions.retain(|v| v.id != id);
        self.versions.len() != before
    }

    /// 基于 base 生成未占用的 id:base、base-2、base-3…
    pub fn fresh_id(&self, base: &str) -> String {
        if self.find_version(base).is_none() {
            return base.to_string();
        }
        for n in 2.. {
            let candidate = format!("{base}-{n}");
            if self.find_version(&candidate).is_none() {
                return candidate;
            }
        }
        unreachable!()
    }
}
