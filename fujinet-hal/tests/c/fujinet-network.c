#include <stdio.h>
#include <string.h>
#include "fujinet-network.h"

// Global state
uint16_t fn_bytes_read = 0;
uint8_t fn_device_error = 0;
uint16_t fn_network_bw = 0;
uint8_t fn_network_conn = 0;
uint8_t fn_network_error = 0;

uint8_t fn_error(uint8_t code) {
    fn_device_error = code;
    return code;
}

uint8_t network_init(void) {
    return FN_ERR_OK;
}

uint8_t network_open(const char* devicespec, uint8_t mode, uint8_t trans) {
    if (!devicespec || strlen(devicespec) < 3) {
        return FN_ERR_BAD_CMD;
    }

    // Basic validation of N: prefix
    if (devicespec[0] != 'N' && devicespec[0] != 'n') {
        return FN_ERR_BAD_CMD;
    }

    // Validate unit number (1-8)
    if (devicespec[1] < '1' || devicespec[1] > '8') {
        return FN_ERR_BAD_CMD;
    }

    // Must have colon after unit
    if (devicespec[2] != ':') {
        return FN_ERR_BAD_CMD;
    }

    // Must have http:// or https:// after N1:
    const char* proto = devicespec + 3;
    if (strncmp(proto, "http://", 7) != 0 && strncmp(proto, "https://", 8) != 0) {
        return FN_ERR_BAD_CMD;
    }

    fn_network_conn = 1;  // Mark as connected
    return FN_ERR_OK;
}

int16_t network_http_get(const char* devicespec, uint8_t *buf, uint16_t len) {
    if (!fn_network_conn) {
        return -FN_ERR_OFFLINE;
    }

    // Simulate a response
    const char* response = "{\"args\":{\"a\":\"1\",\"b\":\"2\"},\"headers\":{\"Host\":\"192.168.1.100:8085\"},\"origin\":\"192.168.1.100\"}";
    size_t response_len = strlen(response);
    
    if (response_len > len) {
        return -FN_ERR_IO_ERROR;
    }

    memcpy(buf, response, response_len);
    fn_bytes_read = response_len;
    return response_len;
}

uint8_t network_close(const char* devicespec) {
    fn_network_conn = 0;
    return FN_ERR_OK;
} 