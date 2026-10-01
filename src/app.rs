use crate::cli::{self, Action, PluginAction};
use crate::plugin::{self, Discovery, Plugin};
use crate::{APP_NAME, manager, output, paths};

//                    app.rs
//                      |
//         +------------+-------------+
//         |            |             |
//        cli         plugin        manager
//         |            |             |
//      parsing      discover       install
//                   execute        uninstall
//                      |
//                    output

pub fn run(args: &[String]) -> Result<i32, String> {
    match cli::parse(args)? {
        Action::Help => with_plugins(output::help),
        Action::Version => {
            println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        // Closure
        // ~~ fn something(discovery: &Discovery) {
        //     ...
        // }
        // Nếu query: Option<String> thì query.as_deref()
        // chuyển Option<String> -> Option<&str>
        Action::Search(query) => with_discovery(|discovery| {
            output::plugins(search(&discovery.plugins, query.as_deref()));
        }),
        Action::Plugin(action) => run_plugin_action(action),
        Action::Dispatch { selector, args } => dispatch(&selector, &args),
    }
}

fn run_plugin_action(action: PluginAction) -> Result<i32, String> {
    match action {
        PluginAction::Help => {
            output::plugin_help();
            Ok(0)
        }
        PluginAction::List => with_plugins(|plugins| output::plugins(plugins)),
        PluginAction::Path => {
            println!("{}", paths::bin_dir()?.display());
            Ok(0)
        }
        PluginAction::Install(options) => {
            manager::install(&options)?;
            Ok(0)
        }
        PluginAction::Uninstall { name } => {
            manager::uninstall(&name)?;
            Ok(0)
        }
    }
}

// Tìm 1 plugin mà user yêu cầu và chạy plugin đó
fn dispatch(selector: &str, args: &[String]) -> Result<i32, String> {
    // Scan rôi tạo
    // Discovery {
    //     plugins: ...,
    //     warnings: ...
    // }
    let discovery = plugin::discover()?;

    // Return Option<&Plugin>
    // Có thể là: Some(plugin)
    // Hoặc None
    // ~~ let selected = match select_plugin(&discovery.plugins, selector) {
    //     Some(plugin) => plugin,
    //
    //     None => {
    //         return Err(format!(
    //             "plugin `{selector}` chưa được cài đặt; hãy chạy `{APP_NAME} plugin list`"
    //         ));
    //     }
    // };
    let selected = select_plugin(&discovery.plugins, selector).ok_or_else(|| {
        format!("plugin `{selector}` chưa được cài đặt; hãy chạy `{APP_NAME} plugin list`")
    })?;

    // Tại sao ok_or_else, không phải ok_or
    // Ta cũng có .ok_or(error)
    // Nhưng: ok_or_else(|| ...) dùng closure.
    // - Điểm lợi: Chỉ tạo error string nếu thực sự là None.
    // Ví dụ: Some(plugin)
    // thì closure này: || format!(...) không được gọi.
    // --> lazy evaluation

    plugin::execute(selected, args)
}

// 'a là lifetime parameter.
// - Plugin reference trả về sống không lâu hơn slice plugins đầu vào.
// - Input: plugins: &'a [Plugin]
// - Output: Option<&'a Plugin>
// - Nghĩa là output reference trỏ vào chính data của plugins.
//
// Ví dụ:
// let plugins = vec![...];
// let selected = select_plugin(&plugins, "copyast");
// selected không chứa copy của plugin.
// Nó chỉ reference tới plugin trong: plugins
fn select_plugin<'a>(plugins: &'a [Plugin], selector: &str) -> Option<&'a Plugin> {
    plugins
        .iter()
        .find(|plugin| plugin.name == selector)
        .or_else(|| plugins.iter().find(|plugin| plugin.matches(selector)))
}

// Compiler biết concrete type thực sự rất dài kiểu: Filter<Iter<'a, Plugin>, Closure...>
// nhưng ta không cần viết
// --> Đây chính là lợi ích của: impl Trait
// --> Nếu không dùng impl Iterator, type có thể rất khó đọc.


// Nó search trên:
// - name
// - description
// - aliases
// Ví dụ plugin:
// Plugin {
//     name: "kubernetes",
//     description: "Manage Kubernetes clusters",
//     aliases: vec!["k8s", "kube"],
// }
//
// Query: k8s
// - Check:
// - query.is_empty()
// - false
//
// - name contains k8s
// - false
//
// - description contains k8s
// - false
//
// - aliases contains k8s
// - true
// --> plugin được giữ lại.
//
// Nếu query empty: query.is_empty() là true.
// Do || short-circuit: true || ... Rust không check các phần còn lại.
// --> Kết quả: query rỗng → tất cả plugin được trả về.
fn search<'a>(plugins: &'a [Plugin], query: Option<&str>) -> impl Iterator<Item = &'a Plugin> {
    let query = query.unwrap_or_default().trim().to_ascii_lowercase();

    // search()
    //  ├── query String
    //  │
    //  │ move
    //  ▼
    // Filter Iterator
    //  └── closure owns query
    plugins.iter().filter(move |plugin| {
        query.is_empty()
            || plugin.name.to_ascii_lowercase().contains(&query)
            || plugin.description.to_ascii_lowercase().contains(&query)
            || plugin
                .aliases
                .iter()
                .any(|alias| alias.to_ascii_lowercase().contains(&query))
    })
}

// impl FnOnce(&[Plugin]) render: impl FnOnce(&[Plugin])
// --> Nghĩa là argument render là: Bất kỳ function hoặc closure nào có thể được gọi một lần với &[Plugin].
// Ví dụ hợp lệ:
// - with_plugins(output::help);
// - with_plugins(|plugins| {
//     output::plugins(plugins);
//   });
// Vì sao FnOnce?
// Rust có 3 closure trait chính:
// - Fn
// - FnMut
// - FnOnce
// Quan hệ đại khái:
// Fn
// ↓
// FnMut
// ↓
// FnOnce
// - FnOnce là constraint ít nghiêm ngặt nhất: Tôi chỉ cần đảm bảo closure gọi được ít nhất một lần.
fn with_plugins(render: impl FnOnce(&[Plugin])) -> Result<i32, String> {
    with_discovery(|discovery| render(&discovery.plugins))
}

fn with_discovery(render: impl FnOnce(&Discovery)) -> Result<i32, String> {
    let discovery = plugin::discover()?;
    render(&discovery);
    output::warnings(&discovery.warnings);
    Ok(0)
}
