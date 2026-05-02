#include "device_discovery.h"

#include <gtest/gtest.h>

TEST(DeviceDiscovery, discover_returns_optional) {
    // Arrange: no specific setup needed; tests against real udev subsystem
    // Act
    auto result = discover_ec20();

    // Assert: either finds an EC20 or returns nullopt (both valid on CI)
    if (result) {
        EXPECT_FALSE(result->serial_port.empty());
        EXPECT_FALSE(result->alsa_device.empty());
        EXPECT_NE(result->alsa_device.find("hw:"), std::string::npos);
    } else {
        SUCCEED() << "no EC20 connected, discovery correctly returned nullopt";
    }
}

TEST(DeviceDiscovery, device_info_fields_are_non_empty_when_found) {
    // Arrange
    auto result = discover_ec20();
    if (!result) {
        GTEST_SKIP() << "EC20 not connected, skipping hardware-dependent test";
    }

    // Assert
    EXPECT_TRUE(result->serial_port.find("/dev/") == 0);
    EXPECT_TRUE(result->alsa_device.find("hw:") == 0);
}
