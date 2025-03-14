#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "fujinet-network.h"

const char* httpbin = "N1:http://localhost:8085/";
char url_buffer[128];
uint8_t response_buffer[4096];

void print_error(const char* operation, uint8_t result) {
    printf("Error in %s: code %d\n", operation, result);
}

char *create_url(char *method) {
    sprintf(url_buffer, "%s%s", httpbin, method);
    return (char *) url_buffer;
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
    const char* url = create_url("get");
    printf("Performing HTTP GET to %s...\n", url);
    int16_t bytes_read = network_http_get(url, response_buffer, sizeof(response_buffer));
    if (bytes_read < 0) {
        print_error("network_http_get", -bytes_read);
        return 1;
    }
    printf("HTTP GET successful, received %d bytes\n", bytes_read);
    printf("Response:\n%s\n", response_buffer);

    // Test HTTP POST
    const char* post_url = create_url("post");
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
    const char* delete_url = create_url("delete");
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