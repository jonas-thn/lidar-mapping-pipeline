#include "Simulation.hpp"
#include "Point.hpp"
#include "main.h"
#include "usart.h"
#include <math.h>

static volatile float simulated_motor_angle = 0.0f;

void vMotorSimulationTask(void *pvParameters) {
	const TickType_t xDelayPeriod = pdMS_TO_TICKS(10);
	const float angle_increment = 3.6f;

	while (1) {
		simulated_motor_angle += angle_increment;

		if (simulated_motor_angle >= 360.0f) {
			simulated_motor_angle -= 360.0f;
		}

		vTaskDelay(xDelayPeriod);
	}
}

void vLidarSimulationTask(void *pvParameters) {
	const TickType_t xDelayPeriod = pdMS_TO_TICKS(40);

	Point3D mock_point;
	mock_point.sync_word = 0xAA55;
	mock_point.quality = 0x0064;

	while (1) {
		float current_deg = simulated_motor_angle;
		float current_rad = current_deg * (M_PI / 180.0f);

		int16_t simulated_z = (int16_t) (300.0f * sinf(current_rad * 2.0f));

		uint16_t base_distance_mm = 1200;
		mock_point.x_mm = (int16_t) (base_distance_mm * cosf(current_rad));
		mock_point.y_mm = (int16_t) (base_distance_mm * sinf(current_rad));
		mock_point.z_mm = simulated_z;

		HAL_UART_Transmit(&huart6, (uint8_t*) &mock_point, sizeof(Point3D),
				10);

		vTaskDelay(xDelayPeriod);
	}
}

