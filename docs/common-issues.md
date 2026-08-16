# Các lỗi thường gặp khi gõ Tiếng Việt và Cách khắc phục

Tài liệu này tổng hợp chi tiết các lỗi phổ biến khi sử dụng bộ gõ tiếng Việt trên máy tính, bao gồm cả **bộ gõ mặc định của hệ thống** (macOS/Windows) và các **bộ gõ bên thứ ba** (Unikey, EVKey, OpenKey...).

---

## 1. Lỗi của Bộ gõ Mặc định (System Default)

### 1.1. macOS Default Input (Telex/VNI)

Đây là bộ gõ có sẵn khi bạn cài macOS.

- **Hộp đen che chữ (Pre-edit Window)**
  - **Hiện tượng**: Khi gõ trong Terminal, Telegram hoặc các app chưa hỗ trợ tốt InputMethodKit, một hộp đen nhỏ hiện ra chứa ký tự đang gõ, che mất nội dung bên dưới.
  - **Cách khắc phục**: Hiện tại chưa có cách tắt triệt để trên bộ gõ mặc định. Người dùng thường phải chuyển sang sử dụng bộ gõ bên thứ ba (như OpenKey, EVKey) có tính năng "Edit-in-place".

- **Dấu gạch chân (Underline)**
  - **Hiện tượng**: Ký tự đang gõ luôn bị gạch chân (marked text), gây rối mắt.
  - **Cách khắc phục**: Tuy đây là thiết kế mặc định của Apple, nhưng một số ứng dụng cho phép tắt trong phần cài đặt hiển thị (tùy app). Giải pháp tốt nhất vẫn là dùng bộ gõ ngoài hỗ trợ mode không gạch chân (Pre-edit Disable).

- **Tự sửa lỗi sai (Auto-correction)**
  - **Hiện tượng**: Gõ tiếng Anh bị tự sửa thành tiếng Việt hoặc ngược lại, gõ tên riêng bị sửa sai.
  - **Cách khắc phục**: Tắt tính năng này bằng cách vào **System Settings** -> **Keyboard** -> **Text Input** -> **Edit** -> Tắt **"Correct spelling automatically"**.

### 1.2. Windows Default Input (Microsoft Vietnamese Keyboard)

Đây là bộ gõ tích hợp sẵn trong Windows 10/11.

- **Lặp chữ trong Discord/Electron Apps**
  - **Hiện tượng**: Gõ `được` -> `đđược`, `học` -> `hhọc`. Thường gặp trên Discord, Slack.
  - **Cách khắc phục**: Hạn chế dùng bộ gõ mặc định của Windows cho các app này. Nên cài đặt Unikey hoặc EVKey để khắc phục.

---

## 2. Lỗi của Bộ gõ Bên thứ 3 (Unikey, EVKey, OpenKey...)

### 2.1. Lỗi trên Windows (Unikey, EVKey)

- **Không gõ được trong ứng dụng Admin**
  - **Hiện tượng**: Mở Task Manager, Regedit hoặc cài đặt phần mềm thì không gõ được tiếng Việt (mất biểu tượng bộ gõ hoặc gõ ra tiếng Anh).
  - **Cách khắc phục**: Chuột phải vào biểu tượng bộ gõ (Unikey/EVKey) -> Chọn **Run as Administrator**. Nên cài đặt để bộ gõ luôn khởi động với quyền Admin.

- **Nhảy chữ khi dùng tính năng Gợi ý từ**
  - **Hiện tượng**: Khi trình duyệt (Chrome/Edge) tự điền URL, nhấn Backspace hoặc gõ tiếp hay bị dính chữ (`dantri.com` -> `dân trí.com`).
  - **Cách khắc phục**:
    - Tắt tính năng "Gợi ý tìm kiếm và URL" trong cài đặt trình duyệt.
    - Hoặc sử dụng EVKey (có tùy chọn "Sửa lỗi Chrome/Excel").

### 2.2. Lỗi trên macOS (OpenKey, GoTiengViet)

- **Mất quyền Accessibility**
  - **Hiện tượng**: Không gõ được dấu dù biểu tượng bộ gõ vẫn sáng, hoặc gõ ra ký tự lạ.
  - **Cách khắc phục**: Vào **System Settings** -> **Privacy & Security** -> **Accessibility**. Chọn bộ gõ và nhấn dấu trừ (`-`) để xóa, sau đó nhấn dấu cộng (`+`) để thêm lại nhằm làm mới quyền.

- **Bị chặn bởi Secure Input**
  - **Hiện tượng**: Khi gõ password xong ở một app, chuyển sang app khác gõ tiếng Việt bị mất dấu hoàn toàn.
  - **Trên Gõ Nhanh**: Khi bộ gõ đang bật, biểu tượng menu bar chuyển thành `!` nếu macOS Secure Input đang chặn bộ gõ. Gõ Nhanh sẽ tự làm mới và hoạt động lại khi macOS nhả trạng thái này; không cần khởi động lại app.
    - Chuyển app, click rời ô mật khẩu, wake hoặc unlock sẽ kiểm tra ngay. Nếu ứng dụng nền nhả Secure Input mà không phát sinh sự kiện, watchdog dùng chung của bộ gõ thường sẽ phát hiện trong vòng khoảng 2–2,5 giây (có thể lâu hơn nếu hệ thống đang bận).
  - **Cách khắc phục**:
    - Rời ô nhập mật khẩu, chuyển tab/app, hoặc hoàn tất đăng nhập. Gõ Nhanh kiểm tra lại ngay khi bạn click hoặc chuyển ứng dụng.
    - Nếu dấu `!` vẫn còn, tìm ứng dụng quản lý mật khẩu hoặc ô mật khẩu đang giữ Secure Input và đóng nó lại.
    - Hoặc Logout và Login lại tài khoản macOS để reset trạng thái Secure Input.

---

## 3. Lỗi do Ứng dụng & Môi trường

Các lỗi này không hẳn do bộ gõ, mà do cách ứng dụng xử lý ký tự đầu vào.

### 3.1. Google Docs / Sheets

- **Hiện tượng**: Lặp chữ (`trường` -> `TTrường`) hoặc mất dấu khi gõ nhanh.
- **Nguyên nhân**: Google Docs sử dụng cơ chế Canvas/DOM riêng, đôi khi không đồng bộ kịp với tốc độ gửi phím của bộ gõ.
- **Cách khắc phục**:
  - Sử dụng bộ gõ có tính năng chuyên biệt "Sửa lỗi Google Docs" (như EVKey, OpenKey).
  - Tắt các Extension trình duyệt không cần thiết để giảm tải cho trang web.

### 3.2. Microsoft Excel

- **Hiện tượng**: Mất dấu các ký tự `ư`, `ơ` khi sửa lại nội dung ô (Edit cell) hoặc thanh hư tắt.
- **Cách khắc phục**:
  - Trong bộ gõ (Unikey/EVKey), tìm tùy chọn "Sửa lỗi Excel" hoặc "Luôn sử dụng Clipboard cho UniKey".
  - Tránh dùng các bảng mã cũ (TCVN3, VNI) nếu không bắt buộc.

### 3.3. Terminal & Vim

- **Hiện tượng**: Nhảy trỏ chuột sai vị trí, không dùng được phím tắt `Shift+:` hay `Esc` khi đang bật tiếng Việt.
- **Cách khắc phục**:
  - Tập thói quen tắt bộ gõ (về tiếng Anh) mỗi khi gõ lệnh hoặc chuyển mode trong Vim.
  - Cấu hình Terminal Profile để sử dụng bộ gõ tiếng Anh mặc định nếu có thể.

---

> **Lời khuyên chung**:
>
> - Để có trải nghiệm ổn định nhất, hãy luôn cập nhật bộ gõ lên phiên bản mới nhất.
> - Trên một máy tính chỉ nên kích hoạt **MỘT** bộ gõ tiếng Việt duy nhất tại một thời điểm để tránh xung đột.
