use crate::cli::{self, Action, PluginAction};
use crate::plugin::{self, Discovery, Plugin};
use crate::{APP_NAME, manager, output, paths};

pub fn run(args: &[String]) -> Result<i32, String> {
    match cli::parse(args)? {
        Action::Help => with_plugins(output::help),
        Action::Version => {
            println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
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

fn dispatch(selector: &str, args: &[String]) -> Result<i32, String> {
    let discovery = plugin::discover()?;
    let selected = select_plugin(&discovery.plugins, selector).ok_or_else(|| {
        format!("plugin `{selector}` chưa được cài đặt; hãy chạy `{APP_NAME} plugin list`")
    })?;

    plugin::execute(selected, args)
}

fn select_plugin<'a>(plugins: &'a [Plugin], selector: &str) -> Option<&'a Plugin> {
    plugins
        .iter()
        .find(|plugin| plugin.name == selector)
        .or_else(|| plugins.iter().find(|plugin| plugin.matches(selector)))
}

fn search<'a>(plugins: &'a [Plugin], query: Option<&str>) -> impl Iterator<Item = &'a Plugin> {
    let query = query.unwrap_or_default().trim().to_ascii_lowercase();

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

fn with_plugins(render: impl FnOnce(&[Plugin])) -> Result<i32, String> {
    with_discovery(|discovery| render(&discovery.plugins))
}

fn with_discovery(render: impl FnOnce(&Discovery)) -> Result<i32, String> {
    let discovery = plugin::discover()?;
    render(&discovery);
    output::warnings(&discovery.warnings);
    Ok(0)
}
