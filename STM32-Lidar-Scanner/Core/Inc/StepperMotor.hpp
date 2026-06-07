#ifndef INC_STEPPERMOTOR_HPP_
#define INC_STEPPERMOTOR_HPP_

#include "main.h"

class StepperMotor {
private:
	TIM_HandleTypeDef *htim;
	uint32_t channel;
	GPIO_TypeDef *dir_port;
	uint16_t dir_pin;

	const uint32_t TIMER_CLK = 1000000;

public:
	StepperMotor(TIM_HandleTypeDef *htim, uint32_t channel,
			GPIO_TypeDef *dir_port, uint16_t dir_pin);

	void start();
	void stop();
	void setDirection(bool clockwise);
	void setSpeedHz(uint32_t frequency);
};

#endif /* INC_STEPPERMOTOR_HPP_ */
