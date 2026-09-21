# Yuntuns

Yuntuns là một ứng dụng chủ CLI nhỏ gọn. Ứng dụng không tích hợp sẵn các mô-đun
chức năng; các lệnh được cung cấp bởi những plugin thực thi độc lập.

## Tại sao dùng plugin dạng tệp thực thi?

- Việc cài đặt Yuntuns không kéo theo các phần phụ thuộc của mọi công cụ.
- Có thể cài đặt, nâng cấp và phát hành từng plugin một cách độc lập.
- Không cần biên dịch lại Yuntuns khi thêm plugin.
- Có thể viết plugin bằng bất kỳ ngôn ngữ nào, miễn là plugin tuân thủ đặc tả
  siêu dữ liệu ngắn gọn trong [`docs/PLUGIN_SPEC.md`](docs/PLUGIN_SPEC.md).

## Biên dịch

```bash
cargo build --release
```

Gói `yuntuns` không có phần phụ thuộc bên thứ ba nào khi chạy.

## Quản lý plugin

Cài đặt một plugin đã được phát hành trên Cargo:

```bash
yuntuns plugin install copyast
```

Cài đặt plugin đã được biên dịch sẵn mà không cần bộ công cụ Rust:

```bash
yuntuns plugin install copyast --binary ./copyast
```

Trên Windows, hãy dùng `copyast.exe`. Tệp thực thi sẽ được xác thực theo giao
thức plugin trước khi được sao chép vào thư mục do Yuntuns quản lý.

Cài đặt trong quá trình phát triển trên máy cục bộ:

```bash
yuntuns plugin install copyast --path ../copyast
```

Cài đặt từ Git:

```bash
yuntuns plugin install copyast --git https://github.com/yunotools/copyast
```

Xem danh sách, đường dẫn và gỡ cài đặt plugin:

```bash
yuntuns plugin list
yuntuns plugin path
yuntuns plugin uninstall copyast
```

Các plugin do Yuntuns cài đặt nằm trong một thư mục gốc Cargo biệt lập, bên
trong thư mục dữ liệu của Yuntuns. Đặt biến `YUNTUNS_HOME` để thay đổi vị trí đó.

## Chạy plugin

```bash
yuntuns copyast . ./copyast-output.txt
yuntuns copyast --help
```

Yuntuns hỗ trợ bí danh do plugin định nghĩa, vì vậy Copyast vẫn tương thích với:

```bash
yuntuns -c . ./copyast-output.txt
```
