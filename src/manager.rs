use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::{InstallOptions, InstallSource};
use crate::{paths, plugin};

pub fn install(options: &InstallOptions) -> Result<(), String> {
    let root = paths::home_dir()?;
    fs::create_dir_all(&root).map_err(|error| {
        format!(
            "không thể tạo thư mục gốc của plugin {}: {error}",
            root.display()
        )
    })?;

    if let InstallSource::Binary(source) = &options.source {
        if options.version.is_some() {
            return Err("không thể dùng --version cùng với --binary".to_owned());
        }
        return install_binary(options, source);
    }

    let mut command = cargo_command();
    command.arg("install");

    match &options.source {
        InstallSource::Registry => {
            command.arg(&options.name);
        }
        InstallSource::Git(url) => {
            command.args(["--git", url, &options.name]);
        }
        InstallSource::Path(path) => {
            command.arg("--path").arg(path);
        }
        InstallSource::Binary(_) => unreachable!("cài đặt từ tệp thực thi không gọi Cargo"),
    }

    command.arg("--root").arg(&root);
    if let Some(version) = &options.version {
        command.args(["--version", version]);
    }
    if options.force {
        command.arg("--force");
    }

    let status = command
        .status()
        .map_err(|error| format!("không thể khởi chạy Cargo: {error}"))?;
    if !status.success() {
        return Err(format!("Cargo không thể cài đặt `{}`", options.name));
    }

    let executable = prepare_cargo_executable(&options.name)?;
    let installed = plugin::load(&executable).map_err(|error| {
        format!("gói đã được cài đặt nhưng không phải là plugin Yuntuns hợp lệ: {error}")
    })?;
    write_source_record(&options.name, "cargo")?;
    print_installed(&installed, &executable);
    Ok(())
}

pub fn uninstall(name: &str) -> Result<(), String> {
    if fs::read_to_string(source_record(name)?).is_ok_and(|source| source.trim() == "binary") {
        let executable = managed_executable(name)?;
        fs::remove_file(&executable)
            .map_err(|error| format!("không thể xóa {}: {error}", executable.display()))?;
        remove_source_record(name)?;
        println!("Đã gỡ cài đặt plugin `{name}`");
        return Ok(());
    }

    let root = paths::home_dir()?;
    let status = cargo_command()
        .args(["uninstall", "--root"])
        .arg(&root)
        .arg(name)
        .status()
        .map_err(|error| format!("không thể khởi chạy Cargo: {error}"))?;

    if !status.success() {
        return Err(format!("Cargo không thể gỡ cài đặt `{name}`"));
    }

    remove_file_if_exists(&managed_executable_path(name)?)?;
    remove_source_record(name)?;
    println!("Đã gỡ cài đặt plugin `{name}`");
    Ok(())
}

fn install_binary(options: &InstallOptions, source: &Path) -> Result<(), String> {
    if !source.is_file() {
        return Err(format!(
            "tệp thực thi của plugin không tồn tại: {}",
            source.display()
        ));
    }

    let source_plugin = plugin::inspect(source)?;
    if source_plugin.name != options.name {
        return Err(format!(
            "tệp thực thi của plugin báo tên `{}`, nhưng tên mong đợi là `{}`",
            source_plugin.name, options.name
        ));
    }

    let executable = managed_executable_path(&options.name)?;
    if executable.exists() && !options.force {
        return Err(format!(
            "plugin `{}` đã được cài đặt; hãy dùng --force để thay thế",
            options.name
        ));
    }

    let bin_dir = paths::bin_dir()?;
    fs::create_dir_all(&bin_dir).map_err(|error| {
        format!(
            "không thể tạo thư mục plugin {}: {error}",
            bin_dir.display()
        )
    })?;
    fs::copy(source, &executable).map_err(|error| {
        format!(
            "không thể sao chép plugin từ {} sang {}: {error}",
            source.display(),
            executable.display()
        )
    })?;

    let installed = plugin::load(&executable).map_err(|error| {
        format!("tệp thực thi đã sao chép không vượt qua bước xác thực plugin: {error}")
    })?;
    write_source_record(&options.name, "binary")?;
    print_installed(&installed, &executable);
    Ok(())
}

fn cargo_command() -> Command {
    std::env::var_os("CARGO")
        .map(Command::new)
        .unwrap_or_else(|| Command::new("cargo"))
}

fn managed_executable(name: &str) -> Result<PathBuf, String> {
    let executable = managed_executable_path(name)?;
    executable.exists().then_some(executable).ok_or_else(|| {
        format!("plugin `{name}` chưa được cài đặt trong thư mục plugin được quản lý")
    })
}

fn prepare_cargo_executable(name: &str) -> Result<PathBuf, String> {
    let plugin_executable = managed_executable_path(name)?;
    let standalone_executable = standalone_executable_path(name)?;

    if standalone_executable.is_file() {
        let standalone = plugin::inspect(&standalone_executable)?;
        if standalone.name != name {
            return Err(format!(
                "tệp thực thi đã cài đặt báo tên `{}`, nhưng tên mong đợi là `{name}`",
                standalone.name
            ));
        }

        remove_file_if_exists(&plugin_executable)?;
        if fs::hard_link(&standalone_executable, &plugin_executable).is_err() {
            fs::copy(&standalone_executable, &plugin_executable).map_err(|error| {
                format!(
                    "không thể tạo điểm khởi chạy plugin {}: {error}",
                    plugin_executable.display()
                )
            })?;
        }
    }

    plugin_executable
        .is_file()
        .then_some(plugin_executable)
        .ok_or_else(|| {
            format!("gói đã cài đặt không cung cấp tệp thực thi `{name}` hoặc `yuntuns-{name}`")
        })
}

fn managed_executable_path(name: &str) -> Result<PathBuf, String> {
    let file_name = if cfg!(windows) {
        format!("yuntuns-{name}.exe")
    } else {
        format!("yuntuns-{name}")
    };
    Ok(paths::bin_dir()?.join(file_name))
}

fn standalone_executable_path(name: &str) -> Result<PathBuf, String> {
    let file_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    Ok(paths::bin_dir()?.join(file_name))
}

fn source_record(name: &str) -> Result<PathBuf, String> {
    Ok(paths::home_dir()?.join(format!(".yuntuns-{name}.source")))
}

fn write_source_record(name: &str, source: &str) -> Result<(), String> {
    let path = source_record(name)?;
    fs::write(&path, source).map_err(|error| {
        format!(
            "không thể ghi thông tin cài đặt vào {}: {error}",
            path.display()
        )
    })
}

fn remove_source_record(name: &str) -> Result<(), String> {
    let path = source_record(name)?;
    remove_file_if_exists(&path)
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("không thể xóa {}: {error}", path.display())),
    }
}

fn print_installed(installed: &plugin::Plugin, executable: &Path) {
    println!(
        "Đã cài đặt plugin `{}`{} tại {}",
        installed.name,
        installed
            .version
            .as_deref()
            .map(|version| format!(" v{version}"))
            .unwrap_or_default(),
        executable.display()
    );
}
