import Foundation
import Sparkle

// MARK: - Update Manager (Sparkle Wrapper)
// Auto-check + auto-download always ON. Silent install on quit (SUAutomaticallyUpdate).

class UpdateManager: NSObject, ObservableObject {
    static let shared = UpdateManager()

    private var controller: SPUStandardUpdaterController!

    @Published var canCheckForUpdates = false
    @Published var updateAvailable = false
    @Published var isChecking = false
    private var showDialogOnFind = false

    private override init() {
        super.init()
        controller = SPUStandardUpdaterController(
            startingUpdater: false,
            updaterDelegate: self,
            userDriverDelegate: nil
        )
    }

    func start() {
        do {
            try controller.updater.start()
            controller.updater.publisher(for: \.canCheckForUpdates)
                .assign(to: &$canCheckForUpdates)
        } catch {
            NSLog("[UpdateManager] Failed to start: %@", error.localizedDescription)
        }
    }

    /// Silent background check — no Sparkle popup, updates badge only
    func checkInBackground() {
        showDialogOnFind = false
        isChecking = true
        controller.updater.checkForUpdatesInBackground()
    }

    /// User-initiated check — silent, but auto-show dialog if update found
    func checkAndShowIfAvailable() {
        showDialogOnFind = true
        isChecking = true
        controller.updater.checkForUpdatesInBackground()
    }

    /// Show Sparkle's update dialog (install + restart option)
    func checkForUpdates() {
        controller.checkForUpdates(nil)
    }
}

// MARK: - SPUUpdaterDelegate

extension UpdateManager: SPUUpdaterDelegate {
    func updater(_ updater: SPUUpdater, didFindValidUpdate item: SUAppcastItem) {
        NSLog("[UpdateManager] Found update: %@", item.versionString)
        DispatchQueue.main.async {
            self.updateAvailable = true
            self.isChecking = false
            if self.showDialogOnFind {
                self.showDialogOnFind = false
                // Delay to let background check cycle finish before showing UI
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                    self.checkForUpdates()
                }
            }
        }
    }

    func updater(_ updater: SPUUpdater, didFinishUpdateCycleFor updateCheck: SPUUpdateCheck, error: (any Error)?) {
        NSLog("[UpdateManager] Cycle finished, updateCheck=%d, error=%@", updateCheck.rawValue, String(describing: error))
        DispatchQueue.main.async { self.isChecking = false }
    }
}
