cmake_minimum_required(VERSION 3.0)

project(PISA CXX C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE "Release")
endif()
MESSAGE( STATUS "CMAKE_BUILD_TYPE: " ${CMAKE_BUILD_TYPE} )

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

if (UNIX)
   # For hardware popcount and other special instructions
   set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -march=native")
   # Extensive warnings
   set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -Wno-missing-braces")
endif()

find_package(OpenMP)
if(OPENMP_FOUND)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${OpenMP_CXX_FLAGS}")
endif()
set(THREADS_PREFER_PTHREAD_FLAG ON)
find_package(Threads REQUIRED)

EXECUTE_PROCESS(COMMAND git submodule update --init
                WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}/..
                OUTPUT_QUIET
        )

# Add PISA
set(PISA_BUILD_TOOLS OFF CACHE BOOL "skip building")
set(PISA_ENABLE_TESTING OFF CACHE BOOL "skip testing")
set(PISA_ENABLE_BENCHMARKING OFF CACHE BOOL "skip benchmarking")
add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/pisa EXCLUDE_FROM_ALL)

add_executable(build_index tools/build_index.cpp)
target_link_libraries(build_index
    pisa
    nlohmann_json
)

add_executable(do_query tools/do_query.cpp)
target_link_libraries(do_query
  pisa
)
