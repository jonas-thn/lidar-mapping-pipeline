#include "LidarParser.hpp"
#include <string.h>

bool LidarParser::processByte(uint8_t byte) {
	switch (state) {
	case ParserState::SYNC1:
		if (byte == 0x55) {
			raw_buffer[0] = byte;
			state = ParserState::SYNC2;
		}
		break;

	case ParserState::SYNC2:
		if (byte == 0xAA) {
			raw_buffer[1] = byte;
			byte_counter = 2;
			state = ParserState::COLLECT;
		} else if (byte == 0x55) {
			raw_buffer[0] = byte;
		} else {
			state = ParserState::SYNC1;
		}
		break;

	case ParserState::COLLECT:
		raw_buffer[byte_counter++] = byte;

		if (byte_counter == 60) {
			memcpy(&current_packet, raw_buffer, 60);

			for (int i = 0; i < 12; i++) {
				uint16_t raw_distance = current_packet.points[i].distance_mm;

				if (raw_distance & 0x8000) {
					current_packet.points[i].distance_mm = 0;
				} else {
					current_packet.points[i].distance_mm = raw_distance
							& 0x3FFF;
				}
			}

			state = ParserState::SYNC1;
			return true;
		}
		break;
	}
	return false;
}

LidarPacket LidarParser::getPacket() {
	return current_packet;
}

