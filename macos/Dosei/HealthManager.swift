import Foundation
import AppKit

class HealthManager: NSObject {
    // Singleton instance
    static let shared = HealthManager()
    
    // Health status
    private(set) var isHealthy = false
    private(set) var healthStatus = [String: Bool]()
    
    // Delegate to notify about health changes
    weak var delegate: HealthManagerDelegate?
    
    // Timer for periodic health checks
    private var healthCheckTimer: Timer?
    private let healthCheckInterval: TimeInterval = 30.0 // Check every 30 seconds
    
    private override init() {
        super.init()
        setupHealthCheck()
    }
    
    func setupHealthCheck() {
        // Initial health check
        checkHealth()
        
        // Set up timer for periodic health checks
        healthCheckTimer = Timer.scheduledTimer(withTimeInterval: healthCheckInterval, repeats: true) { [weak self] _ in
            self?.checkHealth()
        }
    }
    
    func checkHealth() {
        // Check CLI symlink
        let symlinkExists = checkCliSymlinkExists()
        
        // Check Rust service
        let rustServiceRunning = checkRustServiceRunning()
        
        // Check Docker installation and status
        let dockerInstalled = checkDockerInstalled()
        let dockerRunning = dockerInstalled ? checkDockerRunning() : false
        
        // Update health status dictionary
        healthStatus = [
            "cliSymlink": symlinkExists,
            "rustService": rustServiceRunning,
            "dockerInstalled": dockerInstalled,
            "dockerRunning": dockerRunning
        ]
        
        // Overall health is true only if all checks pass
        let newHealthStatus = symlinkExists && rustServiceRunning && dockerInstalled && dockerRunning
        
        // Only notify if health status changed
        if isHealthy != newHealthStatus {
            isHealthy = newHealthStatus
            delegate?.healthStatusChanged(isHealthy: isHealthy)
        }
    }
    
    func checkCliSymlinkExists() -> Bool {
        let symlinkPath = "/usr/local/bin/dosei"
        return FileManager.default.fileExists(atPath: symlinkPath)
    }
    
    func checkRustServiceRunning() -> Bool {
        // We'll need a reference to the process from AppDelegate
        // This will be set through the setRustProcess method
        return rustProcessIsRunning()
    }
    
    private var rustProcess: Process?
    private var rustProcessPID: Int32 = 0
    
    func setRustProcess(_ process: Process?) {
        self.rustProcess = process
        if let process = process {
            self.rustProcessPID = process.processIdentifier
        } else {
            self.rustProcessPID = 0
        }
    }
    
    func rustProcessIsRunning() -> Bool {
        // First check if we have a direct reference to the process
        if let process = rustProcess, process.isRunning {
            return true
        }
        
        // If we have a PID but lost the process reference, check if the PID is still running
        if rustProcessPID > 0 {
            let process = Process()
            process.executableURL = URL(fileURLWithPath: "/bin/ps")
            process.arguments = ["-p", String(rustProcessPID)]
            
            let pipe = Pipe()
            process.standardOutput = pipe
            
            do {
                try process.run()
                process.waitUntilExit()
                
                // If ps returns exit code 0, the process with PID exists
                return process.terminationStatus == 0
            } catch {
                print("Error checking Rust process status: \(error)")
            }
        }
        
        return false
    }
    
    func checkDockerInstalled() -> Bool {
        let symlinkPath = "/usr/local/bin/docker"
        return FileManager.default.fileExists(atPath: symlinkPath)
    }
    
    func checkDockerRunning() -> Bool {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/local/bin/docker")
        process.arguments = ["info"]
        
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe
        
        do {
            try process.run()
            process.waitUntilExit()
            
            // If the exit status is 0, docker is running
            return process.terminationStatus == 0
        } catch {
            print("Error checking Docker status: \(error)")
            return false
        }
    }
    
    func getDetailedHealthStatus() -> String {
        var status = "Health Status:\n\n"
        
        // CLI Symlink
        let symlinkStatus = healthStatus["cliSymlink"] == true
        status += "• CLI Symlink: \(symlinkStatus ? "✅" : "❌")\n"
        if !symlinkStatus {
            status += "   Action: Restart Dosei to trigger CLI install process\n\n"
        } else {
            status += "\n"
        }
        
        // Rust Service
        let rustStatus = healthStatus["rustService"] == true
        status += "• DoseiD: \(rustStatus ? "✅" : "❌")\n"
        if !rustStatus {
            status += "   Action: Restart Dosei :(\n\n"
        } else {
            status += "\n"
        }
        
        // Docker Installed
        let dockerInstalledStatus = healthStatus["dockerInstalled"] == true
        status += "• Docker Installed: \(dockerInstalledStatus ? "✅" : "❌")\n"
        if !dockerInstalledStatus {
            status += "   Action: Please install Docker Desktop from docker.com\n\n"
        } else {
            status += "\n"
        }
        
        // Docker Running
        let dockerRunningStatus = healthStatus["dockerRunning"] == true
        status += "• Docker Running: \(dockerRunningStatus ? "✅" : "❌")\n"
        if !dockerRunningStatus && dockerInstalledStatus {
            status += "   Action: Start Docker Desktop from your Applications folder\n\n"
        } else if !dockerRunningStatus {
            status += "\n"
        } else {
            status += "\n"
        }
        
        return status
    }
}

// Protocol for health status updates
protocol HealthManagerDelegate: AnyObject {
    func healthStatusChanged(isHealthy: Bool)
}
