#include "app_main.h"
#include "main.h"
#include "usart.h"

const uint8_t MAX_POINTS = 10;
uint8_t dma_rx_buffer[MAX_POINTS * sizeof(LidarPoint)];

void app_main(void)
{
	HAL_UART_Receive_DMA(&huart1, dma_rx_buffer, sizeof(dma_rx_buffer));

	while(1)
	{
		HAL_GPIO_TogglePin(LD2_GPIO_Port, LD2_Pin);
		HAL_Delay(500);
	}
}




