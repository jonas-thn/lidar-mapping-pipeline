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

static constexpr uint16_t RX_BUFFER_SIZE = 512;
static constexpr uint8_t MAX_POINTS_PER_PACKET = 12;
static constexpr float LIDAR_RAW_SCALE_FACTOR = 64.0f;
static constexpr float DEG_TO_RAD = static_cast<float>(M_PI / 180.0f);

static uint8_t rx_buffer[RX_BUFFER_SIZE];
static uint16_t read_index = 0;
static LidarParser parser;
static LidarPacket latest_valid_packet;
static Point3D tx_buffer[MAX_POINTS_PER_PACKET];

TaskHandle_t xLidarTaskHandle = NULL;

static void process_dma_ringbuffer() {
	uint16_t dma_counter = __HAL_DMA_GET_COUNTER(huart1.hdmarx); //counts backwards
	uint16_t write_index = RX_BUFFER_SIZE - dma_counter;

	while (read_index != write_index) {
		uint8_t next_byte = rx_buffer[read_index];

		if (parser.processByte(next_byte) == true) {
			latest_valid_packet = parser.getPacket();

			//first bit status (0111... mask) & 64 conversion
			float start_deg = static_cast<float>(latest_valid_packet.start_angle
					& 0x7FFF) / LIDAR_RAW_SCALE_FACTOR;
			float end_deg = static_cast<float>(latest_valid_packet.end_angle
					& 0x7FFF) / LIDAR_RAW_SCALE_FACTOR;

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

			uint8_t points_to_send = 0;

			for (int i = 0; i < MAX_POINTS_PER_PACKET; i++) {
				uint16_t dist_mm = latest_valid_packet.points[i].distance_mm;

				if (dist_mm == 0) {
					continue;
				}

				float current_deg = start_deg
						+ (static_cast<float>(i) * step_deg);
				current_deg = fmodf(current_deg, 360.0f);
				const float current_rad = current_deg * DEG_TO_RAD;

				tx_buffer[points_to_send].sync_word = 0xAA55;
				tx_buffer[points_to_send].quality =
						latest_valid_packet.points[i].quality;
				tx_buffer[points_to_send].x_mm = (int16_t) (dist_mm
						* cosf(current_rad));
				tx_buffer[points_to_send].y_mm = (int16_t) (dist_mm
						* sinf(current_rad));
				tx_buffer[points_to_send].z_mm = 0;

				points_to_send++;
			}

			if (points_to_send > 0) {
				while (huart6.gState != HAL_UART_STATE_READY) {
					vTaskDelay(pdMS_TO_TICKS(1));
				}

				HAL_UART_Transmit_DMA(&huart6, (uint8_t*) tx_buffer,
						sizeof(Point3D) * points_to_send);
			}
		}

		read_index++;
		if (read_index >= RX_BUFFER_SIZE) {
			read_index = 0;
		}
	}
}

static void vLidarProcessingTask(void *pvParameters) {
	while (1) {
		ulTaskNotifyTake(pdTRUE, portMAX_DELAY);
		HAL_GPIO_TogglePin(LD2_GPIO_Port, LD2_Pin);
		process_dma_ringbuffer();
	}
}

void app_main(void) {
	xTaskCreate(vLidarProcessingTask, "Lidar Processing", 1024, NULL, 3,
			&xLidarTaskHandle);

	HAL_UART_Receive_DMA(&huart1, rx_buffer, sizeof(rx_buffer));

	__HAL_UART_ENABLE_IT(&huart1, UART_IT_IDLE);

//	xTaskCreate(vMotorSimulationTask, "Motor Simulation", 256, NULL, 3, NULL);
//	xTaskCreate(vLidarSimulationTask, "Lidar Simulation", 512, NULL, 2, NULL);

	while (1) {
		vTaskDelay(portMAX_DELAY);
	}
}

