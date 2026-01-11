// main.c - Pure C macOS menu bar app using Objective-C runtime
// Target: ~15MB RAM, smallest possible binary
//
// This demonstrates calling Cocoa/AppKit from Pure C
// WARNING: This is extremely verbose compared to Swift/Obj-C

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <objc/runtime.h>
#include <objc/message.h>
#include <CoreGraphics/CoreGraphics.h>

// Rust FFI declarations
extern void ime_init(void);
extern void ime_enabled(bool enabled);
extern void ime_method(uint8_t method);
extern void ime_clear(void);

// Helper macros for Objective-C runtime calls
#define cls(name) (id)objc_getClass(name)
#define sel(name) sel_registerName(name)
#define msg ((id(*)(id, SEL, ...))objc_msgSend)
#define msg_stret ((void(*)(void*, id, SEL, ...))objc_msgSend_stret)

// Global state
static id g_status_item = NULL;
static id g_app_delegate = NULL;
static bool g_enabled = true;
static int g_method = 0; // 0=Telex, 1=VNI

// Forward declarations
static void create_menu(void);
static void update_status_icon(void);

// ============================================================================
// Action handlers (called from menu items)
// ============================================================================

static void toggle_vietnamese(id self, SEL cmd, id sender) {
    (void)self; (void)cmd; (void)sender;
    g_enabled = !g_enabled;
    ime_enabled(g_enabled);
    update_status_icon();
    printf("[C] Toggle Vietnamese: %s\n", g_enabled ? "ON" : "OFF");
}

static void select_telex(id self, SEL cmd, id sender) {
    (void)self; (void)cmd; (void)sender;
    g_method = 0;
    ime_method(0);
    printf("[C] Method: Telex\n");
}

static void select_vni(id self, SEL cmd, id sender) {
    (void)self; (void)cmd; (void)sender;
    g_method = 1;
    ime_method(1);
    printf("[C] Method: VNI\n");
}

static void quit_app(id self, SEL cmd, id sender) {
    (void)self; (void)cmd; (void)sender;
    id app = msg(cls("NSApplication"), sel("sharedApplication"));
    msg(app, sel("terminate:"), NULL);
}

// ============================================================================
// AppDelegate methods
// ============================================================================

static void app_did_finish_launching(id self, SEL cmd, id notification) {
    (void)self; (void)cmd; (void)notification;
    printf("[C] App did finish launching\n");

    // Initialize Rust engine
    ime_init();
    ime_enabled(g_enabled);
    ime_method(g_method);

    // Create status bar item
    id status_bar = msg(cls("NSStatusBar"), sel("systemStatusBar"));
    g_status_item = msg(status_bar, sel("statusItemWithLength:"), -1.0); // NSVariableStatusItemLength
    msg(g_status_item, sel("retain"));

    // Create menu
    create_menu();
    update_status_icon();

    printf("[C] Menu bar app ready\n");
}

static BOOL app_should_terminate_after_last_window(id self, SEL cmd, id app) {
    (void)self; (void)cmd; (void)app;
    return NO;
}

// ============================================================================
// Menu creation
// ============================================================================

static id create_menu_item(const char* title, SEL action, id target) {
    id title_str = msg(cls("NSString"), sel("stringWithUTF8String:"), title);
    id key_equiv = msg(cls("NSString"), sel("stringWithUTF8String:"), "");

    id item = msg(cls("NSMenuItem"), sel("alloc"));
    item = msg(item, sel("initWithTitle:action:keyEquivalent:"), title_str, action, key_equiv);
    msg(item, sel("setTarget:"), target);

    return item;
}

static void create_menu(void) {
    id menu = msg(msg(cls("NSMenu"), sel("alloc")), sel("init"));

    // Header
    id header = create_menu_item("Go Nhanh - Pure C", NULL, NULL);
    msg(header, sel("setEnabled:"), NO);
    msg(menu, sel("addItem:"), header);

    // Separator
    id sep1 = msg(cls("NSMenuItem"), sel("separatorItem"));
    msg(menu, sel("addItem:"), sep1);

    // Toggle
    id toggle = create_menu_item("Bat/Tat tieng Viet", sel("toggleVietnamese:"), g_app_delegate);
    msg(menu, sel("addItem:"), toggle);

    // Separator
    id sep2 = msg(cls("NSMenuItem"), sel("separatorItem"));
    msg(menu, sel("addItem:"), sep2);

    // Methods
    id telex = create_menu_item("Telex", sel("selectTelex:"), g_app_delegate);
    msg(menu, sel("addItem:"), telex);

    id vni = create_menu_item("VNI", sel("selectVNI:"), g_app_delegate);
    msg(menu, sel("addItem:"), vni);

    // Separator
    id sep3 = msg(cls("NSMenuItem"), sel("separatorItem"));
    msg(menu, sel("addItem:"), sep3);

    // Quit
    id quit = create_menu_item("Thoat", sel("quitApp:"), g_app_delegate);
    msg(menu, sel("addItem:"), quit);

    // Set menu
    msg(g_status_item, sel("setMenu:"), menu);
}

static void update_status_icon(void) {
    id button = msg(g_status_item, sel("button"));

    // Create attributed string for icon
    const char* icon_text = g_enabled ? "V" : "E";
    id icon_str = msg(cls("NSString"), sel("stringWithUTF8String:"), icon_text);
    msg(button, sel("setTitle:"), icon_str);
}

// ============================================================================
// Main entry point
// ============================================================================

int main(int argc, char* argv[]) {
    (void)argc; (void)argv;

    printf("[C] Starting Pure C macOS app...\n");

    // Create autorelease pool
    id pool = msg(msg(cls("NSAutoreleasePool"), sel("alloc")), sel("init"));

    // Get shared application
    id app = msg(cls("NSApplication"), sel("sharedApplication"));

    // Create custom AppDelegate class
    Class delegate_class = objc_allocateClassPair((Class)cls("NSObject"), "AppDelegate", 0);

    // Add methods to delegate
    class_addMethod(delegate_class, sel("applicationDidFinishLaunching:"),
                    (IMP)app_did_finish_launching, "v@:@");
    class_addMethod(delegate_class, sel("applicationShouldTerminateAfterLastWindowClosed:"),
                    (IMP)app_should_terminate_after_last_window, "B@:@");
    class_addMethod(delegate_class, sel("toggleVietnamese:"),
                    (IMP)toggle_vietnamese, "v@:@");
    class_addMethod(delegate_class, sel("selectTelex:"),
                    (IMP)select_telex, "v@:@");
    class_addMethod(delegate_class, sel("selectVNI:"),
                    (IMP)select_vni, "v@:@");
    class_addMethod(delegate_class, sel("quitApp:"),
                    (IMP)quit_app, "v@:@");

    // Register the class
    objc_registerClassPair(delegate_class);

    // Create delegate instance and retain it to prevent deallocation
    g_app_delegate = msg(msg((id)delegate_class, sel("alloc")), sel("init"));
    msg(g_app_delegate, sel("retain")); // Keep strong reference

    // Instead of setDelegate (which uses weak refs), observe notification directly
    id nc = msg(cls("NSNotificationCenter"), sel("defaultCenter"));
    id notification_name = msg(cls("NSString"), sel("stringWithUTF8String:"),
                               "NSApplicationDidFinishLaunchingNotification");
    msg(nc, sel("addObserver:selector:name:object:"),
        g_app_delegate,
        sel("applicationDidFinishLaunching:"),
        notification_name,
        app);

    // Set activation policy (LSUIElement = accessory)
    msg(app, sel("setActivationPolicy:"), 1); // NSApplicationActivationPolicyAccessory

    // Run
    printf("[C] Running main loop...\n");
    msg(app, sel("run"));

    // Cleanup (never reached)
    msg(pool, sel("drain"));

    return 0;
}
