#pragma once

#include <cstdint>
#include <optional>
#include <string>

struct DeviceInfo {
    std::string serial_port;
    std::string alsa_device;
};

constexpr uint16_t EC20_VENDOR_ID  = 0x2C7C;
constexpr uint16_t EC20_PRODUCT_ID = 0x0125;

std::optional<DeviceInfo> discover_ec20();
