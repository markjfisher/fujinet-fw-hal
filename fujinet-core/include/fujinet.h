#ifndef FUJINET_H
#define FUJINET_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque types
typedef struct FujiDevice FujiDevice;
typedef struct FujiPlatform FujiPlatform;
typedef struct FujiHostTranslator FujiHostTranslator;

// Error codes
typedef enum {
    FUJI_ERROR_OK = 0,
    FUJI_ERROR_IO = 1,
    FUJI_ERROR_NOT_READY = 2,
    FUJI_ERROR_NOT_SUPPORTED = 3,
    FUJI_ERROR_INVALID_PARAMETER = 4,
    FUJI_ERROR_CONNECTION = 5,
} FujiError;

// Device functions
FujiError fuji_device_open(FujiDevice* device);
FujiError fuji_device_close(FujiDevice* device);
FujiError fuji_device_read_bytes(FujiDevice* device, uint8_t* buffer, size_t len, size_t* bytes_read);
FujiError fuji_device_write_bytes(FujiDevice* device, const uint8_t* buffer, size_t len, size_t* bytes_written);

// Platform functions
FujiError fuji_platform_initialize(FujiPlatform* platform);
FujiError fuji_platform_shutdown(FujiPlatform* platform);

// Host Translator functions
FujiError fuji_host_translator_initialize(FujiHostTranslator* translator);
FujiError fuji_host_translator_process_host_data(
    FujiHostTranslator* translator,
    const uint8_t* data,
    size_t len,
    uint8_t** output,
    size_t* output_len
);

#ifdef __cplusplus
}
#endif

#endif // FUJINET_H 