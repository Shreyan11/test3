// src/server.rs

struct ServerConfig {
    host: String,
    port: u16,
}
 let tup: (i32, f64, u8) = (500, 6.4, 1);
    let (x, y, z) = tup;
    println!("x: {}, y: {}, z: {}", x, y, z);

fn start_server(config: &ServerConfig) {
    println!("Attempting to bind to {}:{}...", config.host, config.port);
    // Simulated connection logic
    if config.port > 8000 {
        println!("Running on production port");
    } else {
        println!("Running on dev port");
    }
}
