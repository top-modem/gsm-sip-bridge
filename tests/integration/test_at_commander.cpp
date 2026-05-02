#include "at_commander.h"
#include "pty_pair.h"

#include <gtest/gtest.h>
#include <cstring>
#include <thread>
#include <chrono>
#include <unistd.h>

class AtCommanderTest : public ::testing::Test {
protected:
    PtyPair pty;
    SerialPort app_port;

    void SetUp() override {
        ASSERT_TRUE(pty.create()) << "failed to create pty pair";
        ASSERT_TRUE(app_port.open(pty.slave_name));
    }

    void TearDown() override {
        app_port.close();
    }

    void sim_send(const std::string& line) {
        std::string data = line + "\r\n";
        ::write(pty.master_fd, data.c_str(), data.size());
    }

    std::string sim_read(int timeout_ms = 2000) {
        char buf[256]{};
        fd_set fds;
        FD_ZERO(&fds);
        FD_SET(pty.master_fd, &fds);
        struct timeval tv;
        tv.tv_sec = timeout_ms / 1000;
        tv.tv_usec = (timeout_ms % 1000) * 1000;

        if (select(pty.master_fd + 1, &fds, nullptr, nullptr, &tv) > 0) {
            ssize_t n = ::read(pty.master_fd, buf, sizeof(buf) - 1);
            if (n > 0) return std::string(buf, n);
        }
        return {};
    }
};

TEST_F(AtCommanderTest, send_and_expect_ok_succeeds) {
    // Arrange
    AtCommander at(app_port);

    // Act
    std::thread responder([this]() {
        std::string cmd = sim_read();
        EXPECT_NE(cmd.find("ATE0"), std::string::npos);
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        sim_send("OK");
    });

    bool result = at.send_and_expect_ok("ATE0", 3000);
    responder.join();

    // Assert
    EXPECT_TRUE(result);
}

TEST_F(AtCommanderTest, send_and_expect_ok_handles_error) {
    // Arrange
    AtCommander at(app_port);

    // Act
    std::thread responder([this]() {
        sim_read();
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        sim_send("ERROR");
    });

    bool result = at.send_and_expect_ok("AT+BADCMD", 3000);
    responder.join();

    // Assert
    EXPECT_FALSE(result);
}

TEST_F(AtCommanderTest, answer_call_sends_ata) {
    // Arrange
    AtCommander at(app_port);

    // Act
    std::thread responder([this]() {
        std::string cmd = sim_read();
        EXPECT_NE(cmd.find("ATA"), std::string::npos);
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        sim_send("OK");
    });

    bool result = at.answer_call();
    responder.join();

    // Assert
    EXPECT_TRUE(result);
}

TEST_F(AtCommanderTest, hangup_sends_chup) {
    // Arrange
    AtCommander at(app_port);

    // Act
    std::thread responder([this]() {
        std::string cmd = sim_read();
        EXPECT_NE(cmd.find("AT+CHUP"), std::string::npos);
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        sim_send("OK");
    });

    bool result = at.hangup();
    responder.join();

    // Assert
    EXPECT_TRUE(result);
}

TEST_F(AtCommanderTest, poll_urc_returns_ring) {
    // Arrange
    AtCommander at(app_port);

    // Act
    sim_send("RING");
    std::this_thread::sleep_for(std::chrono::milliseconds(50));

    std::optional<std::string> urc;
    for (int i = 0; i < 20 && !urc; ++i) {
        urc = at.poll_urc();
        if (!urc) std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }

    // Assert
    ASSERT_TRUE(urc.has_value());
    EXPECT_NE(urc->find("RING"), std::string::npos);
}

TEST_F(AtCommanderTest, query_network_registration_success) {
    // Arrange
    AtCommander at(app_port);

    // Act
    std::thread responder([this]() {
        sim_read();
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        sim_send("+COPS: 0,0,\"Test Operator\",7");
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        sim_send("OK");
    });

    bool registered = at.query_network_registration();
    responder.join();

    // Assert
    EXPECT_TRUE(registered);
}
