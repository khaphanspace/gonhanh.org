#pragma once
#include <windows.h>
#include <vector>

namespace gonhanh {

// Settings window: fully custom-painted (Direct2D/DirectWrite), Windows 11 (Fluent) styled
// with a Mica backdrop where available.
// Sidebar with app info and links + scrollable page of grouped setting cards.
class SettingsWindow {
public:
    static SettingsWindow& Instance();
    void Show();
    void Hide();
    bool IsVisible() const { return visible_; }
    // Settings changed elsewhere (tray, hotkey): animate toggles to the new state and repaint
    void Refresh();
    // Destroy the (hidden) window and release rendering resources; Show() recreates it
    void DestroyIfHidden();

private:
    SettingsWindow() = default;
    ~SettingsWindow();
    SettingsWindow(const SettingsWindow&) = delete;
    SettingsWindow& operator=(const SettingsWindow&) = delete;

    enum class Hit {
        None, Row, Segment, Scrollbar, Link, AuthorLink, CommunityLink, UpdateBadge
    };
    struct HitResult {
        Hit kind = Hit::None;
        int index = -1;
        bool operator==(const HitResult& o) const { return kind == o.kind && index == o.index; }
        bool operator!=(const HitResult& o) const { return !(*this == o); }
    };

    void Create();
    void ApplyWindowTheme();
    void Layout();
    int S(int logical) const;  // Scale logical px to device px

    // Painting (Direct2D into a 32-bit buffer so Mica shows through transparent areas)
    void Paint(HDC hdc);
    void PaintSidebar();
    void PaintSettings();

    // Interaction
    HitResult HitTest(POINT pt) const;
    void Activate(const HitResult& hit);
    void ToggleRow(int row);
    void SetMethod(int method);
    void StartUpdateCheck();
    void ScrollTo(int pos);
    bool ScrollbarGeometry(RECT& track, RECT& thumb) const;
    bool SyncToggleAnim(bool snap);  // returns true while any toggle is still moving

    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);

    HWND hwnd_ = nullptr;
    bool visible_ = false;
    bool mica_ = false;  // Mica backdrop active (Windows 11 22H2+)
    UINT dpi_ = 96;

    // Settings page layout (content coordinates, i.e. before scrolling)
    std::vector<RECT> rowRects_;
    RECT segmentRects_[2] = {};
    int contentHeight_ = 0;
    int scrollPos_ = 0;

    // Toggle animation progress per row (0 = off, 1 = on)
    std::vector<float> toggleAnim_;
    // Telex/VNI selector pill position (0 = Telex, 1 = VNI)
    float methodAnim_ = 0.0f;

    // Hit rects in client coordinates (set while painting)
    RECT linkRects_[3] = {};  // Sidebar: Ủng hộ, Báo lỗi, GitHub
    RECT authorLinkRect_ = {};
    RECT communityLinkRect_ = {};
    RECT updateBadgeRect_ = {};

    // Mouse state
    HitResult hover_;
    HitResult pressed_;
    bool trackingMouse_ = false;
    bool draggingThumb_ = false;
    int dragStartY_ = 0;
    int dragStartScroll_ = 0;
};

} // namespace gonhanh
