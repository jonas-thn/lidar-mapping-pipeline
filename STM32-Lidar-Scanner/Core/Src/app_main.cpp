#include "app_main.h"
#include "main.h"
#include "usart.h"
#include "LidarParser.hpp"

#include <math.h>
#include <stdio.h>
#include <string.h>

struct Point2D {
	float x;
	float y;
	uint16_t quality;
};

Point2D scan_cloud[12];

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
			float start_deg = (latest_valid_packet.start_angle & 0x7FFF) / 64.0f;
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
				scan_cloud[i].quality = latest_valid_packet.points[i].quality;

				if (dist_mm == 0) {
					scan_cloud[i].x = 0.0f;
					scan_cloud[i].y = 0.0f;
					continue;
				}

				float current_deg = start_deg + (i * step_deg);
				current_deg = fmodf(current_deg, 360.0f);

				float current_rad = current_deg * (M_PI / 180.0f);

				scan_cloud[i].x = dist_mm * cosf(current_rad);
				scan_cloud[i].y = dist_mm * sinf(current_rad);
			}

			char tx_buffer[32];
			for (int i = 0; i < 12; i++) {
			    if (scan_cloud[i].x == 0.0f && scan_cloud[i].y == 0.0f) {
			        continue;
			    }

			    int x_mm = (int)scan_cloud[i].x;
			    int y_mm = (int)scan_cloud[i].y;

			    sprintf(tx_buffer, "%d,%d\n", x_mm, y_mm);

			    HAL_UART_Transmit(&huart6, (uint8_t*)tx_buffer, strlen(tx_buffer), 10);
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

	while (1) {
		HAL_GPIO_TogglePin(LD2_GPIO_Port, LD2_Pin);

		process_dma_ringbuffer();

		HAL_Delay(500);
	}
}

