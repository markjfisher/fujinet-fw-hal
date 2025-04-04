#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "fujinet-network.h"

const char* httpbin = "N1:http://192.168.1.100:8085/";
char url_buffer[128];
char *url;
uint8_t response_buffer[4096];

void print_error(const char* operation, uint8_t result) {
    printf("Error in %s: code %d (", operation, result);
    switch(result) {
        case FN_ERR_IO_ERROR:
            printf("IO Error");
            break;
        case FN_ERR_BAD_CMD:
            printf("Bad Command/Arguments");
            break;
        case FN_ERR_OFFLINE:
            printf("Device Offline");
            break;
        case FN_ERR_NO_DEVICE:
            printf("No Device");
            break;
        case FN_ERR_UNKNOWN:
            printf("Unknown Error");
            break;
        default:
            printf("Undefined Error");
    }
    printf(")\n");
}

char *create_url(char *method) {
    sprintf(url_buffer, "%s%s", httpbin, method);
    return (char *) url_buffer;
}

// void print_response() {
//     int16_t bytes_read = network_http_get(url, response_buffer, sizeof(response_buffer));
//     if (bytes_read < 0) {
//         print_error("network_http_get", -bytes_read);
//         return;
//     }
//     printf("HTTP GET successful, received %d bytes\n", bytes_read);
//     printf("Response:\n%s\n", response_buffer);
// }

int8_t do_open() {
    // Open the network
    int8_t result;

    printf("Opening network...\n");
    result = network_open(url, OPEN_MODE_HTTP_GET, OPEN_TRANS_NONE);
    if (result != FN_ERR_OK) {
        print_error("network_open", result);
        return 1;
    }
    printf("Network opened successfully\n");
    return result;
}

int main() {
    uint8_t result;
    int16_t bytes_read;
    const char* post_url;
    const char* post_data;
    uint8_t binary_data[] = {0x01, 0x02, 0x03, 0x04};
    const char* delete_url;

    printf("Starting HTTP test...\n");
    printf("Using endpoint: %s\n", httpbin);
    
    // Initialize the network
    printf("Initializing network...\n");
    result = network_init();
    if (result != FN_ERR_OK) {
        print_error("network_init", result);
        return 1;
    }
    printf("Network initialized successfully\n");

    // Open the network
    url = create_url("get?a=1&b=2");
    result = do_open(url);
    if (result != FN_ERR_OK) {
        print_error("open", result);
        return 1;
    }

    // Test HTTP GET
    printf("Performing HTTP GET to %s...\n", url);
    bytes_read = network_http_get(url, response_buffer, sizeof(response_buffer));
    if (bytes_read < 0) {
        print_error("network_http_get", -bytes_read);
        return 1;
    }
    printf("HTTP GET successful, received %d bytes\n", bytes_read);
    printf("Response:\n%s\n", response_buffer);

    // Close the network
    result = network_close(url);
    if (result != FN_ERR_OK) {
        print_error("network_close", result);
        return 1;
    }
    printf("Network closed successfully\n");

    // // Test HTTP POST
    // post_url = create_url("post");
    // post_data = "{\"test\": \"data\"}";
    // printf("Performing HTTP POST to %s with data: %s\n", post_url, post_data);
    // result = network_http_post(post_url, post_data);
    // if (result != FN_ERR_OK) {
    //     print_error("network_http_post", result);
    //     return 1;
    // }
    // printf("HTTP POST successful\n");

    // // Test HTTP POST with binary data
    // printf("Performing HTTP POST with binary data to %s\n", post_url);
    // result = network_http_post_bin(post_url, binary_data, sizeof(binary_data));
    // if (result != FN_ERR_OK) {
    //     print_error("network_http_post_bin", result);
    //     return 1;
    // }
    // printf("HTTP POST binary successful\n");

    // // Test HTTP DELETE
    // delete_url = create_url("delete");
    // printf("Performing HTTP DELETE to %s\n", delete_url);
    // result = network_http_delete(delete_url, OPEN_TRANS_NONE);
    // if (result != FN_ERR_OK) {
    //     print_error("network_http_delete", result);
    //     return 1;
    // }
    // printf("HTTP DELETE successful\n");

    printf("All tests completed successfully!\n");
    return 0;
} 