#include "IMUSensor.hpp"
#include "FreeRTOS.h"
#include "task.h"
#include <math.h>
#include <stdio.h>

static constexpr uint16_t MPU6050_ADDR = (0x68 << 1);
static constexpr uint16_t REG_POWER_CONTROL_1 = 0x6B;
static constexpr uint16_t REG_GYRO_CONFIG = 0x1B;
static constexpr uint16_t REG_GYRO_Z_HIGH_BYTE = 0x47;
static constexpr uint16_t REG_IDENTIFICATION = 0x75;
static constexpr uint8_t EXPECTED_IDENTIFICATION = 0x68;
static constexpr uint8_t GYRO_RANGE_500DPS = 0x08;
static constexpr float GYRO_SENSITIVITY_500DPS = 65.5f; //32767(max) / 500deg/s

IMUSensor::IMUSensor(I2C_HandleTypeDef *hi2c) :
		hi2c(hi2c), current_yaw_deg(0.0f), gyro_zero_rate_bias(0.0f), last_update_ticks(
				0) {

}

bool IMUSensor::init() {
	uint8_t check;
	uint8_t data;

	vTaskDelay(pdMS_TO_TICKS(100));

	//reply ckeck
	HAL_I2C_Mem_Read(hi2c, MPU6050_ADDR, REG_IDENTIFICATION, 1, &check, 1, 100);

	if (check != EXPECTED_IDENTIFICATION) {
		return false;
	}

	data = 0x80; //device reset
	HAL_I2C_Mem_Write(hi2c, MPU6050_ADDR, REG_POWER_CONTROL_1, 1, &data, 1,
			100);

	vTaskDelay(pdMS_TO_TICKS(100));

	//clear sleep bit
	data = 0x00;
	if (HAL_I2C_Mem_Write(hi2c, MPU6050_ADDR, REG_POWER_CONTROL_1, 1, &data, 1,
			100) != HAL_OK) {
		return false;
	}

	//gyroskope range 500deg/s
	data = GYRO_RANGE_500DPS;
	HAL_I2C_Mem_Write(hi2c, MPU6050_ADDR, REG_GYRO_CONFIG, 1, &data, 1, 100);

	vTaskDelay(pdMS_TO_TICKS(1500));

	calibrate();

	last_update_ticks = xTaskGetTickCount();
	return true;
}

void IMUSensor::calibrate() {
	int32_t total_z = 0;
	const uint32_t samples = 500;
	uint32_t valid_samples = 0;

	for (uint16_t i = 0; i < samples; i++) {
		uint8_t raw_data[2] = { 0, 0 };
		if (HAL_I2C_Mem_Read(hi2c, MPU6050_ADDR, REG_GYRO_Z_HIGH_BYTE, 1,
				raw_data, 2, 10) == HAL_OK) {
			int16_t raw_z = (int16_t) ((raw_data[0] << 8) | raw_data[1]);
			total_z += raw_z;
			valid_samples++;
		}
		vTaskDelay(pdMS_TO_TICKS(2));
	}

	if (valid_samples > 0) {
		gyro_zero_rate_bias = static_cast<float>(total_z)
				/ static_cast<float>(valid_samples);
		printf("Calibration complete. Offset = %f\r\n", gyro_zero_rate_bias);
	} else {
		printf("Calibration failed.\r\n");
		gyro_zero_rate_bias = 0.0f;
	}
}

void IMUSensor::update() {
	uint8_t raw_data[2];

	HAL_StatusTypeDef status = HAL_I2C_Mem_Read(hi2c, MPU6050_ADDR,
			REG_GYRO_Z_HIGH_BYTE, 1, raw_data, 2, 10);

	int16_t raw_z = (int16_t) (raw_data[0] << 8) | raw_data[1];

	static int debug_counter = 0;
	if (debug_counter++ % 100 == 0) {
		if (status != HAL_OK) {
			printf("I2C ERROR CODE: %d\r\n", status);
		} else {
			int16_t raw_z = (int16_t) ((raw_data[0] << 8) | raw_data[1]);
			printf("Raw: %d | Offset: %.1f | Angle: %.1f\r\n", raw_z,
					gyro_zero_rate_bias, current_yaw_deg);
		}
	}

	if (status != HAL_OK) {
		return;
	}

	float rate_z = (static_cast<float>(raw_z) - gyro_zero_rate_bias)
			/ GYRO_SENSITIVITY_500DPS;

//	if (fabs(rate_z) < 0.1f) {
//		rate_z = 0.0f;
//	}

	uint32_t current_ticks = xTaskGetTickCount();
	float dt = static_cast<float>(current_ticks - last_update_ticks)
			/ static_cast<float>(configTICK_RATE_HZ);
	last_update_ticks = current_ticks;

	current_yaw_deg += (rate_z * dt);

	if (current_yaw_deg >= 360.0f) {
		current_yaw_deg -= 360.0f;
	} else if (current_yaw_deg < 0.0f) {
		current_yaw_deg += 360.0f;
	}
}

void IMUSensor::resetYaw() {
	current_yaw_deg = 0.0f;
	last_update_ticks = xTaskGetTickCount();
}

float IMUSensor::getYawDegrees() const {
	return current_yaw_deg;
}
