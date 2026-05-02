#include "serial_port.h"
#include "pty_pair.h"

#include <gtest/gtest.h>
#include <thread>
#include <chrono>
#include <cstring>
#include <unistd.h>

class SerialPortTest : public ::testing::Test {
protected:
    PtyPair pty;

    void SetUp() override {
        ASSERT_TRUE(pty.create()) << "failed to create pty pair";
    }
};

TEST_F(SerialPortTest, open_and_close) {
    // Arrange
    SerialPort port;

    // Act
    bool opened = port.open(pty.slave_name);

    // Assert
    ASSERT_TRUE(opened);
    EXPECT_TRUE(port.is_open());
    port.close();
    EXPECT_FALSE(port.is_open());
}

TEST_F(SerialPortTest, write_and_read_via_pty) {
    // Arrange
    SerialPort port;
    ASSERT_TRUE(port.open(pty.slave_name));

    // Act: write through the master side, read from SerialPort
    const char* msg = "HELLO\r\n";
    ssize_t n = ::write(pty.master_fd, msg, std::strlen(msg));
    ASSERT_GT(n, 0);

    std::this_thread::sleep_for(std::chrono::milliseconds(50));
    auto line = port.read_line();

    // Assert
    ASSERT_TRUE(line.has_value());
    EXPECT_EQ(*line, "HELLO");
}

TEST_F(SerialPortTest, write_line_sends_to_master) {
    // Arrange
    SerialPort port;
    ASSERT_TRUE(port.open(pty.slave_name));

    // Act
    ASSERT_TRUE(port.write_line("AT"));
    std::this_thread::sleep_for(std::chrono::milliseconds(50));

    char buf[64]{};
    ssize_t n = ::read(pty.master_fd, buf, sizeof(buf) - 1);

    // Assert
    ASSERT_GT(n, 0);
    EXPECT_NE(std::string(buf).find("AT"), std::string::npos);
}

TEST_F(SerialPortTest, read_returns_nullopt_on_timeout) {
    // Arrange
    SerialPort port;
    ASSERT_TRUE(port.open(pty.slave_name));

    // Act
    auto line = port.read_line();

    // Assert
    EXPECT_FALSE(line.has_value());
}

TEST_F(SerialPortTest, open_invalid_path_fails) {
    // Arrange
    SerialPort port;

    // Act
    bool opened = port.open("/dev/nonexistent_tty_device_xyz");

    // Assert
    EXPECT_FALSE(opened);
    EXPECT_FALSE(port.is_open());
}

TEST_F(SerialPortTest, move_semantics) {
    // Arrange
    SerialPort port;
    ASSERT_TRUE(port.open(pty.slave_name));

    // Act
    SerialPort moved = std::move(port);

    // Assert
    EXPECT_FALSE(port.is_open());
    EXPECT_TRUE(moved.is_open());
}
