#ifndef GONHANH_ENGINE_H
#define GONHANH_ENGINE_H

#include <fcitx/inputmethodengine.h>
#include <fcitx/inputcontext.h>
#include <fcitx/instance.h>
#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx-utils/log.h>

#include "RustBridge.h"

FCITX_DEFINE_LOG_CATEGORY(gonhanh, "gonhanh");
#define GONHANH_DEBUG() FCITX_LOGC(gonhanh, Debug)
#define GONHANH_INFO() FCITX_LOGC(gonhanh, Info)
#define GONHANH_WARN() FCITX_LOGC(gonhanh, Warn)
#define GONHANH_ERROR() FCITX_LOGC(gonhanh, Error)

namespace GoNhanh {

// Input context state
class GoNhanhState : public fcitx::InputContextProperty {
public:
    GoNhanhState(fcitx::InputContext* ic) : ic_(ic) {}

    void reset() {
        RustBridge::clear();
    }

private:
    fcitx::InputContext* ic_;
};

// Main Fcitx5 engine class
// Note: Fcitx5 manages addon lifecycle - no singleton pattern needed
class GoNhanhEngine : public fcitx::InputMethodEngineV2 {
public:
    GoNhanhEngine(fcitx::Instance* instance);
    ~GoNhanhEngine();

    // InputMethodEngine interface
    void activate(const fcitx::InputMethodEntry& entry,
                  fcitx::InputContextEvent& event) override;
    void deactivate(const fcitx::InputMethodEntry& entry,
                    fcitx::InputContextEvent& event) override;
    void keyEvent(const fcitx::InputMethodEntry& entry,
                  fcitx::KeyEvent& keyEvent) override;
    void reset(const fcitx::InputMethodEntry& entry,
               fcitx::InputContextEvent& event) override;

    // Configuration
    void setMethod(InputMethod method);
    void setEnabled(bool enabled);

private:
    fcitx::Instance* fcitxInstance_;
    fcitx::FactoryFor<GoNhanhState> factory_;
    InputMethod currentMethod_ = InputMethod::Telex;
    bool enabled_ = true;

    // Get state for input context
    GoNhanhState* getState(fcitx::InputContext* ic) {
        return ic->propertyFor(&factory_);
    }
};

// Addon factory
class GoNhanhEngineFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance* create(fcitx::AddonManager* manager) override {
        return new GoNhanhEngine(manager->instance());
    }
};

} // namespace GoNhanh

#endif // GONHANH_ENGINE_H
