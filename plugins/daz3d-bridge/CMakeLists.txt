cmake_minimum_required(VERSION 3.15)
project(DazPilotBridge)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(WIN32)
    add_definitions(-DWIN32_LEAN_AND_MEAN -DNOMINMAX)
endif()

if(DEFINED ENV{DAZ_SDK_ROOT} AND NOT DEFINED DAZ_SDK_ROOT)
    set(DAZ_SDK_ROOT "$ENV{DAZ_SDK_ROOT}")
else()
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

# Setup Qt from the DAZ SDK for AUTOMOC
set(DAZ_SDK_QTCORE_LIBRARY "${DAZ_SDK_LIB}/${CMAKE_STATIC_LIBRARY_PREFIX}QtCore4${CMAKE_STATIC_LIBRARY_SUFFIX}")
if(WIN32)
    set(DAZ_SDK_QTCORE_LIBRARY "${DAZ_SDK_LIB}/QtCore4.lib")
endif()
set(QT_QTCORE_LIBRARY_RELEASE ${DAZ_SDK_QTCORE_LIBRARY})
set(QT_LIBRARY_DIR "${DAZ_SDK_LIB}")
if(WIN32)
    set(QT_QMAKE_EXECUTABLE "${DAZ_SDK_BIN}/qmake.exe")
else()
    set(QT_QMAKE_EXECUTABLE "${DAZ_SDK_BIN}/qmake")
endif()
set(QT_BINARY_DIR "${DAZ_SDK_BIN}")
set(QT_HEADERS_DIR "${DAZ_SDK_ROOT}/include")
set(QT_QTCORE_INCLUDE_DIR "${DAZ_SDK_ROOT}/include/QtCore")
set(QT_QTGUI_INCLUDE_DIR "${DAZ_SDK_ROOT}/include/QtGui")
set(QT_QTSCRIPT_INCLUDE_DIR "${DAZ_SDK_ROOT}/include/QtScript")
set(QT_QTOPENGL_INCLUDE_DIR "${DAZ_SDK_ROOT}/include/QtOpenGL")
set(QT_QTNETWORK_INCLUDE_DIR "${DAZ_SDK_ROOT}/include/QtNetwork")
set(QT_QTSQL_INCLUDE_DIR "${DAZ_SDK_ROOT}/include/QtSql")
set(QT_QTXML_INCLUDE_DIR "${DAZ_SDK_ROOT}/include/QtXml")
# The Qt4 libs and moc are bundled inside the Daz SDK
set(QT_MOC_EXECUTABLE "${DAZ_SDK_BIN}/moc${CMAKE_EXECUTABLE_SUFFIX}")
# Create an imported Qt::moc target so AUTOMOC can find moc without find_package(Qt)
add_executable(Qt::moc IMPORTED)
set_property(TARGET Qt::moc PROPERTY IMPORTED_LOCATION "${QT_MOC_EXECUTABLE}")

# Generate moc files manually — AUTOMOC struggles with this custom Qt4 setup
set(MOC_FILES
    ${CMAKE_CURRENT_BINARY_DIR}/DazPilotBridgePlugin.moc
    ${CMAKE_CURRENT_BINARY_DIR}/moc_DazPilotPhyModifier.cpp
)
add_custom_command(OUTPUT ${MOC_FILES}
    COMMAND ${QT_MOC_EXECUTABLE} -I${DAZ_SDK_INCLUDE} -I${DAZ_SDK_ROOT}/include/QtCore
        ${CMAKE_CURRENT_SOURCE_DIR}/DazPilotBridgePlugin.cpp
        -o ${CMAKE_CURRENT_BINARY_DIR}/DazPilotBridgePlugin.moc
    COMMAND ${QT_MOC_EXECUTABLE} -I${DAZ_SDK_INCLUDE} -I${DAZ_SDK_ROOT}/include/QtCore
        ${CMAKE_CURRENT_SOURCE_DIR}/DazPilotPhyModifier.h
        -o ${CMAKE_CURRENT_BINARY_DIR}/moc_DazPilotPhyModifier.cpp
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/DazPilotBridgePlugin.cpp
            ${CMAKE_CURRENT_SOURCE_DIR}/DazPilotPhyModifier.h
)
set_source_files_properties(${MOC_FILES} PROPERTIES GENERATED TRUE)

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

list(APPEND CMAKE_AUTOMOC_MOC_OPTIONS -i)

set(CMAKE_AUTOMOC ON)
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
    ${MOC_FILES}
)
if(WIN32)
    target_sources(DazPilotBridge PRIVATE DazPilotBridge.def)
endif()

target_link_libraries(DazPilotBridge PRIVATE ${DAZ_LIBS})
target_include_directories(DazPilotBridge PRIVATE ${CMAKE_CURRENT_BINARY_DIR})
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
