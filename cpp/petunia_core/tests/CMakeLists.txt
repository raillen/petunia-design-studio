add_executable(core_smoke core_smoke.cpp)
target_link_libraries(core_smoke PRIVATE petunia_core petunia_core_warnings)
add_test(NAME core_smoke COMMAND core_smoke)
