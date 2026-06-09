use crate::types::{Point3D, CloudState};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::UdpSocket;

pub async fn run_udp_listener(port: u16, shared_state: Arc<Mutex<CloudState>>) {
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

                        let mut state = shared_state.lock().unwrap();
                        state.points.push(*point);

                        let now = Instant::now();
                        state.last_packet_time = now;
                        state.points_this_second += 1;

                        if now.duration_since(state.last_pps_calc).as_secs_f32() >= 1.0 {
                            state.current_pps = state.points_this_second;
                            state.points_this_second = 0;
                            state.last_pps_calc = now;
                        }

                        //limit 
                        if state.points.len() > 50_000 {
                            state.points.remove(0);
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
