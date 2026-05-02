cmake_minimum_required(VERSION 3.14)
project(audio-echo VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(PkgConfig REQUIRED)
pkg_check_modules(ALSA REQUIRED alsa)

add_executable(audio-echo
    src/main.cpp
    src/device_discovery.cpp
    src/serial_port.cpp
    src/at_commander.cpp
    src/audio_loop.cpp
)

target_include_directories(audio-echo PRIVATE
    src
    ${ALSA_INCLUDE_DIRS}
)

target_link_libraries(audio-echo PRIVATE
    ${ALSA_LIBRARIES}
    pthread
)

target_compile_options(audio-echo PRIVATE
    -Wall -Wextra -Wpedantic -Werror
)

enable_testing()

find_package(GTest QUIET)
if(NOT GTest_FOUND)
    include(FetchContent)
    FetchContent_Declare(
        googletest
        GIT_REPOSITORY https://github.com/google/googletest.git
        GIT_TAG v1.15.2
    )
    set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
    FetchContent_MakeAvailable(googletest)
    set(GTest_FOUND TRUE)
endif()

if(GTest_FOUND)
    add_executable(audio-echo-tests
        tests/integration/test_device_discovery.cpp
        tests/integration/test_serial_port.cpp
        tests/integration/test_at_commander.cpp
        tests/integration/test_audio_loop.cpp
        tests/integration/test_end_to_end.cpp
        src/device_discovery.cpp
        src/serial_port.cpp
        src/at_commander.cpp
        src/audio_loop.cpp
    )

    target_include_directories(audio-echo-tests PRIVATE
        src
        tests/integration
        ${ALSA_INCLUDE_DIRS}
    )

    target_link_libraries(audio-echo-tests PRIVATE
        GTest::gtest GTest::gtest_main
        ${ALSA_LIBRARIES}
        pthread
        util
    )

    target_compile_options(audio-echo-tests PRIVATE
        -Wall -Wextra -Wpedantic
    )

    include(GoogleTest)
    gtest_discover_tests(audio-echo-tests)
else()
    message(WARNING "GTest not found, tests will not be built")
endif()
