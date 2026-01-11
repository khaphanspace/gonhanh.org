// main.m - Minimal Objective-C macOS menu bar app with Settings
// Target: ~15-18MB RAM (same as EVKey)
// NO Swift - pure Objective-C

#import <Cocoa/Cocoa.h>

// Rust FFI declarations
extern void ime_init(void);
extern void ime_enabled(bool enabled);
extern void ime_method(uint8_t method);
extern void ime_clear(void);
extern void ime_modern(bool modern);
extern void ime_skip_w_shortcut(bool skip);
extern void ime_bracket_shortcut(bool enabled);
extern void ime_esc_restore(bool enabled);
extern void ime_english_auto_restore(bool enabled);
extern void ime_auto_capitalize(bool enabled);

// Settings keys
static NSString *const kEnabled = @"enabled";
static NSString *const kMethod = @"method";
static NSString *const kModernTone = @"modernTone";
static NSString *const kAutoWShortcut = @"autoWShortcut";
static NSString *const kBracketShortcut = @"bracketShortcut";
static NSString *const kEscRestore = @"escRestore";
static NSString *const kEnglishAutoRestore = @"englishAutoRestore";
static NSString *const kAutoCapitalize = @"autoCapitalize";
static NSString *const kSoundEnabled = @"soundEnabled";

// Global state
static BOOL g_enabled = YES;
static int g_method = 0; // 0=Telex, 1=VNI

// Forward declarations
@class SettingsWindowController;

// ============================================================================
// Settings Window Controller
// ============================================================================

@interface SettingsWindowController : NSObject
@property (nonatomic, strong) NSWindow *window;
+ (instancetype)shared;
- (void)showWindow;
@end

@implementation SettingsWindowController

+ (instancetype)shared {
    static SettingsWindowController *instance = nil;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        instance = [[SettingsWindowController alloc] init];
    });
    return instance;
}

- (instancetype)init {
    self = [super init];
    if (self) {
        [self createWindow];
    }
    return self;
}

- (void)createWindow {
    NSRect frame = NSMakeRect(0, 0, 500, 420);
    NSWindowStyleMask style = NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskMiniaturizable;

    self.window = [[NSWindow alloc] initWithContentRect:frame
                                              styleMask:style
                                                backing:NSBackingStoreBuffered
                                                  defer:NO];
    self.window.title = @"Gõ Nhanh - Cài đặt";
    [self.window center];
    self.window.releasedWhenClosed = NO;

    // Create tab view
    NSTabView *tabView = [[NSTabView alloc] initWithFrame:NSInsetRect(frame, 20, 20)];
    tabView.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;

    // General tab
    NSTabViewItem *generalTab = [[NSTabViewItem alloc] initWithIdentifier:@"general"];
    generalTab.label = @"Chung";
    generalTab.view = [self createGeneralTab];
    [tabView addTabViewItem:generalTab];

    // Advanced tab
    NSTabViewItem *advancedTab = [[NSTabViewItem alloc] initWithIdentifier:@"advanced"];
    advancedTab.label = @"Nâng cao";
    advancedTab.view = [self createAdvancedTab];
    [tabView addTabViewItem:advancedTab];

    // About tab
    NSTabViewItem *aboutTab = [[NSTabViewItem alloc] initWithIdentifier:@"about"];
    aboutTab.label = @"Giới thiệu";
    aboutTab.view = [self createAboutTab];
    [tabView addTabViewItem:aboutTab];

    self.window.contentView = tabView;
}

- (NSView *)createGeneralTab {
    NSView *view = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 460, 340)];
    CGFloat y = 300;

    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];

    // Input method selection
    NSTextField *methodLabel = [NSTextField labelWithString:@"Kiểu gõ:"];
    methodLabel.frame = NSMakeRect(20, y, 100, 22);
    [view addSubview:methodLabel];

    NSPopUpButton *methodPopup = [[NSPopUpButton alloc] initWithFrame:NSMakeRect(120, y - 2, 150, 26)];
    [methodPopup addItemsWithTitles:@[@"Telex", @"VNI"]];
    [methodPopup selectItemAtIndex:g_method];
    methodPopup.target = self;
    methodPopup.action = @selector(methodChanged:);
    [view addSubview:methodPopup];

    y -= 50;

    // Checkboxes
    struct {
        NSString *title;
        NSString *key;
        BOOL defaultValue;
        SEL action;
    } options[] = {
        {@"w → ư trong Telex", kAutoWShortcut, YES, @selector(autoWShortcutChanged:)},
        {@"Dấu thanh chuẩn (không tự do)", kModernTone, NO, @selector(modernToneChanged:)},
        {@"Tự động khôi phục tiếng Anh", kEnglishAutoRestore, YES, @selector(englishAutoRestoreChanged:)},
        {@"Âm thanh khi bật/tắt", kSoundEnabled, YES, @selector(soundEnabledChanged:)},
    };

    for (int i = 0; i < 4; i++) {
        NSButton *checkbox = [NSButton checkboxWithTitle:options[i].title
                                                  target:self
                                                  action:options[i].action];
        checkbox.frame = NSMakeRect(20, y, 350, 22);
        BOOL isOn = [defaults objectForKey:options[i].key] ? [defaults boolForKey:options[i].key] : options[i].defaultValue;
        checkbox.state = isOn ? NSControlStateValueOn : NSControlStateValueOff;
        [view addSubview:checkbox];
        y -= 35;
    }

    return view;
}

- (NSView *)createAdvancedTab {
    NSView *view = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 460, 340)];
    CGFloat y = 300;

    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];

    struct {
        NSString *title;
        NSString *key;
        BOOL defaultValue;
        SEL action;
    } options[] = {
        {@"[ ] → ơ ư (thay cho aa uw)", kBracketShortcut, NO, @selector(bracketShortcutChanged:)},
        {@"ESC khôi phục về ASCII", kEscRestore, YES, @selector(escRestoreChanged:)},
        {@"Tự động viết hoa sau dấu câu", kAutoCapitalize, NO, @selector(autoCapitalizeChanged:)},
    };

    for (int i = 0; i < 3; i++) {
        NSButton *checkbox = [NSButton checkboxWithTitle:options[i].title
                                                  target:self
                                                  action:options[i].action];
        checkbox.frame = NSMakeRect(20, y, 350, 22);
        BOOL isOn = [defaults objectForKey:options[i].key] ? [defaults boolForKey:options[i].key] : options[i].defaultValue;
        checkbox.state = isOn ? NSControlStateValueOn : NSControlStateValueOff;
        [view addSubview:checkbox];
        y -= 35;
    }

    return view;
}

- (NSView *)createAboutTab {
    NSView *view = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 460, 340)];

    // App icon placeholder
    NSImageView *iconView = [[NSImageView alloc] initWithFrame:NSMakeRect(190, 220, 80, 80)];
    iconView.image = [NSImage imageNamed:@"AppIcon"];
    iconView.imageScaling = NSImageScaleProportionallyUpOrDown;
    [view addSubview:iconView];

    // App name
    NSTextField *nameLabel = [NSTextField labelWithString:@"Gõ Nhanh"];
    nameLabel.font = [NSFont systemFontOfSize:24 weight:NSFontWeightBold];
    nameLabel.alignment = NSTextAlignmentCenter;
    nameLabel.frame = NSMakeRect(0, 180, 460, 30);
    [view addSubview:nameLabel];

    // Version
    NSString *version = [[NSBundle mainBundle] objectForInfoDictionaryKey:@"CFBundleShortVersionString"] ?: @"1.0";
    NSTextField *versionLabel = [NSTextField labelWithString:[NSString stringWithFormat:@"Phiên bản %@", version]];
    versionLabel.font = [NSFont systemFontOfSize:12];
    versionLabel.textColor = [NSColor secondaryLabelColor];
    versionLabel.alignment = NSTextAlignmentCenter;
    versionLabel.frame = NSMakeRect(0, 155, 460, 20);
    [view addSubview:versionLabel];

    // Description
    NSTextField *descLabel = [NSTextField labelWithString:@"Bộ gõ tiếng Việt nhanh, nhẹ, chính xác\n(Objective-C Edition)"];
    descLabel.font = [NSFont systemFontOfSize:13];
    descLabel.textColor = [NSColor secondaryLabelColor];
    descLabel.alignment = NSTextAlignmentCenter;
    descLabel.frame = NSMakeRect(0, 110, 460, 40);
    [view addSubview:descLabel];

    // Website button
    NSButton *websiteButton = [NSButton buttonWithTitle:@"gonhanh.org" target:self action:@selector(openWebsite:)];
    websiteButton.bezelStyle = NSBezelStyleInline;
    websiteButton.frame = NSMakeRect(185, 70, 90, 24);
    [view addSubview:websiteButton];

    return view;
}

// MARK: - Actions

- (void)methodChanged:(NSPopUpButton *)sender {
    g_method = (int)sender.indexOfSelectedItem;
    ime_method(g_method);
    [[NSUserDefaults standardUserDefaults] setInteger:g_method forKey:kMethod];
    NSLog(@"[ObjC] Method: %@", g_method == 0 ? @"Telex" : @"VNI");
}

- (void)autoWShortcutChanged:(NSButton *)sender {
    BOOL isOn = sender.state == NSControlStateValueOn;
    ime_skip_w_shortcut(!isOn);
    [[NSUserDefaults standardUserDefaults] setBool:isOn forKey:kAutoWShortcut];
}

- (void)modernToneChanged:(NSButton *)sender {
    BOOL isOn = sender.state == NSControlStateValueOn;
    ime_modern(isOn);
    [[NSUserDefaults standardUserDefaults] setBool:isOn forKey:kModernTone];
}

- (void)englishAutoRestoreChanged:(NSButton *)sender {
    BOOL isOn = sender.state == NSControlStateValueOn;
    ime_english_auto_restore(isOn);
    [[NSUserDefaults standardUserDefaults] setBool:isOn forKey:kEnglishAutoRestore];
}

- (void)soundEnabledChanged:(NSButton *)sender {
    BOOL isOn = sender.state == NSControlStateValueOn;
    [[NSUserDefaults standardUserDefaults] setBool:isOn forKey:kSoundEnabled];
}

- (void)bracketShortcutChanged:(NSButton *)sender {
    BOOL isOn = sender.state == NSControlStateValueOn;
    ime_bracket_shortcut(isOn);
    [[NSUserDefaults standardUserDefaults] setBool:isOn forKey:kBracketShortcut];
}

- (void)escRestoreChanged:(NSButton *)sender {
    BOOL isOn = sender.state == NSControlStateValueOn;
    ime_esc_restore(isOn);
    [[NSUserDefaults standardUserDefaults] setBool:isOn forKey:kEscRestore];
}

- (void)autoCapitalizeChanged:(NSButton *)sender {
    BOOL isOn = sender.state == NSControlStateValueOn;
    ime_auto_capitalize(isOn);
    [[NSUserDefaults standardUserDefaults] setBool:isOn forKey:kAutoCapitalize];
}

- (void)openWebsite:(id)sender {
    [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"https://gonhanh.org"]];
}

- (void)showWindow {
    [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
    [NSApp activateIgnoringOtherApps:YES];
    [self.window makeKeyAndOrderFront:nil];
}

@end

// ============================================================================
// AppDelegate
// ============================================================================

@interface AppDelegate : NSObject <NSApplicationDelegate, NSWindowDelegate>
@property (nonatomic, strong) NSStatusItem *statusItem;
@end

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    NSLog(@"[ObjC] App did finish launching");

    // Load saved settings
    [self loadSettings];

    // Initialize Rust engine
    ime_init();
    ime_enabled(g_enabled);
    ime_method(g_method);
    [self applySettings];

    // Create status bar item
    self.statusItem = [[NSStatusBar systemStatusBar] statusItemWithLength:NSVariableStatusItemLength];
    [self setupMenu];
    [self updateStatusIcon];

    NSLog(@"[ObjC] Menu bar app ready");
}

- (void)loadSettings {
    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];

    // Register defaults
    [defaults registerDefaults:@{
        kEnabled: @YES,
        kMethod: @0,
        kAutoWShortcut: @YES,
        kModernTone: @NO,
        kEnglishAutoRestore: @YES,
        kBracketShortcut: @NO,
        kEscRestore: @YES,
        kAutoCapitalize: @NO,
        kSoundEnabled: @YES,
    }];

    g_enabled = [defaults boolForKey:kEnabled];
    g_method = (int)[defaults integerForKey:kMethod];
}

- (void)applySettings {
    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];

    ime_skip_w_shortcut(![defaults boolForKey:kAutoWShortcut]);
    ime_modern([defaults boolForKey:kModernTone]);
    ime_english_auto_restore([defaults boolForKey:kEnglishAutoRestore]);
    ime_bracket_shortcut([defaults boolForKey:kBracketShortcut]);
    ime_esc_restore([defaults boolForKey:kEscRestore]);
    ime_auto_capitalize([defaults boolForKey:kAutoCapitalize]);
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {
    return NO;
}

// ============================================================================
// Menu
// ============================================================================

- (void)setupMenu {
    NSMenu *menu = [[NSMenu alloc] init];

    // Header
    NSMenuItem *header = [[NSMenuItem alloc] initWithTitle:@"Gõ Nhanh - Obj-C"
                                                    action:nil
                                             keyEquivalent:@""];
    header.enabled = NO;
    [menu addItem:header];
    [menu addItem:[NSMenuItem separatorItem]];

    // Toggle
    NSMenuItem *toggle = [[NSMenuItem alloc] initWithTitle:@"Bật/Tắt tiếng Việt"
                                                    action:@selector(toggleVietnamese:)
                                             keyEquivalent:@""];
    toggle.target = self;
    [menu addItem:toggle];
    [menu addItem:[NSMenuItem separatorItem]];

    // Methods
    NSMenuItem *telex = [[NSMenuItem alloc] initWithTitle:@"Telex"
                                                   action:@selector(selectTelex:)
                                            keyEquivalent:@""];
    telex.target = self;
    telex.tag = 0;
    telex.state = g_method == 0 ? NSControlStateValueOn : NSControlStateValueOff;
    [menu addItem:telex];

    NSMenuItem *vni = [[NSMenuItem alloc] initWithTitle:@"VNI"
                                                 action:@selector(selectVNI:)
                                          keyEquivalent:@""];
    vni.target = self;
    vni.tag = 1;
    vni.state = g_method == 1 ? NSControlStateValueOn : NSControlStateValueOff;
    [menu addItem:vni];
    [menu addItem:[NSMenuItem separatorItem]];

    // Settings
    NSMenuItem *settings = [[NSMenuItem alloc] initWithTitle:@"Cài đặt..."
                                                      action:@selector(showSettings:)
                                               keyEquivalent:@","];
    settings.target = self;
    [menu addItem:settings];
    [menu addItem:[NSMenuItem separatorItem]];

    // Quit
    NSMenuItem *quit = [[NSMenuItem alloc] initWithTitle:@"Thoát"
                                                  action:@selector(terminate:)
                                           keyEquivalent:@"q"];
    [menu addItem:quit];

    self.statusItem.menu = menu;
}

- (void)updateStatusIcon {
    NSString *title = g_enabled ? @"V" : @"E";
    self.statusItem.button.title = title;
}

- (void)updateMenuState {
    NSMenu *menu = self.statusItem.menu;
    for (NSMenuItem *item in menu.itemArray) {
        if (item.tag == 0) {
            item.state = g_method == 0 ? NSControlStateValueOn : NSControlStateValueOff;
        } else if (item.tag == 1) {
            item.state = g_method == 1 ? NSControlStateValueOn : NSControlStateValueOff;
        }
    }
}

// ============================================================================
// Actions
// ============================================================================

- (void)toggleVietnamese:(id)sender {
    g_enabled = !g_enabled;
    ime_enabled(g_enabled);
    [[NSUserDefaults standardUserDefaults] setBool:g_enabled forKey:kEnabled];
    [self updateStatusIcon];
    NSLog(@"[ObjC] Toggle Vietnamese: %@", g_enabled ? @"ON" : @"OFF");
}

- (void)selectTelex:(id)sender {
    g_method = 0;
    ime_method(0);
    [[NSUserDefaults standardUserDefaults] setInteger:0 forKey:kMethod];
    [self updateMenuState];
    NSLog(@"[ObjC] Method: Telex");
}

- (void)selectVNI:(id)sender {
    g_method = 1;
    ime_method(1);
    [[NSUserDefaults standardUserDefaults] setInteger:1 forKey:kMethod];
    [self updateMenuState];
    NSLog(@"[ObjC] Method: VNI");
}

- (void)showSettings:(id)sender {
    [[SettingsWindowController shared] showWindow];
}

// NSWindowDelegate
- (void)windowWillClose:(NSNotification *)notification {
    [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
}

@end

// ============================================================================
// Main
// ============================================================================

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSLog(@"[ObjC] Starting Objective-C macOS app...");

        NSApplication *app = [NSApplication sharedApplication];
        app.activationPolicy = NSApplicationActivationPolicyAccessory;

        AppDelegate *delegate = [[AppDelegate alloc] init];
        app.delegate = delegate;

        [app run];
    }
    return 0;
}
