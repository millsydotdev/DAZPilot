cmake_minimum_required(VERSION 3.15)
project(DazPilotBridge)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# CRITICAL: Define WIN32_LEAN_AND_MEAN and include winsock2 FIRST to avoid conflicts
add_definitions(-DWIN32_LEAN_AND_MEAN -DNOMINMAX)

set(DAZ_SDK_ROOT    "E:/DazAI/DAZStudio4.5+ SDK")
set(DAZ_SDK_INCLUDE "${DAZ_SDK_ROOT}/include")
set(DAZ_SDK_LIB     "${DAZ_SDK_ROOT}/lib/x64")

include_directories(${DAZ_SDK_INCLUDE})

# Setup Qt from the DAZ SDK for AUTOMOC
set(DZ_MIXED_PLATFORM x64)
set(DAZ_SDK_QTCORE_LIBRARY "${DAZ_SDK_ROOT}/lib/${DZ_MIXED_PLATFORM}/QtCore4.lib")
set(QT_QTCORE_LIBRARY_RELEASE ${DAZ_SDK_QTCORE_LIBRARY})
set(QT_QMAKE_EXECUTABLE "${DAZ_SDK_ROOT}/bin/${DZ_MIXED_PLATFORM}/qmake.exe")
set(QT_BINARY_DIR "${DAZ_SDK_ROOT}/bin/${DZ_MIXED_PLATFORM}")
set(QT_HEADERS_DIR "${DAZ_SDK_ROOT}/include")
set(QT_QTCORE_INCLUDE_DIR "${DAZ_SDK_ROOT}/include/QtCore")

# the qt find module needs this folder but our build does not so just fake it
file(MAKE_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/DUMMY_MKSPECS/default" )
set(QT_MKSPECS_DIR "${CMAKE_CURRENT_BINARY_DIR}/DUMMY_MKSPECS")

find_package(Qt4 4.8.1 REQUIRED QtCore QtGui QtScript QtOpenGL QtNetwork QtSql QtXml)

set(DZSDK_QT_CORE_TARGET Qt4::QtCore)
set(DZSDK_QT_GUI_TARGET Qt4::QtGui)
set(DZSDK_QT_SCRIPT_TARGET Qt4::QtScript)
set(DZSDK_QT_OPENGL_TARGET Qt4::QtOpenGL)
set(DZSDK_QT_NETWORK_TARGET Qt4::QtNetwork)
set(DZSDK_QT_SQL_TARGET	Qt4::QtSql)
set(DZSDK_QT_XML_TARGET	Qt4::QtXml)

list(APPEND CMAKE_AUTOMOC_MOC_OPTIONS -i)

set(CMAKE_AUTOMOC ON)
set(CMAKE_INCLUDE_CURRENT_DIR ON)

# Make sure even in debug we use MD and undef debug for compatibility with release dzcore
add_compile_options("/MD" "/U_DEBUG")

add_definitions(-D_CRT_SECURE_NO_WARNINGS -DDAZPILOTBRIDGE_EXPORTS)

set(DAZ_LIBS
    "${DAZ_SDK_LIB}/dzcore.lib"
    ${DZSDK_QT_CORE_TARGET}
    ${DZSDK_QT_GUI_TARGET}
    ${DZSDK_QT_SCRIPT_TARGET}
    ${DZSDK_QT_NETWORK_TARGET}
    Ws2_32.lib
)

set(DAZ_PLUGINS_DIR     "C:/Program Files/DAZ 3D/DAZStudio4/plugins")
set(TAURI_RESOURCES_DIR "${CMAKE_SOURCE_DIR}/../../src-tauri/resources")

# ═══════════════════════════════════════════════════════════════════════════════
# DazPilot Bridge Plugin
# Includes: TCP bridge + AI scene command dispatcher + DazPilotPhy modifier
# ═══════════════════════════════════════════════════════════════════════════════
add_library(DazPilotBridge SHARED
    DazPilotBridge.cpp
    DazPilotBridgePlugin.cpp
    DazPilotPhyModifier.cpp
    DazPilotBridge.h
    DazPilotBridgePlugin.h
    DazPilotPhyModifier.h
)

target_link_libraries(DazPilotBridge PRIVATE ${DAZ_LIBS})

set_target_properties(DazPilotBridge PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/dist"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/dist"
)

# ── Deploy DazPilotBridge ───────────────────────────────────────────────────
add_custom_command(TARGET DazPilotBridge POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory "${TAURI_RESOURCES_DIR}"
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "$<TARGET_FILE:DazPilotBridge>"
        "${TAURI_RESOURCES_DIR}/DazPilotBridge.dll"
    COMMENT "Copying DazPilotBridge.dll -> src-tauri/resources/"
)
add_custom_command(TARGET DazPilotBridge POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "$<TARGET_FILE:DazPilotBridge>"
        "${DAZ_PLUGINS_DIR}/DazPilotBridge.dll"
    COMMENT "Copying DazPilotBridge.dll -> Daz Studio plugins/"
)
