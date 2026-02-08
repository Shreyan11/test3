// src/server.rs

struct ServerConfig {
    address: String,  // Renamed from 'host'
    port: u16,
    timeout: u32,     // Added new field
}

fn start_server(config: &ServerConfig) {
    // Updated to use 'address'
    println!("Attempting to bind to {}:{} with timeout {}s...", config.address, config.port, config.timeout);
    
    // Updated logic
    if config.port > 8000 && config.timeout < 60 {
        println!("Running on production port (unsafe timeout)");
    } else {
        println!("Configuration OK");
    }
}
