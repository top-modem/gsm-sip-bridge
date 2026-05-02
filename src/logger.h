#pragma once

#include <chrono>
#include <cstdarg>
#include <cstdio>
#include <ctime>
#include <iomanip>
#include <sstream>

enum class LogLevel { INFO, WARN, ERROR };

inline const char* level_str(LogLevel level) {
    switch (level) {
        case LogLevel::INFO:  return "INFO";
        case LogLevel::WARN:  return "WARN";
        case LogLevel::ERROR: return "ERROR";
    }
    return "UNKNOWN";
}

inline std::string timestamp_now() {
    auto now = std::chrono::system_clock::now();
    auto time_t_now = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
        now.time_since_epoch()) % 1000;

    std::tm tm_buf{};
    localtime_r(&time_t_now, &tm_buf);

    std::ostringstream oss;
    oss << std::put_time(&tm_buf, "%Y-%m-%dT%H:%M:%S")
        << '.' << std::setfill('0') << std::setw(3) << ms.count();
    return oss.str();
}

inline void log_msg(LogLevel level, const char* fmt, ...) __attribute__((format(printf, 2, 3)));

inline void log_msg(LogLevel level, const char* fmt, ...) {
    std::string ts = timestamp_now();
    std::fprintf(stdout, "%s %s ", ts.c_str(), level_str(level));

    va_list args;
    va_start(args, fmt);
    std::vfprintf(stdout, fmt, args);
    va_end(args);

    std::fputc('\n', stdout);
    std::fflush(stdout);
}

#define LOG_INFO(fmt, ...)  log_msg(LogLevel::INFO,  fmt, ##__VA_ARGS__)
#define LOG_WARN(fmt, ...)  log_msg(LogLevel::WARN,  fmt, ##__VA_ARGS__)
#define LOG_ERROR(fmt, ...) log_msg(LogLevel::ERROR, fmt, ##__VA_ARGS__)
