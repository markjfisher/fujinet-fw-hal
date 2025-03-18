use std::ffi::c_void;

// Opaque types for C
pub type FujiDevice = c_void;
pub type FujiPlatform = c_void;
pub type FujiHostTranslator = c_void;

// FujiNet error codes
pub const FN_ERR_OK: u8 = 0x00;      /* No error */
pub const FN_ERR_IO_ERROR: u8 = 0x01; /* There was IO error/issue with the device */
pub const FN_ERR_BAD_CMD: u8 = 0x02;  /* Function called with bad arguments */
pub const FN_ERR_OFFLINE: u8 = 0x03;  /* The device is offline */
pub const FN_ERR_WARNING: u8 = 0x04;  /* Device specific non-fatal warning issued */
pub const FN_ERR_NO_DEVICE: u8 = 0x05; /* There is no network device */
pub const FN_ERR_UNKNOWN: u8 = 0xff;   /* Device specific error we didn't handle */
