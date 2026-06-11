#include "app_main.h"

#include "FreeRTOS.h"
#include "task.h"

#include "main.h"
#include "usart.h"
#include "LidarParser.hpp"
#include "Point.hpp"
//#include "Simulation.hpp"

#include <math.h>
#include <stdio.h>
#include <string.h>

const uint16_t BUFFER_SIZE = 512;
uint8_t dma_rx_buffer[BUFFER_SIZE];
uint16_t read_index = 0;
LidarParser parser;
LidarPacket latest_valid_packet;

void process_dma_ringbuffer() {
	uint16_t dma_counter = __HAL_DMA_GET_COUNTER(huart1.hdmarx); //counts backwards
	uint16_t write_index = BUFFER_SIZE - dma_counter;

	while (read_index != write_index) {
		uint8_t next_byte = dma_rx_buffer[read_index];

		if (parser.processByte(next_byte) == true) {
			latest_valid_packet = parser.getPacket();

			//first bit status (0111... mask) & 64 conversion
			float start_deg = (latest_valid_packet.start_angle & 0x7FFF)
					/ 64.0f;
			float end_deg = (latest_valid_packet.end_angle & 0x7FFF) / 64.0f;

			start_deg = fmodf(start_deg, 360.0f);
			if (start_deg < 0.0f)
				start_deg += 360.0f;

			end_deg = fmodf(end_deg, 360.0f);
			if (end_deg < 0.0f)
				end_deg += 360.0f;

			float diff = end_deg - start_deg;
			if (diff < 0.0f) {
				diff += 360.0;
			}

			float step_deg = diff / 11.0f;

			for (int i = 0; i < 12; i++) {
				uint16_t dist_mm = latest_valid_packet.points[i].distance_mm;

				if (dist_mm == 0) {
					continue;
				}

				Point3D tx_point;
				tx_point.quality = latest_valid_packet.points[i].quality;

				float current_deg = start_deg + (i * step_deg);
				current_deg = fmodf(current_deg, 360.0f);
				float current_rad = current_deg * (M_PI / 180.0f);

				tx_point.sync_word = 0xAA55;
				tx_point.x_mm = (int16_t) (dist_mm * cosf(current_rad));
				tx_point.y_mm = (int16_t) (dist_mm * sinf(current_rad));
				tx_point.z_mm = 0;

				HAL_UART_Transmit(&huart6, (uint8_t*) &tx_point,
						sizeof(Point3D), 10);
			}
		}

		read_index++;
		if (read_index >= BUFFER_SIZE) {
			read_index = 0;
		}
	}
}

void app_main(void) {
	HAL_UART_Receive_DMA(&huart1, dma_rx_buffer, sizeof(dma_rx_buffer));

//	xTaskCreate(vMotorSimulationTask, "Motor Simulation", 256, NULL, 3, NULL);
//	xTaskCreate(vLidarSimulationTask, "Lidar Simulation", 512, NULL, 2, NULL);

	while (1) {
		HAL_GPIO_TogglePin(LD2_GPIO_Port, LD2_Pin);

		process_dma_ringbuffer();

		vTaskDelay(pdMS_TO_TICKS(10));
	}
}


