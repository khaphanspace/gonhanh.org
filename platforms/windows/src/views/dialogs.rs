//! Onboarding as a native TaskDialog

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::*;

use crate::services::settings::Method;
use crate::win::wide;

fn app_icon() -> *const u16 {
    crate::win::make_int_resource(1) // icon from app.rc
}

/// First run: choose Telex / VNI. Returns the chosen method.
pub fn run_onboarding(current: Method) -> Method {
    let telex = wide("Telex\naa → â   ow → ơ   dd → đ   s → sắc   f → huyền");
    let vni = wide("VNI\na6 → â   o7 → ơ   d9 → đ   1 → sắc   2 → huyền");
    let start = wide("Bắt đầu gõ tiếng Việt");
    let radios = [
        TASKDIALOG_BUTTON {
            nButtonID: 100,
            pszButtonText: telex.as_ptr(),
        },
        TASKDIALOG_BUTTON {
            nButtonID: 101,
            pszButtonText: vni.as_ptr(),
        },
    ];
    let buttons = [TASKDIALOG_BUTTON {
        nButtonID: 1,
        pszButtonText: start.as_ptr(),
    }];
    let title = wide("Chào mừng đến GoNhanh");
    let main = wide("Bộ gõ tiếng Việt nhanh và nhẹ");
    let content = wide("Không quảng cáo, không theo dõi, miễn phí.\n\nBạn quen dùng kiểu gõ nào?");
    let footer = wide(
        "GoNhanh chạy ở khay hệ thống, góc phải thanh tác vụ. Nhấp vào biểu tượng để bật/tắt, \
         nhấp chuột phải để đổi kiểu gõ và mở Cài đặt. Phím tắt bật/tắt: Ctrl + Space.",
    );

    let mut tdc: TASKDIALOGCONFIG = unsafe { std::mem::zeroed() };
    tdc.cbSize = std::mem::size_of::<TASKDIALOGCONFIG>() as u32;
    tdc.hInstance = unsafe { GetModuleHandleW(std::ptr::null()) };
    tdc.dwFlags = TDF_USE_COMMAND_LINKS | TDF_POSITION_RELATIVE_TO_WINDOW;
    tdc.pszWindowTitle = title.as_ptr();
    tdc.Anonymous1.pszMainIcon = app_icon();
    tdc.pszMainInstruction = main.as_ptr();
    tdc.pszContent = content.as_ptr();
    tdc.cRadioButtons = 2;
    tdc.pRadioButtons = radios.as_ptr();
    tdc.nDefaultRadioButton = if current == Method::Vni { 101 } else { 100 };
    tdc.cButtons = 1;
    tdc.pButtons = buttons.as_ptr();
    tdc.Anonymous2.pszFooterIcon = TD_INFORMATION_ICON;
    tdc.pszFooter = footer.as_ptr();

    let mut radio = 100i32;
    let ok =
        unsafe { TaskDialogIndirect(&tdc, std::ptr::null_mut(), &mut radio, std::ptr::null_mut()) }
            >= 0;
    if ok && radio == 101 {
        Method::Vni
    } else if ok {
        Method::Telex
    } else {
        current
    }
}
