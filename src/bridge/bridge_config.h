#pragma once

#include <cstdint>
#include <string>

struct BridgeConfig {
    std::string sip_destination;
    uint16_t sip_dial_timeout_sec = 30;

    struct LoadResult {
        bool ok = false;
        std::string error;
    };

    static LoadResult load(const std::string& path, BridgeConfig& out);
};
