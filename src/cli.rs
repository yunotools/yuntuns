use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Help,
    Version,
    Search(Option<String>),
    Plugin(PluginAction),
    Dispatch { selector: String, args: Vec<String> },
}

#[derive(Debug, PartialEq, Eq)]
pub enum PluginAction {
    Help,
    List,
    Path,
    Install(InstallOptions),
    Uninstall { name: String },
}

#[derive(Debug, PartialEq, Eq)]
pub struct InstallOptions {
    pub name: String,
    pub source: InstallSource,
    pub version: Option<String>,
    pub force: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InstallSource {
    Registry,
    Git(String),
    Path(PathBuf),
    Binary(PathBuf),
}

pub fn parse(args: &[String]) -> Result<Action, String> {
    let Some(first) = args.first() else {
        return Ok(Action::Help);
    };

    match first.as_str() {
        "-h" | "--help" => require_no_extra(args, Action::Help),
        "-v" | "--version" => require_no_extra(args, Action::Version),
        "-s" | "--search" => parse_search(&args[1..]),
        "plugin" | "plugins" => parse_plugin_action(&args[1..]),
        value if value.starts_with("--search=") => {
            if args.len() != 1 {
                return Err("--search chỉ chấp nhận tối đa một từ khóa tìm kiếm".to_owned());
            }
            let query = value.trim_start_matches("--search=").trim();
            Ok(Action::Search(
                (!query.is_empty()).then(|| query.to_owned()),
            ))
        }
        selector => Ok(Action::Dispatch {
            selector: selector.to_owned(),
            args: args[1..].to_vec(),
        }),
    }
}

fn require_no_extra(args: &[String], action: Action) -> Result<Action, String> {
    if args.len() == 1 {
        Ok(action)
    } else {
        Err(format!("đối số không mong đợi `{}`", args[1]))
    }
}

fn parse_search(args: &[String]) -> Result<Action, String> {
    match args {
        [] => Ok(Action::Search(None)),
        [query] if !query.starts_with('-') => Ok(Action::Search(Some(query.clone()))),
        [query] => Err(format!("từ khóa tìm kiếm không hợp lệ `{query}`")),
        _ => Err("--search chỉ chấp nhận tối đa một từ khóa tìm kiếm".to_owned()),
    }
}

fn parse_plugin_action(args: &[String]) -> Result<Action, String> {
    let action = match args.first().map(String::as_str) {
        None | Some("-h" | "--help") => PluginAction::Help,
        Some("list") if args.len() == 1 => PluginAction::List,
        Some("path") if args.len() == 1 => PluginAction::Path,
        Some("install") => PluginAction::Install(parse_install_options(&args[1..])?),
        Some("uninstall") => {
            let [_, name] = args else {
                return Err("cách dùng: yuntuns plugin uninstall <TÊN>".to_owned());
            };
            validate_name(name)?;
            PluginAction::Uninstall { name: name.clone() }
        }
        Some(command) => return Err(format!("lệnh quản lý plugin không xác định `{command}`")),
    };

    Ok(Action::Plugin(action))
}

fn parse_install_options(args: &[String]) -> Result<InstallOptions, String> {
    let Some(name) = args.first() else {
        return Err("cách dùng: yuntuns plugin install <TÊN> [TÙY_CHỌN]".to_owned());
    };
    validate_name(name)?;

    let mut source = InstallSource::Registry;
    let mut has_source = false;
    let mut version = None;
    let mut force = false;
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--git" => {
                if has_source {
                    return Err("chỉ được dùng một tùy chọn nguồn plugin".to_owned());
                }
                let value = required_value(args, index, "--git")?;
                source = InstallSource::Git(value.to_owned());
                has_source = true;
                index += 2;
            }
            "--path" => {
                if has_source {
                    return Err("chỉ được dùng một tùy chọn nguồn plugin".to_owned());
                }
                let value = required_value(args, index, "--path")?;
                source = InstallSource::Path(PathBuf::from(value));
                has_source = true;
                index += 2;
            }
            "--binary" => {
                if has_source {
                    return Err("chỉ được dùng một tùy chọn nguồn plugin".to_owned());
                }
                let value = required_value(args, index, "--binary")?;
                source = InstallSource::Binary(PathBuf::from(value));
                has_source = true;
                index += 2;
            }
            "--version" => {
                let value = required_value(args, index, "--version")?;
                version = Some(value.to_owned());
                index += 2;
            }
            "--force" => {
                force = true;
                index += 1;
            }
            option => return Err(format!("tùy chọn cài đặt không xác định `{option}`")),
        }
    }

    Ok(InstallOptions {
        name: name.clone(),
        source,
        version,
        force,
    })
}

fn required_value<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, String> {
    args.get(index + 1)
        .filter(|value| !value.starts_with('-'))
        .map(String::as_str)
        .ok_or_else(|| format!("{option} yêu cầu một giá trị"))
}

pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(format!(
            "tên plugin không hợp lệ `{name}`; chỉ dùng chữ cái, chữ số, '-' hoặc '_'"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn dispatch_keeps_plugin_arguments() {
        assert_eq!(
            parse(&args(&["copyast", ".", "out.txt", "--dry-run"])),
            Ok(Action::Dispatch {
                selector: "copyast".to_owned(),
                args: args(&[".", "out.txt", "--dry-run"]),
            })
        );
    }

    #[test]
    fn parses_local_install() {
        assert_eq!(
            parse(&args(&[
                "plugin",
                "install",
                "copyast",
                "--path",
                "../copyast",
                "--force",
            ])),
            Ok(Action::Plugin(PluginAction::Install(InstallOptions {
                name: "copyast".to_owned(),
                source: InstallSource::Path(PathBuf::from("../copyast")),
                version: None,
                force: true,
            })))
        );
    }

    #[test]
    fn rejects_multiple_install_sources() {
        let error = parse(&args(&[
            "plugin",
            "install",
            "copyast",
            "--path",
            ".",
            "--git",
            "https://example.test/copyast",
        ]))
        .unwrap_err();

        assert_eq!(error, "chỉ được dùng một tùy chọn nguồn plugin");
    }

    #[test]
    fn parses_prebuilt_binary_install() {
        assert_eq!(
            parse(&args(&[
                "plugin",
                "install",
                "copyast",
                "--binary",
                "./copyast",
            ])),
            Ok(Action::Plugin(PluginAction::Install(InstallOptions {
                name: "copyast".to_owned(),
                source: InstallSource::Binary(PathBuf::from("./copyast")),
                version: None,
                force: false,
            })))
        );
    }
}
