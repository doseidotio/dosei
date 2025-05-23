import SwiftUI
import AppKit

class AppDelegate: NSObject, NSApplicationDelegate, HealthManagerDelegate {
    var statusBarItem: NSStatusItem!
    var statusBarMenu: NSMenu!
    private var rustProcess: Process?
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        // This is the key to hiding the dock icon properly
        NSApp.setActivationPolicy(.accessory)
        
        // Set up the menu bar item
        setupMenuBar()
        
        // Start the service with a slight delay
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            self.startService()
            if !HealthManager.shared.checkCliSymlinkExists() {
                self.installCliAction()
            }
        }
        
        // Set up health manager delegate
        HealthManager.shared.delegate = self
    }
    
    func startService() {
        // Start the Rust application
        startRustApplication()
        
        // Trigger health check
        HealthManager.shared.checkHealth()
    }
    
    func stopService() {
        // Stop the Rust application if it's running
        stopRustApplication()
        
        // Trigger health check
        HealthManager.shared.checkHealth()
    }
    
    func startRustApplication() {
        // Create a new Process to run the Rust application
        rustProcess = Process()
        
        // Get the path to the Rust executable
        let bundle = Bundle.main
        let rustExecutablePath = bundle.bundlePath + "/Contents/Resources/macos-rust"
        
        // Set the executable path
        rustProcess?.executableURL = URL(fileURLWithPath: rustExecutablePath)
        
        // Set up a pipe to capture stdout
        let pipe = Pipe()
        rustProcess?.standardOutput = pipe
        
        // Start the process
        do {
            try rustProcess?.run()
            
            // Read output from the Rust application
            let fileHandle = pipe.fileHandleForReading
            fileHandle.readabilityHandler = { handle in
                let data = handle.availableData
                if let output = String(data: data, encoding: .utf8) {
                    // Print the raw output with color codes intact
                    print(output)
                }
            }
            
            // Pass the process to HealthManager
            HealthManager.shared.setRustProcess(rustProcess)
            
            print("Rust application started successfully")
        } catch {
            print("Failed to start Rust application: \(error)")
            // Inform health manager that process failed to start
            HealthManager.shared.setRustProcess(nil)
        }
    }
    
    func stopRustApplication() {
        // Gracefully stop the Rust process if it's running
        if let process = rustProcess, process.isRunning {
            print("Gracefully stopping Rust application...")
            
            // Send SIGINT to the process for graceful shutdown
            process.interrupt()
            
            // Since we're in applicationWillTerminate, we need to wait synchronously
            // but not too long to avoid delaying app termination
            let gracePeriod: TimeInterval = 1.5 // shorter grace period
            let start = Date()
            
            // Wait synchronously for the process to terminate
            while process.isRunning && Date().timeIntervalSince(start) < gracePeriod {
                Thread.sleep(forTimeInterval: 0.05) // Check more frequently
            }
            
            // If still running after grace period, force terminate
            if process.isRunning {
                print("Grace period expired, forcefully terminating Rust application")
                process.terminate()
                
                // Give it a very short time to actually terminate
                Thread.sleep(forTimeInterval: 0.1)
            } else {
                print("Rust application gracefully stopped")
            }
        }
        
        // Always clean up
        rustProcess = nil
        HealthManager.shared.setRustProcess(nil)
    }
    
    func setupMenuBar() {
        // Create the status bar item
        statusBarItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        
        // Set the status bar icon
        if let button = statusBarItem.button {
            button.image = NSImage(named: "MenuBarIcon")
            button.image?.isTemplate = true // This makes the icon adapt to dark/light mode
        }
        
        // Create the menu
        statusBarMenu = NSMenu()
        
        // Status item with custom view
        updateStatusItemWithCircle(isHealthy: false)
        
        // Open console in browser (localhost)
        let showHealthItem = NSMenuItem(title: "Open Health Status", action: #selector(showHealthDetails(_:)), keyEquivalent: "i")
        statusBarMenu.addItem(showHealthItem)
        
        statusBarMenu.addItem(NSMenuItem.separator())
        
        // Open console in browser (localhost)
        let showConsoleItem = NSMenuItem(title: "Open Console", action: #selector(openConsoleInLocalhost(_:)), keyEquivalent: "o")
        statusBarMenu.addItem(showConsoleItem)
        
        statusBarMenu.addItem(NSMenuItem.separator())
        
        // Quit app
        let quitItem = NSMenuItem(title: "Quit Dosei", action: #selector(quitApp(_:)), keyEquivalent: "q")
        statusBarMenu.addItem(quitItem)
        
        // Assign menu to status bar item
        statusBarItem.menu = statusBarMenu
    }
    
    func installCliAction() {
        let bundle = Bundle.main
        let doseiExecutablePath = bundle.bundlePath + "/Contents/Resources/dosei"
        let symlinkPath = "/usr/local/bin/dosei"
        
        let customPrompt = "Dosei needs administrator privileges to install the Dosei CLI."
        let appleScriptCommand = """
        do shell script "ln -sf '\(doseiExecutablePath)' '\(symlinkPath)'" with administrator privileges with prompt "\(customPrompt)"
        """
        
        let scriptObject = NSAppleScript(source: appleScriptCommand)
        
        DispatchQueue.global(qos: .utility).async {
            var error: NSDictionary?
            if let scriptResult = scriptObject?.executeAndReturnError(&error) {
                print("CLI installation successful: \(scriptResult)")
                HealthManager.shared.checkHealth()
            } else if let error = error {
                print("Failed to install CLI: \(error)")
            }
        }
    }
    
    @objc func showHealthDetails(_ sender: NSMenuItem) {
        let alert = NSAlert()
        alert.messageText = "Dosei Health Status"
        alert.informativeText = HealthManager.shared.getDetailedHealthStatus()
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
    
    func updateStatusItemWithCircle(isHealthy: Bool) {
        // Remove the old status item if it exists
        if statusBarMenu.items.count > 0 {
            statusBarMenu.removeItem(at: 0)
        }
        
        // Create a new status menu item
        let statusItem = NSMenuItem(title: "", action: nil, keyEquivalent: "")
        
        // Create a custom view with the circle and text
        let customView = NSView(frame: NSRect(x: 0, y: 0, width: 200, height: 20))
        
        // Create the circle view
        let circleView = NSView(frame: NSRect(x: 10, y: 5, width: 10, height: 10))
        circleView.wantsLayer = true
        circleView.layer?.cornerRadius = 5
        circleView.layer?.backgroundColor = isHealthy ? NSColor.green.cgColor : NSColor.red.cgColor
        
        // Create the status text
        let statusText = NSTextField(frame: NSRect(x: 30, y: 2, width: 160, height: 16))
        statusText.stringValue = "Status: \(isHealthy ? "Healthy" : "Unhealthy")"
        statusText.isEditable = false
        statusText.isSelectable = false
        statusText.isBezeled = false
        statusText.drawsBackground = false
        statusText.textColor = .systemGray
        
        // Add the views to the custom view
        customView.addSubview(circleView)
        customView.addSubview(statusText)
        
        // Set the custom view as the view for the menu item
        statusItem.view = customView
        
        // Insert at the beginning of the menu
        statusBarMenu.insertItem(statusItem, at: 0)
    }
    
    @objc func openConsoleInLocalhost(_ sender: NSMenuItem) {
        // Direct localhost fallback
        if let url = URL(string: "http://127.0.0.1:8844") {
            NSWorkspace.shared.open(url)
        }
    }
    
    // Health Manager Delegate
    func healthStatusChanged(isHealthy: Bool) {
        DispatchQueue.main.async {
            self.updateStatusItemWithCircle(isHealthy: isHealthy)
        }
    }
    
    @objc func quitApp(_ sender: NSMenuItem) {
        NSApplication.shared.terminate(self)
    }
    
    func applicationWillTerminate(_ notification: Notification) {
        // Stop the service when the app terminates
        stopService()
    }
}
