#include <gtest/gtest.h>
#include "bridge/beep_generator.h"
#include <cstdint>
#include <vector>

TEST(BeepGenerator, samples_within_s16_range) {
    // Arrange
    BeepGenerator beep;
    std::vector<int16_t> buf(beep.cycle_samples());

    // Act
    beep.fill_frame(buf.data(), buf.size());

    // Assert
    for (auto sample : buf) {
        EXPECT_GE(sample, INT16_MIN);
        EXPECT_LE(sample, INT16_MAX);
    }
}

TEST(BeepGenerator, on_period_has_correct_sample_count) {
    // Arrange
    BeepGenerator beep(400, 200, 200, 8000);

    // Act / Assert
    EXPECT_EQ(beep.on_samples(), 1600u);
    EXPECT_EQ(beep.off_samples(), 1600u);
    EXPECT_EQ(beep.cycle_samples(), 3200u);
}

TEST(BeepGenerator, tone_on_phase_nonzero) {
    // Arrange
    BeepGenerator beep(400, 200, 200, 8000, 16000);
    unsigned int on = beep.on_samples();
    std::vector<int16_t> buf(on);

    // Act
    beep.fill_frame(buf.data(), on);

    // Assert: at least some samples should be nonzero during tone-on
    bool has_nonzero = false;
    for (auto s : buf) {
        if (s != 0) { has_nonzero = true; break; }
    }
    EXPECT_TRUE(has_nonzero);
}

TEST(BeepGenerator, tone_off_phase_zero) {
    // Arrange
    BeepGenerator beep(400, 200, 200, 8000, 16000);
    unsigned int on = beep.on_samples();
    unsigned int off = beep.off_samples();
    std::vector<int16_t> skip_buf(on);
    std::vector<int16_t> off_buf(off);

    // Act: advance past the tone-on phase
    beep.fill_frame(skip_buf.data(), on);
    beep.fill_frame(off_buf.data(), off);

    // Assert: all samples in off phase should be zero
    for (auto s : off_buf) {
        EXPECT_EQ(s, 0);
    }
}

TEST(BeepGenerator, reset_restarts_pattern) {
    // Arrange
    BeepGenerator beep(400, 200, 200, 8000, 16000);
    std::vector<int16_t> first(160);
    std::vector<int16_t> second(160);

    // Act
    beep.fill_frame(first.data(), 160);
    beep.reset();
    beep.fill_frame(second.data(), 160);

    // Assert
    for (size_t i = 0; i < 160; ++i) {
        EXPECT_EQ(first[i], second[i]);
    }
}

TEST(BeepGenerator, pattern_repeats_across_cycles) {
    // Arrange
    BeepGenerator beep(400, 200, 200, 8000, 16000);
    size_t cycle = beep.cycle_samples();
    std::vector<int16_t> first_cycle(cycle);
    std::vector<int16_t> second_cycle(cycle);

    // Act
    beep.fill_frame(first_cycle.data(), cycle);
    beep.fill_frame(second_cycle.data(), cycle);

    // Assert
    for (size_t i = 0; i < cycle; ++i) {
        EXPECT_EQ(first_cycle[i], second_cycle[i]);
    }
}
