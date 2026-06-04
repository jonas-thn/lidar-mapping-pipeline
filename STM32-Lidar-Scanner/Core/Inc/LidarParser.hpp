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
    uint8_t  sync1;
    uint8_t  sync2;
    uint8_t  type;
    uint8_t  num_points;
    uint16_t speed;
    uint16_t start_angle;
    LidarPoint points[12];
    uint16_t end_angle;
    uint16_t crc16;
};

#pragma pack(pop)

enum class ParserState {
    SYNC1,
    SYNC2,
    COLLECT
};

class LidarParser {
private:
	ParserState state = ParserState::SYNC1;

	uint8_t raw_buffer[60];
	uint8_t byte_counter = 0;

	LidarPacket current_packet;

	uint16_t calculateCRC16(const uint8_t* data, uint16_t len);

public:
	bool processByte(uint8_t byte);
	LidarPacket getPacket();
};

#endif /* INC_LIDARPARSER_HPP_ */
