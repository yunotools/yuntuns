use crate::APP_NAME;
use crate::plugin::Plugin;

pub fn help(plugins: &[Plugin]) {
    println!(
        "{APP_NAME} - ứng dụng chủ plugin CLI gọn nhẹ\n\n\
         Cách dùng:\n  \
           {APP_NAME} <PLUGIN> [ĐỐI_SỐ...]\n  \
           {APP_NAME} plugin <LỆNH>\n  \
           {APP_NAME} --search [TỪ_KHÓA]\n\n\
         Tùy chọn chung:\n  \
           -h, --help                 Hiển thị trợ giúp này\n  \
           -v, --version              Hiển thị phiên bản ứng dụng\n  \
           -s, --search [TỪ_KHÓA]     Tìm kiếm plugin đã cài đặt\n\n\
         Quản lý plugin:\n  \
           plugin list                Liệt kê các plugin đã cài đặt\n  \
           plugin install ...         Cài đặt hoặc cập nhật plugin\n  \
           plugin uninstall TÊN       Gỡ cài đặt plugin\n  \
           plugin path                In đường dẫn thư mục plugin được quản lý"
    );

    println!("\nCác plugin đã cài đặt:");
    self::plugins(plugins);
}

pub fn plugin_help() {
    println!(
        "Quản lý các plugin Yuntuns\n\n\
         Cách dùng:\n  \
           {APP_NAME} plugin list\n  \
           {APP_NAME} plugin path\n  \
           {APP_NAME} plugin install <TÊN> [--binary TỆP | --path THƯ_MỤC | --git URL] [TÙY_CHỌN]\n  \
           {APP_NAME} plugin uninstall <TÊN>"
    );
}

pub fn plugins<'a>(plugins: impl IntoIterator<Item = &'a Plugin>) {
    let mut found = false;

    for plugin in plugins {
        found = true;
        let aliases = if plugin.aliases.is_empty() {
            String::new()
        } else {
            format!(" [{}]", plugin.aliases.join(", "))
        };
        let version = plugin
            .version
            .as_deref()
            .map(|value| format!(" v{value}"))
            .unwrap_or_default();

        println!(
            "  {:<14} {}{}{}",
            plugin.name, plugin.description, version, aliases
        );
    }

    if !found {
        println!("  (không có)");
    }
}

pub fn warnings(warnings: &[String]) {
    for warning in warnings {
        eprintln!("cảnh báo: {warning}");
    }
}
