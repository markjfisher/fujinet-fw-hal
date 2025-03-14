#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "fujinet-hal.h"

// Error codes
#define FN_ERR_OK 0
#define FN_ERR_BAD_CMD 2
#define FN_ERR_NO_DEVICE 5

// Transaction types
#define OPEN_TRANS_NONE 0

void print_error(const char* operation, uint8_t result) {
    printf("Error in %s: code %d\n", operation, result);
}

int main() {
    printf("Starting HTTP test...\n");
    
    // Initialize the network
    printf("Initializing network...\n");
    uint8_t result = network_init();
    if (result != FN_ERR_OK) {
        print_error("network_init", result);
        return 1;
    }
    printf("Network initialized successfully\n");

    // Test HTTP GET
    const char* url = "N1:https://httpbin.org/get";
    printf("Performing HTTP GET to %s...\n", url);
    result = network_http_get(url);
    if (result != FN_ERR_OK) {
        print_error("network_http_get", result);
        return 1;
    }
    printf("HTTP GET successful\n");

    // Test HTTP POST
    const char* post_url = "N1:https://httpbin.org/post";
    const char* post_data = "{\"test\": \"data\"}";
    printf("Performing HTTP POST to %s with data: %s\n", post_url, post_data);
    result = network_http_post(post_url, post_data);
    if (result != FN_ERR_OK) {
        print_error("network_http_post", result);
        return 1;
    }
    printf("HTTP POST successful\n");

    // Test HTTP POST with binary data
    uint8_t binary_data[] = {0x01, 0x02, 0x03, 0x04};
    printf("Performing HTTP POST with binary data to %s\n", post_url);
    result = network_http_post_bin(post_url, binary_data, sizeof(binary_data));
    if (result != FN_ERR_OK) {
        print_error("network_http_post_bin", result);
        return 1;
    }
    printf("HTTP POST binary successful\n");

    // Test HTTP DELETE
    const char* delete_url = "N1:https://httpbin.org/delete";
    printf("Performing HTTP DELETE to %s\n", delete_url);
    result = network_http_delete(delete_url, OPEN_TRANS_NONE);
    if (result != FN_ERR_OK) {
        print_error("network_http_delete", result);
        return 1;
    }
    printf("HTTP DELETE successful\n");

    printf("All tests completed successfully!\n");
    return 0;
} 