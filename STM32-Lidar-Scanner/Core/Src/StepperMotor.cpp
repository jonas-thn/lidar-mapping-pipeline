#include "StepperMotor.hpp"

StepperMotor::StepperMotor(TIM_HandleTypeDef *htim, uint32_t channel,
		GPIO_TypeDef *dir_port, uint16_t dir_pin) :
		htim(htim), channel(channel), dir_port(dir_port), dir_pin(dir_pin) {

}

void StepperMotor::start() {
	HAL_TIM_PWM_Start(htim, channel);
}

void StepperMotor::stop() {
	HAL_TIM_PWM_Stop(htim, channel);
}

void StepperMotor::setDirection(bool clockwise) {
	HAL_GPIO_WritePin(dir_port, dir_pin,
			clockwise ? GPIO_PIN_SET : GPIO_PIN_RESET);
}

void StepperMotor::setSpeedHz(uint32_t frequency) {
	if (frequency < 10) {
		stop();
		return;
	}

	uint32_t arr_value = (TIMER_CLK / frequency) - 1;
	if (arr_value > 65535) {
		arr_value = 65535;
	}

	uint32_t ccr_value = arr_value / 2; //50% duty cycle

	htim->Instance->ARR = arr_value;

	htim->Instance->CCR1 = ccr_value;
}

