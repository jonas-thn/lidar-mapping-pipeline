#include "app_main.h"
#include "main.h"
#include "usart.h"
#include "LidarParser.hpp"

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

