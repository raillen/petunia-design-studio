add_library(petunia_core INTERFACE)
target_include_directories(petunia_core INTERFACE
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<INSTALL_INTERFACE:include>)
target_compile_features(petunia_core INTERFACE cxx_std_23)

add_library(petunia_core_warnings INTERFACE)
if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
  target_compile_options(petunia_core_warnings INTERFACE -Wall -Wextra -Werror -Wpedantic)
endif()
