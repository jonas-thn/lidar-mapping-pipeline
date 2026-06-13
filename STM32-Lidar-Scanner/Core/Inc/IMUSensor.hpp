#ifndef INC_IMUSENSOR_HPP_
#define INC_IMUSENSOR_HPP_

#include "main.h"

class IMUSensor {
public:
	IMUSensor(I2C_HandleTypeDef* hi2c);

	bool init();
	void update();

	void resetYaw();
	float getYawDegrees() const;

private:
	I2C_HandleTypeDef* hi2c;

	float current_yaw_deg;
	float gyro_zero_rate_bias;
	uint32_t last_update_ticks;

	void calibrate();
};

#endif /* INC_IMUSENSOR_HPP_ */
