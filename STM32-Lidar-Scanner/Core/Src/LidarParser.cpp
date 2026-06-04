#include "LidarParser.hpp"

bool LidarParser::processByte(uint8_t byte) {
	switch (state) {
	case ParserState::SYNC1:
		if (byte == 0x55) {
			state = ParserState::SYNC2;
		}
		break;

	case ParserState::SYNC2:
		if (byte == 0xAA) {
			state = ParserState::HEADER;
			byte_counter = 0;
		} else {
			state = ParserState::SYNC1;
		}
		break;

	case ParserState::HEADER:
		temp_buffer[byte_counter++] = byte;
		if (byte_counter == 6) {
			current_packet.unknown_byte = temp_buffer[0];
			current_packet.num_points = temp_buffer[1];
			current_packet.speed = (temp_buffer[3] << 8) | temp_buffer[2];
			current_packet.start_angle = (temp_buffer[5] << 8) | temp_buffer[4];

			if (current_packet.num_points != 12) {
				state = ParserState::SYNC1;
				break;
			}

			state = ParserState::PAYLOAD;
			byte_counter = 0;
		}
		break;

	case ParserState::PAYLOAD:
		temp_buffer[byte_counter++] = byte;
		if (byte_counter == 48) {
			uint8_t *dest = (uint8_t*) current_packet.points;

			for (int i = 0; i < 48; i++) {
				dest[i] = temp_buffer[i];
			}

			state = ParserState::FOOTER;
			byte_counter = 0;
		}
		break;

	case ParserState::FOOTER:
		temp_buffer[byte_counter++] = byte;

		if (byte_counter == 4) {
			current_packet.checksum = (temp_buffer[3] << 24)
					| (temp_buffer[2] << 16) | (temp_buffer[1] << 8)
					| temp_buffer[0];

			//TODO Chechsum Test

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

