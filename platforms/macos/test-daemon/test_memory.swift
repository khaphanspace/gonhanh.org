#!/usr/bin/env swift
// Minimal test to check memory footprint without SwiftUI
// Build: swiftc -O -L.. -lgonhanh_core test_memory.swift -o test_daemon
// Run: ./test_daemon

import Foundation

// FFI declarations (minimal)
@_silgen_name("ime_init") func ime_init()

print("GoNhanh Daemon Memory Test")
print("===========================")

// Initialize Rust engine
ime_init()
print("✓ Rust engine initialized")

// Keep running
print("")
print("Daemon running... Check memory with:")
print("  ps aux | grep test_daemon")
print("  vmmap -summary $(pgrep test_daemon) | grep 'Physical footprint'")
print("")
print("Press Ctrl+C to stop")

// Simple run loop
RunLoop.current.run()
