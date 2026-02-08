// src/server.rs

struct ServerConfig {
    host: String,
    port: u16,
}

fn start_server(config: &ServerConfig) {
    println!("Attempting to bind to {}:{}...", config.host, config.port);
    // Simulated connection logic
    if config.port > 8000 {
        println!("Running on production port");
    } else {
        println!("Running on dev port");
    }
    println!("Running on dev 11port");
}
