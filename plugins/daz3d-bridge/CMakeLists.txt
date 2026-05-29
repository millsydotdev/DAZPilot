cmake_minimum_required(VERSION 3.15)
project(DazPilotBridge)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(WIN32)
    add_definitions(-DWIN32_LEAN_AND_MEAN -DNOMINMAX)
endif()

# Check DAZ_SDK_PATH (used by Rust sdk_indexer), then DAZ_SDK_ROOT, then default
if(DEFINED ENV{DAZ_SDK_PATH} AND NOT DEFINED DAZ_SDK_ROOT)
    set(DAZ_SDK_ROOT "$ENV{DAZ_SDK_PATH}")
elseif(DEFINED ENV{DAZ_SDK_ROOT} AND NOT DEFINED DAZ_SDK_ROOT)
    set(DAZ_SDK_ROOT "$ENV{DAZ_SDK_ROOT}")
elseif(NOT DEFINED DAZ_SDK_ROOT)
    set(DAZ_SDK_ROOT "${CMAKE_SOURCE_DIR}/../../thirdparty/DAZStudio4.5+ SDK" CACHE PATH "Daz Studio SDK root")
endif()
set(DAZ_SDK_INCLUDE "${DAZ_SDK_ROOT}/include")
if(WIN32)
    set(DAZ_SDK_PLATFORM "x64" CACHE STRING "Daz Studio SDK platform folder")
elseif(APPLE)
    set(DAZ_SDK_PLATFORM "mac64" CACHE STRING "Daz Studio SDK platform folder")
else()
    set(DAZ_SDK_PLATFORM "linux64" CACHE STRING "Daz Studio SDK platform folder")
endif()
set(DAZ_SDK_LIB "${DAZ_SDK_ROOT}/lib/${DAZ_SDK_PLATFORM}")
set(DAZ_SDK_BIN "${DAZ_SDK_ROOT}/bin/${DAZ_SDK_PLATFORM}")

# ── dzsdkmemory import target (required for VS 2019+ / DAZ 4.20+) ──────────
if(WIN32)
    if(MSVC_VERSION GREATER_EQUAL 1920)
        set(DAZ_SDK_MEMORY_LIB "${DAZ_SDK_LIB}/dzsdkmemory.lib")
        if(NOT EXISTS "${DAZ_SDK_MEMORY_LIB}")
            message(WARNING "dzsdkmemory.lib not found - plugin may not load on DAZ 4.20+ with VS 2019+")
        else()
            add_library(dzsdkmemory SHARED IMPORTED)
            set_property(TARGET dzsdkmemory APPEND PROPERTY IMPORTED_IMPLIB "${DAZ_SDK_MEMORY_LIB}")
            set_property(TARGET dzsdkmemory APPEND PROPERTY INTERFACE_INCLUDE_DIRECTORIES "${DAZ_SDK_INCLUDE}")
        endif()
    endif()
endif()

if(NOT EXISTS "${DAZ_SDK_INCLUDE}/dzplugin.h")
    message(FATAL_ERROR "Daz Studio SDK not found at '${DAZ_SDK_ROOT}'. Set DAZ_SDK_ROOT to the SDK root directory.")
endif()

include_directories(${DAZ_SDK_INCLUDE})
include_directories(
    "${CMAKE_SOURCE_DIR}"
    "${CMAKE_SOURCE_DIR}/../../thirdparty/doctest/doctest"
    "${CMAKE_SOURCE_DIR}/../../thirdparty/spdlog/include"
    "${CMAKE_SOURCE_DIR}/../../thirdparty/json/include"
    "${CMAKE_SOURCE_DIR}/../../thirdparty/glm"
    "${CMAKE_SOURCE_DIR}/../../thirdparty/stb"
    "${CMAKE_SOURCE_DIR}/../../thirdparty/simdjson/singleheader"
    "${CMAKE_SOURCE_DIR}/../../thirdparty/utfcpp/source"
)
# DzBridge Library
set(DZBRIDGE_ROOT "${CMAKE_SOURCE_DIR}/../../../temp_dazbridge" CACHE PATH "Path to DzBridge root")
include_directories("${DZBRIDGE_ROOT}/include")

# Pre-generated moc files (from Qt 4.8.1 bundled in Daz SDK).
# These are #included by the .cpp source files and must NOT be listed
# as target sources — doing so would cause duplicate symbol errors.

# ── Qt4 targets from Daz SDK ──────────────────────────────────────────────
# FindQt4.cmake was removed in CMake 3.28+, so we create targets manually
# using paths from the Daz SDK. Only modules actually linked are required.

function(_daz_add_qt_target name lib)
    if(WIN32)
        set(_lib "${DAZ_SDK_LIB}/${lib}.lib")
    else()
        set(_lib "${DAZ_SDK_LIB}/${CMAKE_STATIC_LIBRARY_PREFIX}${lib}${CMAKE_STATIC_LIBRARY_SUFFIX}")
    endif()
    if(EXISTS "${_lib}")
        add_library("Qt4::${name}" STATIC IMPORTED)
        set_target_properties("Qt4::${name}" PROPERTIES
            IMPORTED_LOCATION "${_lib}"
            INTERFACE_INCLUDE_DIRECTORIES "${DAZ_SDK_ROOT}/include/Qt${name}"
        )
    else()
        message(WARNING "Qt4::${name} not found at ${_lib}")
        add_library("Qt4::${name}" INTERFACE IMPORTED)
    endif()
endfunction()

_daz_add_qt_target(Core QtCore4)
_daz_add_qt_target(Gui QtGui4)
_daz_add_qt_target(Script QtScript4)
_daz_add_qt_target(OpenGL QtOpenGL4)
_daz_add_qt_target(Network QtNetwork4)
_daz_add_qt_target(Sql QtSql4)
_daz_add_qt_target(Xml QtXml4)

set(DZSDK_QT_CORE_TARGET Qt4::Core)
set(DZSDK_QT_GUI_TARGET Qt4::Gui)
set(DZSDK_QT_SCRIPT_TARGET Qt4::Script)
set(DZSDK_QT_OPENGL_TARGET Qt4::OpenGL)
set(DZSDK_QT_NETWORK_TARGET Qt4::Network)
set(DZSDK_QT_SQL_TARGET  Qt4::Sql)
set(DZSDK_QT_XML_TARGET  Qt4::Xml)

set(CMAKE_INCLUDE_CURRENT_DIR ON)

if(MSVC)
    # Make sure even in debug we use MD and undef debug for compatibility with release dzcore.
    add_compile_options("/MD" "/U_DEBUG")
endif()

add_definitions(-D_CRT_SECURE_NO_WARNINGS -DDAZPILOTBRIDGE_EXPORTS)

set(DAZ_LIBS
    ${DZSDK_QT_CORE_TARGET}
    ${DZSDK_QT_GUI_TARGET}
    ${DZSDK_QT_SCRIPT_TARGET}
    ${DZSDK_QT_NETWORK_TARGET}
)
if(WIN32)
    list(APPEND DAZ_LIBS "${DAZ_SDK_LIB}/dzcore.lib" Ws2_32.lib)
else()
    find_library(DAZ_DZCORE_LIBRARY NAMES dzcore libdzcore PATHS "${DAZ_SDK_LIB}" NO_DEFAULT_PATH)
    if(DAZ_DZCORE_LIBRARY)
        list(APPEND DAZ_LIBS "${DAZ_DZCORE_LIBRARY}")
    else()
        message(WARNING "dzcore not found for platform ${DAZ_SDK_PLATFORM}; creating dummy target for header-only compilation")
        add_library(dzcore INTERFACE IMPORTED)
        list(APPEND DAZ_LIBS dzcore)
    endif()
endif()

if(WIN32)
    set(DAZ_BRIDGE_PLUGIN_FILE "DazPilotBridge.dll")
    set(DAZ_PLUGINS_DIR "C:/Program Files/DAZ 3D/DAZStudio4/plugins" CACHE PATH "Daz Studio plugin install directory")
elseif(APPLE)
    set(DAZ_BRIDGE_PLUGIN_FILE "libDazPilotBridge.dylib")
    set(DAZ_PLUGINS_DIR "$ENV{HOME}/Library/Application Support/DAZ 3D/Studio4/plugins" CACHE PATH "Daz Studio plugin install directory")
else()
    set(DAZ_BRIDGE_PLUGIN_FILE "libDazPilotBridge.so")
    set(DAZ_PLUGINS_DIR "$ENV{HOME}/.local/share/DAZ 3D/Studio4/plugins" CACHE PATH "Daz Studio plugin install directory")
endif()
set(TAURI_RESOURCES_DIR "${CMAKE_SOURCE_DIR}/../../src-tauri/resources")
option(DAZPILOT_DEPLOY_TO_DAZ "Copy DazPilotBridge to the Daz Studio plugins directory after build" OFF)

# ═══════════════════════════════════════════════════════════════════════════════
# DazPilot Bridge Plugin
# Includes: TCP bridge + AI scene command dispatcher + DazPilotPhy modifier
# ═══════════════════════════════════════════════════════════════════════════════
add_library(DazPilotBridge SHARED
    DazPilotBridge.cpp
    DazPilotBridgePlugin.cpp
    DazPilotExportOptions.cpp
    DazPilotPhyModifier.cpp
    Log.cpp
    DazPilotBridge.h
    DazPilotBridgePlugin.h
    DazPilotPhyModifier.h
    Log.h
)
if(WIN32)
    target_sources(DazPilotBridge PRIVATE
        DazPilotBridge.def
        "${DAZ_SDK_INCLUDE}/dzsdkmemorydefines.cpp"
    )
endif()

target_link_libraries(DazPilotBridge PRIVATE ${DAZ_LIBS})
if(TARGET dzsdkmemory)
    target_link_libraries(DazPilotBridge PRIVATE dzsdkmemory)
endif()
target_include_directories(DazPilotBridge PRIVATE
    ${CMAKE_CURRENT_BINARY_DIR}
    ${DAZ_SDK_ROOT}/include/QtCore
    ${DAZ_SDK_ROOT}/include/QtGui
    ${DAZ_SDK_ROOT}/include/QtScript
    ${DAZ_SDK_ROOT}/include/QtNetwork
    ${DAZ_SDK_ROOT}/include/QtOpenGL
)
if(MSVC)
    target_compile_options(DazPilotBridge PRIVATE "/utf-8")
endif()

set_target_properties(DazPilotBridge PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/dist"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/dist"
)

# ── Unit Tests with doctest ────────────────────────────────────────────────
option(BUILD_TESTS "Build unit tests" ON)
if(BUILD_TESTS)
    add_executable(DazPilotBridgeTests
        tests/main.cpp
        tests/test_math.cpp
        tests/test_log.cpp
        tests/test_json.cpp
        tests/test_jsonutil.cpp
        tests/test_glm.cpp
        tests/test_stb.cpp
        tests/test_simdjson.cpp
        tests/test_utfcpp.cpp
        "${CMAKE_SOURCE_DIR}/../../thirdparty/simdjson/singleheader/simdjson.cpp"
    )
    if(MSVC)
        target_compile_options(DazPilotBridgeTests PRIVATE "/utf-8")
    endif()
    target_include_directories(DazPilotBridgeTests PRIVATE
        "${CMAKE_SOURCE_DIR}/../../thirdparty/doctest/doctest"
        "${CMAKE_SOURCE_DIR}/../../thirdparty/spdlog/include"
        "${CMAKE_SOURCE_DIR}/../../thirdparty/json/include"
        "${CMAKE_SOURCE_DIR}/../../thirdparty/glm"
        "${CMAKE_SOURCE_DIR}/../../thirdparty/stb"
        "${CMAKE_SOURCE_DIR}/../../thirdparty/simdjson/singleheader"
        "${CMAKE_SOURCE_DIR}/../../thirdparty/utfcpp/source"
        "${CMAKE_SOURCE_DIR}"
    )
    include(CTest)
    add_test(NAME DazPilotBridgeTests COMMAND DazPilotBridgeTests)
endif()

# ── Deploy DazPilotBridge ───────────────────────────────────────────────────
add_custom_command(TARGET DazPilotBridge POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory "${TAURI_RESOURCES_DIR}"
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "$<TARGET_FILE:DazPilotBridge>"
        "${TAURI_RESOURCES_DIR}/${DAZ_BRIDGE_PLUGIN_FILE}"
    COMMENT "Copying ${DAZ_BRIDGE_PLUGIN_FILE} -> src-tauri/resources/"
)
if(DAZPILOT_DEPLOY_TO_DAZ)
    add_custom_command(TARGET DazPilotBridge POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "$<TARGET_FILE:DazPilotBridge>"
            "${DAZ_PLUGINS_DIR}/${DAZ_BRIDGE_PLUGIN_FILE}"
        COMMENT "Copying ${DAZ_BRIDGE_PLUGIN_FILE} -> Daz Studio plugins/"
    )
endif()
