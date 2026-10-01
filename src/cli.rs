use std::path::PathBuf;

// Action đại diện cho lệnh cấp cao nhất mà user muốn thực hiện
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Help,
    Version,
    Search(Option<String>),
    Plugin(PluginAction),
    // Ví dụ: yuntuns copyast . out.txt --dry-run
    // sẽ thành:
    // Action::Dispatch {
    //     selector: "copyast".to_owned(),
    //     args: vec![
    //         ".".to_owned(),
    //         "out.txt".to_owned(),
    //         "--dry-run".to_owned(),
    //     ],
    // }
    // Ý nghĩa:
    // selector = plugin muốn chạy
    // args     = argument chuyển cho plugin
    // yuntuns copyast . out.txt --dry-run
    //         │       └─────────────── args
    //         │
    //         └── selector
    Dispatch { selector: String, args: Vec<String> },
}

// Đây là command cấp dưới của: yuntuns plugin ...
#[derive(Debug, PartialEq, Eq)]
pub enum PluginAction {
    Help,
    List,
    Path,
    Install(InstallOptions),
    Uninstall { name: String },
}


// Ví dụ:
// yuntuns plugin install copyast \
//     --git https://example.com/copyast \
//     --version 1.2.0 \
//     --force
// sẽ gần thành
// InstallOptions {
//     name: "copyast".to_owned(),
//
//     source: InstallSource::Git(
//         "https://example.com/copyast".to_owned()
//     ),
//
//     version: Some("1.2.0".to_owned()),
//
//     force: true,
// }
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
    // User chỉ gõ yuntuns → hiện help.
    // ~~ let first = match args.first() {
    //     Some(first) => first,
    //     None => return Ok(Action::Help),
    // };
    let Some(first) = args.first() else {
        return Ok(Action::Help);
    };

    match first.as_str() {
        "-h" | "--help" => require_no_extra(args, Action::Help),
        "-v" | "--version" => require_no_extra(args, Action::Version),
        "-s" | "--search" => parse_search(&args[1..]),
        "-p" | "p" | "plugin" | "plugins" => parse_plugin_action(&args[1..]),

        // Pattern guard
        value if value.starts_with("--search=") => {
            if args.len() != 1 {
                return Err("--search chỉ chấp nhận tối đa một từ khóa tìm kiếm".to_owned());
            }
            let query = value.trim_start_matches("--search=").trim();
            Ok(Action::Search(
                // bool::then() hoạt động như:
                // true
                // → Some(...)
                //
                // false
                // → None
                (!query.is_empty()).then(|| query.to_owned()),
            ))
        }
        selector => Ok(Action::Dispatch {
            selector: selector.to_owned(),
            args: args[1..].to_vec(),
        }),
    }
}

// Dùng cho command không được nhận argument thêm.
fn require_no_extra(args: &[String], action: Action) -> Result<Action, String> {
    if args.len() == 1 {
        Ok(action)
    } else {
        Err(format!("đối số không mong đợi `{}`", args[1]))
    }
}

// Parser riêng cho search
fn parse_search(args: &[String]) -> Result<Action, String> {
    match args {
        [] => Ok(Action::Search(None)),
        [query] if !query.starts_with('-') => Ok(Action::Search(Some(query.clone()))),
        [query] => Err(format!("từ khóa tìm kiếm không hợp lệ `{query}`")),
        _ => Err("--search chỉ chấp nhận tối đa một từ khóa tìm kiếm".to_owned()),
    }
}

// Parser cho plugin action
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

    // [
    //     "copyast",       // 0
    //     "--path",        // 1
    //     "../copyast",    // 2
    //     "--force"        // 3
    // ]
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

// args
//  ├── "--git"
//  └── "https://..."
//         ↑
//         │
// return &str
// Reference trả về không được sống lâu hơn args.
fn required_value<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, String> {
    args.get(index + 1)
        .filter(|value| !value.starts_with('-'))
        .map(String::as_str)
        // Ví dụ: --git --force
        // sẽ bị xem là thiếu value cho --git
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
