# fujinet-fw-hal

A green field rewrite of fujinet-firmware.

This is an attempt to break fujinet firmware up into a hardware abstraction, with
cleanly separated platform implementations (x86, esp32, raspberry pi etc).

Initially concentrating on the Network device, with http protocol, it will support compiling
down to an x86 library to be included with fujinet host applications originally targetting cc65
but using its C compatibility to allow compilation on gcc, using this library as the fujinet functionality.

Thus the core concepts are 100% compatible with Fujinet: "a multi-peripheral emulator and WiFi network device for vintage computers", but also targetting modern architectures.
