# Đặc tả plugin Yuntuns (giao thức 1)

Plugin là một tệp thực thi có tên `yuntuns-<name>` (`.exe` trên Windows). Yuntuns
tìm các tệp thực thi trong thư mục `bin` do ứng dụng quản lý và trong `PATH`.

Khi được gọi với `--yuntuns-plugin-metadata`, tệp thực thi phải in ra các dòng
`key=value` được mã hóa bằng UTF-8 và kết thúc thành công:

```text
protocol=1
name=copyast
version=0.1.0
description=Sao chép các tệp văn bản vào một tệp ngữ cảnh AI
aliases=-c,--copyast
```

Các khóa bắt buộc là `protocol`, `name` và `description`. Hai khóa `version` và
`aliases` là tùy chọn. Đối với các tệp thực thi được tìm thấy, trường `name`
trong siêu dữ liệu phải khớp với phần tên đứng sau tiền tố `yuntuns-` của tệp
thực thi. Tên chỉ được chứa chữ cái ASCII, chữ số, dấu `-` và dấu `_`.

Khi thực thi thông thường, Yuntuns loại bỏ lệnh hoặc bí danh đã chọn rồi chuyển
các đối số còn lại cho plugin. Plugin sẽ kế thừa stdin, stdout và stderr. Yuntuns
cũng thiết lập các biến môi trường sau:

- `YUNTUNS_INVOKED=1`
- `YUNTUNS_HOST_VERSION=<version>`

Để cài đặt bằng `yuntuns plugin install`, gói Cargo nên dùng tên plugin làm tên
gói. Gói có thể cung cấp tệp thực thi `<name>` để sử dụng độc lập như bình thường
hoặc cung cấp trực tiếp `yuntuns-<name>`; Yuntuns sẽ tạo điểm khởi chạy được quản
lý cho plugin khi cần. Có thể cài đặt tệp thực thi đã được biên dịch sẵn bằng
`--binary <TỆP>`; siêu dữ liệu mà tệp báo cáo phải khớp với tên plugin được yêu
cầu.
