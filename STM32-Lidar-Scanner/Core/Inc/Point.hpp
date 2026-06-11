#ifndef INC_POINT_HPP_
#define INC_POINT_HPP_

#include <stdint.h>

#pragma pack(push, 1)
struct Point3D {
	uint16_t sync_word;
	int16_t x_mm;
	int16_t y_mm;
	int16_t z_mm;
	uint16_t quality;
};
#pragma pack(pop)

#endif /* INC_POINT_HPP_ */
