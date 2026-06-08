use crate::types::Point3D;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;

pub async fn run_udp_listener(port: u16, shared_point_cloud: Arc<Mutex<Vec<Point3D>>>) {
    let addr = format!("0.0.0.0:{}", port);
    log::info!("Listening UDP on {}", addr);

    let socket = UdpSocket::bind(&addr).await.expect("ERROR: UDP binding.");
    log::info!("UDP connected.");

    let mut rx_buffer = [0u8; 1024];
    let mut byte_stream: Vec<u8> = Vec::new();

    loop {
        match socket.recv_from(&mut rx_buffer).await {
            Ok((size, _peer_addr)) => {
                byte_stream.extend_from_slice(&rx_buffer[..size]);

                while byte_stream.len() >= 10 {
                    if byte_stream[0] == 0x55 && byte_stream[1] == 0xAA {
                        let point_bytes = &byte_stream[0..10];
                        let point: &Point3D = bytemuck::from_bytes(point_bytes);

                        let mut cloud = shared_point_cloud.lock().unwrap();
                        cloud.push(*point);

                        //IMPORTANT
                        if cloud.len() > 15 {
                            cloud.remove(0);
                        }

                        byte_stream.drain(0..10);
                    } else {
                        byte_stream.remove(0);
                    }
                }
            }
            Err(e) => {
                log::error!("ERROR: Network: {}", e);
            }
        }
    }
}
