cmake_minimum_required(VERSION 3.28)
project(petunia VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
  set(CMAKE_BUILD_TYPE RelWithDebInfo)
endif()

option(PETUNIA_BUILD_TESTS "Build tests" ON)
option(PETUNIA_ENABLE_SANITIZERS "Enable ASan/UBSan" OFF)

if(PETUNIA_ENABLE_SANITIZERS)
  add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer)
  add_link_options(-fsanitize=address,undefined)
endif()

add_subdirectory(cpp/petunia_core)

if(PETUNIA_BUILD_TESTS)
  enable_testing()
  add_subdirectory(cpp/petunia_core/tests)
endif()

find_package(Python COMPONENTS Interpreter Development REQUIRED)
find_package(nanobind CONFIG QUIET)
if(nanobind_FOUND)
  add_subdirectory(bindings/python)
else()
  message(STATUS "nanobind not found; skipping Python bindings target")
endif()
