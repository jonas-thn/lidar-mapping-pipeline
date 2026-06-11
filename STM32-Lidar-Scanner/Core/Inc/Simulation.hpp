#ifndef INC_SIMULATION_HPP_
#define INC_SIMULATION_HPP_

#include "FreeRTOS.h"
#include "task.h"

#ifdef __cplusplus
extern "C" {
#endif

void vMotorSimulationTask(void *pvParameters);
void vLidarSimulationTask(void *pvParameters);

#ifdef __cplusplus
}
#endif

#endif /* INC_SIMULATION_HPP_ */
