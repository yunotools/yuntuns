use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::cli::validate_name;
use crate::paths;

pub const METADATA_FLAG: &str = "--yuntuns-plugin-metadata";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub aliases: Vec<String>,
    pub executable: PathBuf,
}

impl Plugin {
    pub fn matches(&self, selector: &str) -> bool {
        self.name == selector || self.aliases.iter().any(|alias| alias == selector)
    }
}

#[derive(Debug, Default)]
pub struct Discovery {
    pub plugins: Vec<Plugin>,
    pub warnings: Vec<String>,
}

pub fn discover() -> Result<Discovery, String> {
    let managed_bin = paths::bin_dir()?;
    let mut directories = vec![(managed_bin, true)];
    if let Some(path) = env::var_os("PATH") {
        directories.extend(env::split_paths(&path).map(|directory| (directory, false)));
    }

    let mut candidates = Vec::new();
    let mut seen_paths = HashSet::new();

    for (directory, is_managed) in directories {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };

        let mut directory_candidates = entries
            .flatten()
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        directory_candidates.sort();

        for path in directory_candidates {
            if plugin_name_from_path(&path).is_none() || !path.is_file() {
                continue;
            }

            let identity = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            if seen_paths.insert(identity) {
                candidates.push((path, is_managed));
            }
        }
    }

    let mut discovery = Discovery::default();
    let mut seen_names = HashSet::new();

    for (path, is_managed) in candidates {
        match load(&path) {
            Ok(plugin) if seen_names.insert(plugin.name.clone()) => {
                discovery.plugins.push(plugin);
            }
            Ok(_) => {}
            Err(error) if is_managed => discovery.warnings.push(error),
            Err(_) => {}
        }
    }

    discovery
        .plugins
        .sort_by(|left, right| left.name.cmp(&right.name));
    Ok(discovery)
}

pub fn load(path: &Path) -> Result<Plugin, String> {
    let expected_name = plugin_name_from_path(path)
        .ok_or_else(|| format!("{} không có dạng tên yuntuns-<plugin>", path.display()))?;
    let plugin = inspect(path)?;

    if plugin.name != expected_name {
        return Err(format!(
            "plugin {} báo tên `{}`, nhưng tên mong đợi là `{expected_name}`",
            path.display(),
            plugin.name
        ));
    }

    Ok(plugin)
}

pub fn inspect(path: &Path) -> Result<Plugin, String> {
    let output = Command::new(path)
        .arg(METADATA_FLAG)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("không thể kiểm tra plugin {}: {error}", path.display()))?;

    if !output.status.success() {
        return Err(format!(
            "plugin {} đã từ chối yêu cầu siêu dữ liệu với trạng thái {}",
            path.display(),
            output.status
        ));
    }

    let metadata = String::from_utf8(output.stdout).map_err(|_| {
        format!(
            "plugin {} trả về siêu dữ liệu không phải UTF-8",
            path.display()
        )
    })?;
    parse_metadata(path, &metadata)
}

pub fn execute(plugin: &Plugin, args: &[String]) -> Result<i32, String> {
    let status = Command::new(&plugin.executable)
        .args(args)
        .env("YUNTUNS_INVOKED", "1")
        .env("YUNTUNS_HOST_VERSION", env!("CARGO_PKG_VERSION"))
        .status()
        .map_err(|error| format!("không thể thực thi plugin `{}`: {error}", plugin.name))?;

    Ok(status.code().unwrap_or(1))
}

fn parse_metadata(path: &Path, metadata: &str) -> Result<Plugin, String> {
    let mut protocol = None;
    let mut name = None;
    let mut description = None;
    let mut version = None;
    let mut aliases = Vec::new();

    for line in metadata
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();

        match key.trim() {
            "protocol" => protocol = Some(value.to_owned()),
            "name" => name = Some(value.to_owned()),
            "description" => description = Some(value.to_owned()),
            "version" if !value.is_empty() => version = Some(value.to_owned()),
            "aliases" => {
                aliases = value
                    .split(',')
                    .map(str::trim)
                    .filter(|alias| !alias.is_empty())
                    .map(str::to_owned)
                    .collect();
            }
            _ => {}
        }
    }

    if protocol.as_deref() != Some("1") {
        return Err(format!(
            "plugin {} không hỗ trợ giao thức siêu dữ liệu 1",
            path.display()
        ));
    }

    let name = name.ok_or_else(|| format!("plugin {} không có tên", path.display()))?;
    validate_name(&name).map_err(|error| format!("plugin {}: {error}", path.display()))?;
    let description = description
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("plugin {} không có mô tả", path.display()))?;

    Ok(Plugin {
        name,
        description,
        version,
        aliases,
        executable: path.to_owned(),
    })
}

fn plugin_name_from_path(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;

    #[cfg(windows)]
    let binary_name = file_name
        .strip_suffix(".exe")
        .or_else(|| file_name.strip_suffix(".EXE"))?;

    #[cfg(not(windows))]
    let binary_name = file_name;

    let name = binary_name.strip_prefix("yuntuns-")?;
    (!name.is_empty()).then(|| name.to_owned())
}
