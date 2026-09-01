//! home 业务逻辑:登记/新建/克隆/路径查重。DESIGN.md:home 只登记路径不搬家。

use std::fs;
use std::path::{Path, PathBuf};

use crate::launcher::expand_tilde;
use crate::registry::{sanitize_id_fragment, HomeEntry, RegResult, Registry};

/// 新建 home 的默认根目录:~/.dsh-launcher/homes
pub fn default_homes_root() -> PathBuf {
    crate::registry::launcher_base_dir().join("homes")
}

/// 展开并校验「登记既有 home」的路径:必须存在且为目录。
pub fn validate_existing_home_path(path: &str) -> RegResult<PathBuf> {
    let dir = expand_tilde(path);
    if !dir.is_dir() {
        return Err(format!("home 目录不存在: {}", dir.display()));
    }
    Ok(dir)
}

/// 同一路径(规范化后)不得登记为两个 home;exclude_id 用于更新场景。
pub fn ensure_path_free(reg: &Registry, path: &Path, exclude_id: Option<&str>) -> RegResult<()> {
    let canon = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    for h in &reg.homes {
        if Some(h.id.as_str()) == exclude_id {
            continue;
        }
        let other = fs::canonicalize(Path::new(&h.path)).unwrap_or_else(|_| PathBuf::from(&h.path));
        if canon == other {
            return Err(format!("路径已登记为 home {:?}: {}", h.id, h.path));
        }
    }
    Ok(())
}

/// 构造并登记一个 home;path 必须已通过校验。id 为空时按目录名推导。
pub fn register_home(
    reg: &mut Registry,
    id: Option<String>,
    path: PathBuf,
    bound_version_id: Option<String>,
) -> RegResult<HomeEntry> {
    ensure_path_free(reg, &path, None)?;
    let base = match id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(x) => x.to_string(),
        None => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "home".into());
            sanitize_id_fragment(&name)
        }
    };
    let entry = HomeEntry {
        id: reg.fresh_home_id(&base),
        path: path.to_string_lossy().into_owned(),
        bound_version_id,
        last_good_version_id: None,
    };
    reg.upsert_home(entry.clone())?;
    Ok(entry)
}

/// 递归拷贝目录(克隆 home)。dst 必须不存在。注意:源 home 正在运行时克隆可能得到
/// 不一致的 SQLite,该约束由 M3 的运行锁在 UI 层拦截。
pub fn clone_dir(src: &Path, dst: &Path) -> RegResult<()> {
    if dst.exists() {
        return Err(format!("目标已存在: {}", dst.display()));
    }
    fs::create_dir_all(dst).map_err(|e| format!("创建 {} 失败: {e}", dst.display()))?;
    for entry in fs::read_dir(src).map_err(|e| format!("读取 {} 失败: {e}", src.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = dst.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            clone_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)
                .map_err(|e| format!("复制 {} 失败: {e}", entry.path().display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("dsh-launcher-homes-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn validate_existing_home_path_checks_dir() {
        assert!(validate_existing_home_path("/nonexistent-home-xyz").is_err());
        assert!(validate_existing_home_path("/tmp").is_ok());
    }

    #[test]
    fn ensure_path_free_blocks_duplicates() {
        let root = temp_root("dup");
        let dir = root.join("h1");
        fs::create_dir_all(&dir).unwrap();
        let mut reg = Registry::default();
        register_home(&mut reg, Some("a".into()), dir.clone(), None).unwrap();
        let dup = register_home(&mut reg, Some("b".into()), dir.clone(), None);
        assert!(dup.is_err(), "同路径二次登记应被拒绝");
        // 文件不存在时按字面路径比较,同样查重
        assert!(
            ensure_path_free(&reg, &dir, Some("a")).is_ok(),
            "排除自身应放行"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn register_home_derives_fresh_ids() {
        let mut reg = Registry::default();
        let e = register_home(&mut reg, None, PathBuf::from("/tmp/my-home"), None).unwrap();
        assert_eq!(e.id, "my-home");
        let e2 = register_home(&mut reg, None, PathBuf::from("/tmp/my-home2"), None).unwrap();
        assert_ne!(e2.id, e.id, "同名字段应靠 fresh_id 避让");
    }

    #[test]
    fn clone_dir_copies_tree_and_refuses_existing() {
        let root = temp_root("clone");
        let src = root.join("src");
        fs::create_dir_all(src.join("profiles/default")).unwrap();
        fs::write(src.join("cordis.yml"), "root").unwrap();
        fs::write(src.join("profiles/default/package.json"), "{}").unwrap();
        let dst = root.join("dst");
        clone_dir(&src, &dst).unwrap();
        assert!(dst.join("cordis.yml").is_file());
        assert!(dst.join("profiles/default/package.json").is_file());
        assert!(clone_dir(&src, &dst).is_err(), "目标已存在应拒绝");
        let _ = fs::remove_dir_all(&root);
    }
}
