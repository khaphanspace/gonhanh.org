#include "Engine.h"
#include "KeycodeMap.h"
#include <fstream>
#include <cstdlib>

namespace GoNhanh {

// Load method from config file
static InputMethod loadMethodFromConfig() {
    const char* home = std::getenv("HOME");
    if (!home) return InputMethod::Telex;

    std::string path = std::string(home) + "/.config/gonhanh/method";
    std::ifstream file(path);
    if (!file) return InputMethod::Telex;

    std::string method;
    std::getline(file, method);

    if (method == "vni" || method == "VNI") {
        return InputMethod::VNI;
    }
    return InputMethod::Telex;
}

GoNhanhEngine::GoNhanhEngine(fcitx::Instance* instance)
    : fcitxInstance_(instance)
    , factory_([](fcitx::InputContext& ic) {
        return new GoNhanhState(&ic);
    })
    , currentMethod_(loadMethodFromConfig())
{
    // Initialize Rust core
    RustBridge::initialize();
    RustBridge::setMethod(currentMethod_);
    GONHANH_INFO() << "GoNhanh engine initialized (method: "
                   << (currentMethod_ == InputMethod::Telex ? "Telex" : "VNI") << ")";

    // Register input context property factory
    instance->inputContextManager().registerProperty("goNhanhState", &factory_);
}

GoNhanhEngine::~GoNhanhEngine() {
    GONHANH_INFO() << "GoNhanh engine destroyed";
}

void GoNhanhEngine::activate(const fcitx::InputMethodEntry& entry,
                              fcitx::InputContextEvent& event) {
    GONHANH_DEBUG() << "Activate: " << entry.uniqueName();

    // Clear buffer on activation
    RustBridge::clear();
    RustBridge::setEnabled(enabled_);
    RustBridge::setMethod(currentMethod_);
}

void GoNhanhEngine::deactivate(const fcitx::InputMethodEntry& entry,
                                fcitx::InputContextEvent& event) {
    GONHANH_DEBUG() << "Deactivate: " << entry.uniqueName();

    // Clear buffer on deactivation
    RustBridge::clear();
}

void GoNhanhEngine::reset(const fcitx::InputMethodEntry& entry,
                           fcitx::InputContextEvent& event) {
    GONHANH_DEBUG() << "Reset";

    auto* ic = event.inputContext();
    auto* state = getState(ic);
    if (state) {
        state->reset();
    }
}

void GoNhanhEngine::keyEvent(const fcitx::InputMethodEntry& entry,
                              fcitx::KeyEvent& keyEvent) {
    // Skip key release events
    if (keyEvent.isRelease()) {
        return;
    }

    auto* ic = keyEvent.inputContext();

    // Skip if disabled
    if (!enabled_) {
        return;
    }

    // Skip modifier-only events (Ctrl, Alt, Shift alone)
    auto key = keyEvent.key();
    if (key.isModifier()) {
        return;
    }

    // Check for word break keys (space, punctuation, arrows)
    uint32_t keysym = key.sym();
    if (KeycodeMap::isBreakKey(keysym)) {
        RustBridge::clear();
        return;  // Let the key pass through
    }

    // Skip if Ctrl or Alt is held (shortcuts)
    auto states = key.states();
    if (states.test(fcitx::KeyState::Ctrl) ||
        states.test(fcitx::KeyState::Alt) ||
        states.test(fcitx::KeyState::Super)) {
        RustBridge::clear();
        return;
    }

    // Convert keysym to macOS keycode
    uint16_t macKeycode = KeycodeMap::keysymToMacKeycode(keysym);
    if (macKeycode == KeycodeMap::MacKey::UNKNOWN) {
        // Unknown key - pass through
        return;
    }

    // Get modifier states
    bool caps = states.test(fcitx::KeyState::CapsLock);
    bool ctrl = states.test(fcitx::KeyState::Ctrl);
    bool shift = states.test(fcitx::KeyState::Shift);

    // For letters: Shift XORs CapsLock (Shift+A with CapsLock = lowercase)
    if (KeycodeMap::isLetterKey(keysym)) {
        caps = caps != shift;  // XOR: true if exactly one is set
    }

    GONHANH_DEBUG() << "Key: keysym=" << keysym
                     << " macKey=" << macKeycode
                     << " caps=" << caps
                     << " shift=" << shift;

    // Process through Rust core
    auto [backspace, text] = RustBridge::processKey(macKeycode, caps, ctrl, shift);

    // If no action needed, pass through
    if (text.empty() && backspace == 0) {
        return;
    }

    GONHANH_DEBUG() << "Result: backspace=" << backspace << " text=\"" << text << "\"";

    // Delete characters (backspace)
    if (backspace > 0) {
        ic->deleteSurroundingText(-backspace, backspace);
    }

    // Commit new text
    if (!text.empty()) {
        ic->commitString(text);
    }

    // Filter the key (don't let original key through)
    keyEvent.filterAndAccept();
}

void GoNhanhEngine::setMethod(InputMethod method) {
    currentMethod_ = method;
    RustBridge::setMethod(method);
    GONHANH_INFO() << "Method set to: " << (method == InputMethod::Telex ? "Telex" : "VNI");
}

void GoNhanhEngine::setEnabled(bool enabled) {
    enabled_ = enabled;
    RustBridge::setEnabled(enabled);
    GONHANH_INFO() << "Enabled: " << (enabled ? "true" : "false");
}

} // namespace GoNhanh

// Fcitx5 addon factory registration
FCITX_ADDON_FACTORY(GoNhanh::GoNhanhEngineFactory);
