#ifndef INC_APP_MAIN_H_
#define INC_APP_MAIN_H_

#include <stdint.h>

#pragma pack(push, 1)
struct LidarPoint {
	uint16_t distance_mm;
	uint16_t quality;
};
#pragma pack(pop)

#ifdef __cplusplus
extern "C" {
#endif

void app_main(void);

#ifdef __cplusplus
}
#endif

#endif /* INC_APP_MAIN_H_ */
