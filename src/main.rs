use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, Notify};
use std::sync::Arc;

const BROKER_HOST: &str = "localhost";
const BROKER_PORT: u16 = 1883;
const TOPIC: &str = " random topic";
const QOS: QoS = QoS::AtLeastOnce;

async fn create_mqtt_client() -> Result<(AsyncClient, tokio::task::JoinHandle<()>), Box<dyn Error>> {
    log::info!("port {}:{}", BROKER_HOST, BROKER_PORT);
    let mut mqtt_options = MqttOptions::new("mqtt-client", BROKER_HOST, BROKER_PORT);
    mqtt_options.set_keep_alive(Duration::from_secs(10));
    
    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    log::info!("mqtt client's created successfully");
    
    let _eventloop_task = tokio::spawn(async move {
        log::info!("mqtt Eventloop task started");
        loop {
            match eventloop.poll().await {
                Ok(_event) => log::debug!("event received"),
                Err(e) => log::error!("eventloop error: {:?}", e),
            }
        }
    });
    Ok((client, _eventloop_task))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure more detailed logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::info!("the application should now start");
    
    let (client, _eventloop_task) = create_mqtt_client().await?;
    log::info!("mqtt client initialized");
    
    let (tx, _rx) = mpsc::channel(10);
    let (notify_tx, mut notify_rx) = mpsc::channel(1);
    
    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(State {
        last_sent: Instant::now(),
        should_send: false,
    }));

    
    let notify_tx_clone1 = notify_tx.clone();
    let notify_tx_clone2 = notify_tx.clone();

    let _publisher = {
        let state = state.clone();
        let client = client.clone();
        let notify = notify.clone();
        let notify_tx = notify_tx_clone1;
        tokio::spawn(async move {
            log::info!("publisher task starts here");
            loop {
                let mut state = state.lock().await;
                if state.should_send {
                    state.should_send = false;
                    let elapsed = state.last_sent.elapsed();
                    drop(state);
                    
                    log::info!("ms since the last message was sent {:?}", elapsed);
                    match client.publish(TOPIC, QOS, false, "sample message".as_bytes()).await {
                        Ok(_) => log::info!("message published  "),
                        Err(e) => log::error!("Failed to publish message: {:?}", e),
                    }
                } else if Instant::now().duration_since(state.last_sent) >= Duration::from_secs(5) {

                    state.should_send = true;
                    state.last_sent = Instant::now();
                    drop(state);
                    notify.notify_one();
                    
                    match notify_tx.send(()).await {
                        Ok(_) => log::debug!("sent notification to first thread"),
                        Err(e) => log::error!("error while sending notification: {:?}", e),
                    }
                } else {
                    log::trace!("waiting for the notification");
                    drop(state);
                    notify.notified().await;
                }
                
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
    };

    let _subscriber = {
        let state = state.clone();
        let client = client.clone();
        let notify = notify.clone();
        let notify_tx = notify_tx_clone2;
        tokio::spawn(async move {
            log::info!("subscriber task's started");
            loop {
                log::debug!(" subscrbiing to topic: {}", TOPIC);
                match client.subscribe(TOPIC, QOS).await {
                    Ok(_) => {
                        log::info!("successfully subscribed to topic: {}", TOPIC);
                        let mut state = state.lock().await;
                        state.should_send = true;
                        drop(state);
                        notify.notify_one();
                        
                        match notify_tx.send(()).await {
                            Ok(_) => log::debug!("subscription notification sent"),
                            Err(e) => log::error!("subscription notification's failed: {:?}", e),
                        }
                    }
                    Err(e) => log::error!("can't subscribe to {}: {:?}", TOPIC, e),
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        })
    };

    log::info!("the main loop's starting now");
    loop {
        match notify_rx.recv().await {
            Some(_) => {
                log::debug!("received notification");
                match tx.send("sample message".to_string()).await {
                    Ok(_) => log::trace!("message sent through channel"),
                    Err(e) => log::error!("failed to send message through channel: {:?}", e),
                }
            },
            None => {
                
                break;
            }
        }
    }

    Ok(())
}

struct State {
    last_sent: Instant,
    should_send: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use tokio::sync::{mpsc, Mutex};
    use tokio::time;

    #[tokio::test]
    async fn test_thread_communication() {
        let (tx, mut rx) = mpsc::channel(10);
        let thread_completed = Arc::new(AtomicBool::new(false));
        let thread_completed_clone = thread_completed.clone();

        
        tokio::spawn(async move {
            tx.send("Hello from thread!".to_string()).await.unwrap();
            thread_completed_clone.store(true, Ordering::SeqCst);
        });

        
        let received = time::timeout(Duration::from_secs(2), rx.recv()).await;
        
        assert!(received.is_ok(), "Thread should send message");
        if let Ok(Some(msg)) = received {
            assert_eq!(msg, "Hello from thread!");
        }
        
        assert!(thread_completed.load(Ordering::SeqCst), "Thread should complete");
    }

    
    #[tokio::test]
    async fn test_multi_thread_synchronization() {
        let (tx, mut rx) = mpsc::channel(3);
        let thread_count = 3;
        let completed_threads = Arc::new(AtomicBool::new(false));

        
        for i in 0..thread_count {
            let tx_clone = tx.clone();
            let completed_threads_clone = completed_threads.clone();

            tokio::spawn(async move {
                tx_clone.send(format!("Thread {}", i)).await.unwrap();
                
                
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                if i == thread_count - 1 {
                    completed_threads_clone.store(true, Ordering::SeqCst);
                }
            });
        }

        
        drop(tx);

        
        let mut received_messages = Vec::new();
        while let Ok(Some(msg)) = time::timeout(Duration::from_secs(2), rx.recv()).await {
            received_messages.push(msg);
        }

        
        assert_eq!(received_messages.len(), thread_count, "all threads gotta send a message");
        assert!(completed_threads.load(Ordering::SeqCst), "all threads should complete");
    }

    
    #[tokio::test]
    async fn test_thread_safe_state_mutation() {
        let shared_state = Arc::new(Mutex::new(State {
            last_sent: Instant::now(),
            should_send: false,
        }));

        let thread_count = 5;
        let completed_threads = Arc::new(AtomicBool::new(false));

        
        for _ in 0..thread_count {
            let state_clone = shared_state.clone();
            let completed_threads_clone = completed_threads.clone();

            tokio::spawn(async move {
                let mut state = state_clone.lock().await;
                state.should_send = true;
                
                // Simulate work
                tokio::time::sleep(Duration::from_millis(50)).await;
                
                completed_threads_clone.store(true, Ordering::SeqCst);
            });
        }

        
        tokio::time::sleep(Duration::from_secs(1)).await;

        let state = shared_state.lock().await;
        assert!(state.should_send, "state should be mutated");
        assert!(completed_threads.load(Ordering::SeqCst), "All threads should complete");
    }




}

