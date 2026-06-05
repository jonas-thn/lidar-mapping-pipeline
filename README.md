# Lidar Mapping Pipeline

A custom 3D scanning setup built with a 360° LiDAR and a stepper motor. The project focuses on low-level embedded architecture, non-blocking data processing, and wireless communication to render a real-time point cloud on the GPU.

## Features
* Data Processing: continuous sensor packets via STM32 hardware timers and DMA
* Motor Control: PWM vertical sweep with an isolated power supply
* Wireless Data Bridge: built with ESP32 and UDP
* Custom Visualization: wgpu point cloud rendering 

## Showcase
<p width="100%">
  <img src="" width="49%" />
  <img src="" width="49%" />
</p>

## Tech Stack
* Core: STM32F411RE, C/C++, STM32 HAL, FreeRTOS, CubeMX
* Data Bridge: ESP32, C, ESP-IDF
* Visualization: wgpu, winit, Rust

## Architecture
```mermaid
graph LR
    %% Power Network (Subtle / Background)
    PWR[12V Power Supply] -.->|12V Power| B
    PWR -.->|12V Power| SD[LM2596 Step-Down Module]
    SD -.->|5V Logic Power| C
    SD -.->|5V Logic Power| D

    %% Data & Control Network (Bold / Foreground)
    A[LiDAR Sensor\n360° distance scan] ==>|UART| C
    C ==>|PWM Control| B[Stepper Motor\nvertical sweep]
    C[STM32 Microcontroller\nC++ · STMCube] ==>|UART| D
    D[ESP32 Wireless Bridge\nC · ESP-IDF · FreeRTOS] ==>|UDP| E
    E[3D Point Cloud\n Rust · wgpu]
```

