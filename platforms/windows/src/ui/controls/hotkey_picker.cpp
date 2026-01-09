#include "hotkey_picker.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

void HotkeyPicker::draw(
    ID2D1RenderTarget* rt,
    float x, float y,
    float width,
    uint32_t modifiers,
    uint32_t vk,
    bool recording,
    bool hover
) {
    auto& renderer = D2DRenderer::instance();

    // Background color based on state
    D2D1::ColorF bg_color = recording
        ? D2D1::ColorF(0.96f, 0.98f, 1.0f)  // Light blue when recording
        : (hover ? D2D1::ColorF(0.97f, 0.97f, 0.98f) : D2D1::ColorF(0.96f, 0.96f, 0.97f));

    D2D1::ColorF border_color = recording
        ? Colors::Primary  // Blue border when recording
        : Colors::Border;

    auto bg_brush = create_brush(rt, bg_color);
    auto border_brush = create_brush(rt, border_color);

    D2D1_RECT_F rect = {x, y, x + width, y + HEIGHT};
    D2D1_ROUNDED_RECT rounded = {rect, BORDER_RADIUS, BORDER_RADIUS};

    rt->FillRoundedRectangle(rounded, bg_brush.Get());
    rt->DrawRoundedRectangle(rounded, border_brush.Get(), recording ? 2.0f : 1.0f);

    // Text
    std::wstring text;
    D2D1::ColorF text_color;

    if (recording) {
        auto& recorder = HotkeyRecorder::instance();
        if (recorder.current_vk() != 0) {
            text = hotkey_to_string(recorder.current_modifiers(), recorder.current_vk());
            text_color = Colors::Text;
        } else if (recorder.current_modifiers() != 0) {
            text = hotkey_to_string(recorder.current_modifiers(), 0);
            text += L"...";
            text_color = Colors::TextSecondary;
        } else {
            text = L"Nhập phím tắt...";
            text_color = Colors::TextTertiary;
        }
    } else if (vk != 0) {
        text = hotkey_to_string(modifiers, vk);
        text_color = Colors::Text;
    } else {
        text = L"Chưa đặt";
        text_color = Colors::TextTertiary;
    }

    auto text_brush = create_brush(rt, text_color);
    D2D1_RECT_F text_rect = {x + PADDING, y, x + width - PADDING, y + HEIGHT};

    rt->DrawText(
        text.c_str(),
        static_cast<UINT32>(text.length()),
        renderer.text_format_small(),
        text_rect,
        text_brush.Get()
    );
}

bool HotkeyPicker::hit_test(float x, float y, float width, float mx, float my) {
    return mx >= x && mx <= x + width && my >= y && my <= y + HEIGHT;
}

std::wstring HotkeyPicker::hotkey_to_string(uint32_t modifiers, uint32_t vk) {
    std::wstring result;

    if (modifiers & MOD_CONTROL) {
        result += L"Ctrl+";
    }
    if (modifiers & MOD_ALT) {
        result += L"Alt+";
    }
    if (modifiers & MOD_SHIFT) {
        result += L"Shift+";
    }
    if (modifiers & MOD_WIN) {
        result += L"Win+";
    }

    if (vk != 0) {
        result += vk_to_string(vk);
    }

    return result;
}

std::wstring HotkeyPicker::vk_to_string(uint32_t vk) {
    // Letters A-Z
    if (vk >= 'A' && vk <= 'Z') {
        return std::wstring(1, static_cast<wchar_t>(vk));
    }

    // Numbers 0-9
    if (vk >= '0' && vk <= '9') {
        return std::wstring(1, static_cast<wchar_t>(vk));
    }

    // Function keys F1-F12
    if (vk >= VK_F1 && vk <= VK_F12) {
        return L"F" + std::to_wstring(vk - VK_F1 + 1);
    }

    // Special keys
    switch (vk) {
        case VK_SPACE:   return L"Space";
        case VK_RETURN:  return L"Enter";
        case VK_TAB:     return L"Tab";
        case VK_ESCAPE:  return L"Esc";
        case VK_BACK:    return L"Backspace";
        case VK_DELETE:  return L"Delete";
        case VK_INSERT:  return L"Insert";
        case VK_HOME:    return L"Home";
        case VK_END:     return L"End";
        case VK_PRIOR:   return L"PageUp";
        case VK_NEXT:    return L"PageDown";
        case VK_LEFT:    return L"Left";
        case VK_RIGHT:   return L"Right";
        case VK_UP:      return L"Up";
        case VK_DOWN:    return L"Down";
        case VK_PAUSE:   return L"Pause";
        case VK_SNAPSHOT: return L"PrintScreen";
        case VK_SCROLL:  return L"ScrollLock";
        case VK_NUMLOCK: return L"NumLock";
        case VK_OEM_PLUS:   return L"=";
        case VK_OEM_MINUS:  return L"-";
        case VK_OEM_COMMA:  return L",";
        case VK_OEM_PERIOD: return L".";
        case VK_OEM_1:   return L";";
        case VK_OEM_2:   return L"/";
        case VK_OEM_3:   return L"`";
        case VK_OEM_4:   return L"[";
        case VK_OEM_5:   return L"\\";
        case VK_OEM_6:   return L"]";
        case VK_OEM_7:   return L"'";
        // Numpad
        case VK_NUMPAD0: return L"Num0";
        case VK_NUMPAD1: return L"Num1";
        case VK_NUMPAD2: return L"Num2";
        case VK_NUMPAD3: return L"Num3";
        case VK_NUMPAD4: return L"Num4";
        case VK_NUMPAD5: return L"Num5";
        case VK_NUMPAD6: return L"Num6";
        case VK_NUMPAD7: return L"Num7";
        case VK_NUMPAD8: return L"Num8";
        case VK_NUMPAD9: return L"Num9";
        case VK_MULTIPLY: return L"Num*";
        case VK_ADD:      return L"Num+";
        case VK_SUBTRACT: return L"Num-";
        case VK_DECIMAL:  return L"Num.";
        case VK_DIVIDE:   return L"Num/";
        default:
            // Return hex code for unknown keys
            wchar_t buf[8];
            swprintf(buf, 8, L"0x%02X", vk);
            return buf;
    }
}

// HotkeyRecorder implementation

HotkeyRecorder& HotkeyRecorder::instance() {
    static HotkeyRecorder instance;
    return instance;
}

void HotkeyRecorder::start_recording(Callback on_complete) {
    recording_ = true;
    current_modifiers_ = 0;
    current_vk_ = 0;
    callback_ = on_complete;
}

void HotkeyRecorder::cancel() {
    recording_ = false;
    current_modifiers_ = 0;
    current_vk_ = 0;
    callback_ = nullptr;
}

bool HotkeyRecorder::process_key(uint32_t vk, bool key_down) {
    if (!recording_) return false;

    // Update modifier state
    if (key_down) {
        switch (vk) {
            case VK_CONTROL:
            case VK_LCONTROL:
            case VK_RCONTROL:
                current_modifiers_ |= MOD_CONTROL;
                return true;
            case VK_MENU:
            case VK_LMENU:
            case VK_RMENU:
                current_modifiers_ |= MOD_ALT;
                return true;
            case VK_SHIFT:
            case VK_LSHIFT:
            case VK_RSHIFT:
                current_modifiers_ |= MOD_SHIFT;
                return true;
            case VK_LWIN:
            case VK_RWIN:
                current_modifiers_ |= MOD_WIN;
                return true;
            case VK_ESCAPE:
                // Cancel recording on ESC
                cancel();
                return true;
            default:
                // Non-modifier key pressed - complete recording
                if (current_modifiers_ != 0) {
                    current_vk_ = vk;
                    recording_ = false;
                    if (callback_) {
                        callback_(current_modifiers_, current_vk_);
                        callback_ = nullptr;
                    }
                }
                return true;
        }
    } else {
        // Key up - update modifier state
        switch (vk) {
            case VK_CONTROL:
            case VK_LCONTROL:
            case VK_RCONTROL:
                current_modifiers_ &= ~MOD_CONTROL;
                return true;
            case VK_MENU:
            case VK_LMENU:
            case VK_RMENU:
                current_modifiers_ &= ~MOD_ALT;
                return true;
            case VK_SHIFT:
            case VK_LSHIFT:
            case VK_RSHIFT:
                current_modifiers_ &= ~MOD_SHIFT;
                return true;
            case VK_LWIN:
            case VK_RWIN:
                current_modifiers_ &= ~MOD_WIN;
                return true;
        }
    }

    return true;
}

} // namespace gonhanh::ui
