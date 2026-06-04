#ifndef INC_LIDARPARSER_HPP_
#define INC_LIDARPARSER_HPP_

#include <stdint.h>
#include <stdbool.h>

#pragma pack(push, 1)

struct LidarPoint {
	uint16_t distance_mm;
	uint16_t quality;
};

struct LidarPacket {
	uint8_t unknown_byte;
	uint8_t num_points;
	uint16_t speed;
	uint16_t start_angle;
	LidarPoint points[12];
	uint32_t checksum;
};

#pragma pack(pop)

enum class ParserState {
	SYNC1, SYNC2, HEADER, PAYLOAD, FOOTER
};

class LidarParser {
private:
	ParserState state = ParserState::SYNC1;

	uint8_t temp_buffer[48];
	uint8_t byte_counter = 0;

	LidarPacket current_packet;

public:
	bool processByte(uint8_t byte);
	LidarPacket getPacket();
};

#endif /* INC_LIDARPARSER_HPP_ */
