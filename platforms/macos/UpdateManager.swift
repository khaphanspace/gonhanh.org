import Foundation
import Sparkle

// MARK: - Update Manager (Sparkle Wrapper)

class UpdateManager: NSObject, ObservableObject {
    static let shared = UpdateManager()

    private var controller: SPUStandardUpdaterController!

    @Published var canCheckForUpdates = false

    var updater: SPUUpdater { controller.updater }

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
            NSLog("[UpdateManager] Updater started successfully")
            updater.publisher(for: \.canCheckForUpdates)
                .assign(to: &$canCheckForUpdates)
        } catch {
            NSLog("[UpdateManager] Failed to start updater: %@", error.localizedDescription)
        }
    }

    func checkForUpdates() {
        NSLog("[UpdateManager] checkForUpdates called, canCheck=%d", canCheckForUpdates)
        controller.checkForUpdates(nil)
    }

    var automaticallyChecksForUpdates: Bool {
        get { updater.automaticallyChecksForUpdates }
        set {
            updater.automaticallyChecksForUpdates = newValue
            updater.automaticallyDownloadsUpdates = newValue
        }
    }
}

// MARK: - SPUUpdaterDelegate

extension UpdateManager: SPUUpdaterDelegate {}
