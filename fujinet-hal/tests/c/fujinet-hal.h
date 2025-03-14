#ifndef FUJINET_HAL_H
#define FUJINET_HAL_H

#include <stdint.h>

// Network functions
uint8_t network_init(void);
uint8_t network_http_get(const char* devicespec);
uint8_t network_http_post(const char* devicespec, const char* data);
uint8_t network_http_post_bin(const char* devicespec, const uint8_t* data, uint16_t len);
uint8_t network_http_delete(const char* devicespec, uint8_t trans);
uint8_t network_http_set_channel_mode(const char* devicespec, uint8_t mode);
uint8_t network_http_start_add_headers(const char* devicespec);
uint8_t network_http_end_add_headers(const char* devicespec);
uint8_t network_http_add_header(const char* devicespec, const char* header);

#endif // FUJINET_HAL_H 