#include "bridge/beep_generator.h"
#include <cmath>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

BeepGenerator::BeepGenerator(unsigned int frequency_hz,
                             unsigned int on_ms,
                             unsigned int off_ms,
                             unsigned int sample_rate,
                             int16_t amplitude) {
    on_samples_ = sample_rate * on_ms / 1000;
    off_samples_ = sample_rate * off_ms / 1000;

    unsigned int cycle = on_samples_ + off_samples_;
    tone_buffer_.resize(cycle);

    double phase_increment = 2.0 * M_PI * frequency_hz / sample_rate;

    for (unsigned int i = 0; i < on_samples_; ++i) {
        tone_buffer_[i] = static_cast<int16_t>(
            amplitude * std::sin(phase_increment * i));
    }

    for (unsigned int i = on_samples_; i < cycle; ++i) {
        tone_buffer_[i] = 0;
    }
}

void BeepGenerator::fill_frame(int16_t* buf, size_t frame_count) {
    size_t cycle = tone_buffer_.size();
    for (size_t i = 0; i < frame_count; ++i) {
        buf[i] = tone_buffer_[position_ % cycle];
        ++position_;
    }
}

void BeepGenerator::reset() {
    position_ = 0;
}
