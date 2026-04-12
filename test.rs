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
}
    let five_hundred = tup.0;
    let six_point_four = tup.1;
    let one = tup.2;
