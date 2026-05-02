#include <gtest/gtest.h>
#include "ring_buffer.h"
#include <cstdint>
#include <thread>
#include <vector>

TEST(RingBuffer, write_then_read_returns_same_data) {
    // Arrange
    RingBuffer<int16_t> rb(256);
    int16_t write_data[] = {10, 20, 30, 40, 50};

    // Act
    bool written = rb.try_write(write_data, 5);
    int16_t read_data[5] = {};
    size_t read_count = rb.read(read_data, 5);

    // Assert
    EXPECT_TRUE(written);
    EXPECT_EQ(read_count, 5u);
    for (int i = 0; i < 5; ++i) {
        EXPECT_EQ(read_data[i], write_data[i]);
    }
}

TEST(RingBuffer, write_to_full_returns_false) {
    // Arrange
    RingBuffer<int16_t> rb(4);
    int16_t data[] = {1, 2, 3, 4};

    // Act
    bool first = rb.try_write(data, 4);
    bool second = rb.try_write(data, 1);

    // Assert
    EXPECT_TRUE(first);
    EXPECT_FALSE(second);
}

TEST(RingBuffer, read_from_empty_returns_zero) {
    // Arrange
    RingBuffer<int16_t> rb(64);
    int16_t buf[16] = {};

    // Act
    size_t count = rb.read(buf, 16);

    // Assert
    EXPECT_EQ(count, 0u);
}

TEST(RingBuffer, wraparound_works) {
    // Arrange
    RingBuffer<int16_t> rb(8);
    int16_t write1[] = {1, 2, 3, 4, 5, 6};
    int16_t read_buf[6] = {};

    // Act: fill most of the buffer, read it, then write across the wrap boundary
    rb.try_write(write1, 6);
    rb.read(read_buf, 6);

    int16_t write2[] = {10, 20, 30, 40, 50};
    bool ok = rb.try_write(write2, 5);
    int16_t result[5] = {};
    size_t count = rb.read(result, 5);

    // Assert
    EXPECT_TRUE(ok);
    EXPECT_EQ(count, 5u);
    for (int i = 0; i < 5; ++i) {
        EXPECT_EQ(result[i], write2[i]);
    }
}

TEST(RingBuffer, available_read_write_correct) {
    // Arrange
    RingBuffer<int16_t> rb(16);
    int16_t data[] = {1, 2, 3};

    // Act
    rb.try_write(data, 3);

    // Assert
    EXPECT_EQ(rb.available_read(), 3u);
    EXPECT_EQ(rb.available_write(), 13u);
}

TEST(RingBuffer, concurrent_producer_consumer) {
    // Arrange
    static constexpr size_t TOTAL_ITEMS = 100000;
    static constexpr size_t BATCH = 160;
    RingBuffer<int16_t> rb(4096);
    std::vector<int16_t> produced(TOTAL_ITEMS);
    std::vector<int16_t> consumed(TOTAL_ITEMS);

    for (size_t i = 0; i < TOTAL_ITEMS; ++i) {
        produced[i] = static_cast<int16_t>(i % 32768);
    }

    // Act
    std::thread producer([&]() {
        size_t offset = 0;
        while (offset < TOTAL_ITEMS) {
            size_t chunk = std::min(BATCH, TOTAL_ITEMS - offset);
            if (rb.try_write(&produced[offset], chunk)) {
                offset += chunk;
            } else {
                std::this_thread::yield();
            }
        }
    });

    std::thread consumer([&]() {
        size_t offset = 0;
        while (offset < TOTAL_ITEMS) {
            size_t chunk = std::min(BATCH, TOTAL_ITEMS - offset);
            size_t got = rb.read(&consumed[offset], chunk);
            offset += got;
            if (got == 0) std::this_thread::yield();
        }
    });

    producer.join();
    consumer.join();

    // Assert
    for (size_t i = 0; i < TOTAL_ITEMS; ++i) {
        EXPECT_EQ(consumed[i], produced[i]) << "Mismatch at index " << i;
    }
}
