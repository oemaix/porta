cmake_minimum_required(VERSION 3.16)

set(CSPOT_ROOT ${CMAKE_CURRENT_LIST_DIR}/../../third_party/cspot/cspot)
if(NOT EXISTS ${CSPOT_ROOT}/CMakeLists.txt)
    message(FATAL_ERROR "cspot is missing at ${CSPOT_ROOT}")
endif()

file(READ "${CSPOT_ROOT}/bell/external/civetweb/civetweb.c" _porta_civet)
if(_porta_civet MATCHES "freeRTOSTaskEntryFunc, \"civetweb\", thread_id.stack_size, &thread_id")
    message(FATAL_ERROR "civetweb still passes a stack local to its FreeRTOS thread (P-0067)")
endif()
if(_porta_civet MATCHES "HTTP_ESP_STACK, MALLOC_CAP_SPIRAM")
    message(FATAL_ERROR "civetweb stack is still in PSRAM (P-0067)")
endif()
file(READ "${CSPOT_ROOT}/bell/main/io/BellHTTPServer.cpp" _porta_http)
if(NOT _porta_http MATCHES "num_threads")
    message(FATAL_ERROR "civetweb still uses the default worker count (P-0067)")
endif()

set(BELL_ONLY_CJSON ON CACHE BOOL "" FORCE)
set(BELL_DISABLE_SINKS ON CACHE BOOL "" FORCE)
set(BELL_DISABLE_MQTT ON CACHE BOOL "" FORCE)
set(BELL_DISABLE_AVAHI ON CACHE BOOL "" FORCE)
set(BELL_CODEC_AAC ON CACHE BOOL "" FORCE)
set(BELL_CODEC_MP3 ON CACHE BOOL "" FORCE)
set(BELL_CODEC_OPUS OFF CACHE BOOL "" FORCE)
set(BELL_CODEC_ALAC OFF CACHE BOOL "" FORCE)
set(BELL_CODEC_VORBIS ON CACHE BOOL "" FORCE)

# Nanopb's plugin follows PATH. The IDF venv is first and has no
# google.protobuf. Use the first interpreter on PATH that imports it.
string(REPLACE ":" ";" _porta_path "$ENV{PATH}")
set(PORTA_NANOPB_PYTHON "")
foreach(_dir ${_porta_path})
    if(EXISTS "${_dir}/python3")
        execute_process(
            COMMAND "${_dir}/python3" -c "import google.protobuf"
            RESULT_VARIABLE _porta_py_rc
            OUTPUT_QUIET
            ERROR_QUIET
        )
        if(_porta_py_rc EQUAL 0)
            set(PORTA_NANOPB_PYTHON "${_dir}/python3")
            break()
        endif()
    endif()
endforeach()
if(PORTA_NANOPB_PYTHON STREQUAL "")
    message(FATAL_ERROR "cspot nanopb needs a python3 that imports google.protobuf")
endif()
file(WRITE "${CSPOT_ROOT}/bell/external/nanopb/generator/protoc-gen-nanopb"
"#!/bin/sh
MYPATH=$(dirname \"$0\")
exec ${PORTA_NANOPB_PYTHON} \"$MYPATH/nanopb_generator.py\" --protoc-plugin
")

idf_component_register(
    SRCS porta_connect.cpp wchar_stubs.c
    INCLUDE_DIRS "."
    REQUIRES espressif__mdns mbedtls pthread lwip esp_wifi
)

add_subdirectory(${CSPOT_ROOT} ${CMAKE_CURRENT_BINARY_DIR}/cspot_lib)
target_link_libraries(${COMPONENT_LIB} PUBLIC cspot)
set_source_files_properties(porta_connect.cpp PROPERTIES COMPILE_OPTIONS "-std=gnu++20")
