nanobind_add_module(petunia_native)
target_sources(petunia_native PRIVATE src/petunia_native.cpp)
target_link_libraries(petunia_native PRIVATE petunia_core)
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
  target_compile_definitions(petunia_native PRIVATE NB_DEBUG)
endif()
set_target_properties(petunia_native PROPERTIES
  LIBRARY_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/python")
