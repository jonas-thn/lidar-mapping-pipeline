#include "app_main.h"

#include "FreeRTOS.h"
#include "task.h"

#include "main.h"
#include "usart.h"
#include "i2c.h"
#include "LidarParser.hpp"
#include "Point.hpp"
#include "IMUSensor.hpp"
//#include "Simulation.hpp"

#include <math.h>
#include <stdio.h>
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

int _write(int file, char *ptr, int len) {
    HAL_UART_Transmit(&huart2, (uint8_t*)ptr, len, HAL_MAX_DELAY);
    return len;
}

#ifdef __cplusplus
}
#endif

enum class ScannerState {
	CALIBRATING, IDLE, SCANNING
};

static volatile ScannerState current_state = ScannerState::CALIBRATING;

static constexpr uint16_t RX_BUFFER_SIZE = 512;
static constexpr uint8_t MAX_POINTS_PER_PACKET = 12;
static constexpr float LIDAR_RAW_SCALE_FACTOR = 64.0f;
static constexpr float DEG_TO_RAD = static_cast<float>(M_PI / 180.0f);

static constexpr float ARM_OFFSET_MM = 400.0;

static uint8_t rx_buffer[RX_BUFFER_SIZE];
static uint16_t read_index = 0;
static LidarParser parser;
static LidarPacket latest_valid_packet;
static Point3D tx_buffer[MAX_POINTS_PER_PACKET];

static IMUSensor imu(&hi2c1);

TaskHandle_t xLidarTaskHandle = NULL;

void HAL_GPIO_EXTI_Callback(uint16_t GPIO_Pin) {
	if (GPIO_Pin == B1_Pin) {
		if (current_state == ScannerState::IDLE) {
			current_state = ScannerState::SCANNING;
		} else if (current_state == ScannerState::SCANNING) {
			current_state = ScannerState::IDLE;
		}
	}
}

static void process_dma_ringbuffer() {
	uint16_t dma_counter = __HAL_DMA_GET_COUNTER(huart1.hdmarx); //counts backwards
	uint16_t write_index = RX_BUFFER_SIZE - dma_counter;

	while (read_index != write_index) {
		uint8_t next_byte = rx_buffer[read_index];

		if (parser.processByte(next_byte) == true) {
			latest_valid_packet = parser.getPacket();

			if (current_state != ScannerState::SCANNING) {
				read_index++;
				if (read_index >= RX_BUFFER_SIZE) {
					read_index = 0;
				}
				continue;
			}

			const float current_yaw_deg = imu.getYawDegrees();
			const float current_yaw_rad = current_yaw_deg * DEG_TO_RAD;

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

				//lidar scan points (2D)
				const float y_local = static_cast<float>(dist_mm)
						* cosf(current_rad);
				const float z_local = static_cast<float>(dist_mm)
						* sinf(current_rad);

				//local points (arm out 3D)
				const float x_body = ARM_OFFSET_MM;
				const float y_body = y_local;
				const float z_body = z_local;

				tx_buffer[points_to_send].sync_word = 0xAA55;
				tx_buffer[points_to_send].quality =
						latest_valid_packet.points[i].quality;

				//world points (person rotating 3D)
				tx_buffer[points_to_send].x_mm = static_cast<int16_t>((x_body
						* cosf(current_yaw_rad))
						- (y_body * sinf(current_yaw_rad)));
				tx_buffer[points_to_send].y_mm = static_cast<int16_t>((x_body
						* sinf(current_yaw_rad))
						+ (y_body * cosf(current_yaw_rad)));
				tx_buffer[points_to_send].z_mm = static_cast<int16_t>(z_body);

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
		process_dma_ringbuffer();
	}
}

static void vImuProcessingTask(void *pvParameters) {
	HAL_GPIO_WritePin(LD2_GPIO_Port, LD2_Pin, GPIO_PIN_SET);

	if (imu.init()) {
		current_state = ScannerState::IDLE;
	}

	TickType_t xLastWakeTime = xTaskGetTickCount();
	const TickType_t xFrequency = pdMS_TO_TICKS(10);
	uint8_t led_counter = 0;

	while (1) {
		if (current_state != ScannerState::CALIBRATING) {
			imu.update();
		}

		if (current_state == ScannerState::IDLE) {
			if (led_counter % 50 == 0)
				HAL_GPIO_TogglePin(LD2_GPIO_Port, LD2_Pin);
		} else if (current_state == ScannerState::SCANNING) {
			if (led_counter % 5 == 0)
				HAL_GPIO_TogglePin(LD2_GPIO_Port, LD2_Pin);
		}

		led_counter++;
		vTaskDelayUntil(&xLastWakeTime, xFrequency);
	}
}

void app_main(void) {
	xTaskCreate(vImuProcessingTask, "IMU Processing", 512, NULL, 4, NULL);

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

