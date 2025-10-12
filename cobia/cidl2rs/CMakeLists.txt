#(C) AmsterCHEM 2025
# note that this file is not typically used - it is used only for building the cidl2rs tool as stand-alone. Normally, the tool is built as part of the rust_cobia module build.
#TODO remove from repo

cmake_minimum_required(VERSION 3.10)

project (cidl2rs)

set(CMAKE_CXX_VISIBILITY_PRESET hidden)

if (${CMAKE_CXX_COMPILER_ID} STREQUAL GNU)
	if (MINGW) 
		set (CMAKE_SHARED_LINKER_FLAGS ${CMAKE_SHARED_LINKER_FLAGS} "-Wl,--kill-at")
	endif()
endif()

include_directories($ENV{COBIA_INCLUDE})

if ("${CMAKE_BUILD_TYPE}" STREQUAL Debug) 
	add_definitions(-DDEBUG)
	if (${CMAKE_SYSTEM_NAME} STREQUAL Linux) 
		SET(CMAKE_CXX_FLAGS_DEBUG "-g3")
	endif()
endif()

set(SOURCES cidl2rs.cpp)

if (${CMAKE_SYSTEM_NAME} STREQUAL Windows) 
	set(SOURCES ${SOURCES} cidl2rs.rc)
endif()

add_executable(cidl2rs ${SOURCES})
target_compile_features(cidl2rs PUBLIC cxx_std_20)
if (${CMAKE_SYSTEM_NAME} STREQUAL Linux) 
	add_definitions(-DPOSIX)
elseif (${CMAKE_SYSTEM_NAME} STREQUAL Windows) 
	add_definitions(-DUNICODE)
	if (${CMAKE_CXX_COMPILER_ID} STREQUAL MSVC)
		set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /permissive-")
	endif()
endif()


