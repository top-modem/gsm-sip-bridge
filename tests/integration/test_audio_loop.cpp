#include "audio_loop.h"

#include <gtest/gtest.h>
#include <atomic>
#include <chrono>
#include <thread>

TEST(AudioLoop, open_default_device) {
    // Arrange
    AudioLoop loop;

    // Act: try opening the default ALSA device
    // On CI without snd-pcmtest, this may fail -- that is acceptable
    bool opened = loop.open("default");

    // Assert
    if (opened) {
        auto cfg = loop.config();
        EXPECT_GT(cfg.sample_rate, 0u);
        EXPECT_GT(cfg.period_frames, 0u);
        EXPECT_GT(cfg.buffer_frames, 0u);
        EXPECT_EQ(cfg.channels, 1u);
        loop.close();
    } else {
        GTEST_SKIP() << "no ALSA device available, skipping audio test";
    }
}

TEST(AudioLoop, open_invalid_device_fails) {
    // Arrange
    AudioLoop loop;

    // Act
    bool opened = loop.open("hw:99,99");

    // Assert
    EXPECT_FALSE(opened);
}

TEST(AudioLoop, run_and_stop) {
    // Arrange
    AudioLoop loop;
    if (!loop.open("default")) {
        GTEST_SKIP() << "no ALSA device available";
    }

    // Act: run for a short burst then stop
    std::atomic<bool> running{true};
    std::thread runner([&loop, &running]() {
        loop.run(running);
    });

    std::this_thread::sleep_for(std::chrono::milliseconds(200));
    running.store(false);
    runner.join();

    // Assert: no crash, clean shutdown
    loop.close();
    SUCCEED();
}

TEST(AudioLoop, close_is_idempotent) {
    // Arrange
    AudioLoop loop;

    // Act & Assert: calling close on unopened loop does not crash
    loop.close();
    loop.close();
    SUCCEED();
}
