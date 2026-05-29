# Third-Party Libraries

This document lists all third-party libraries included in the `thirdparty` directory and their usage status in the DAZPilot project.

## Libraries Overview

| Library | Purpose | License | Current Usage | Notes |
|---------|---------|---------|---------------|-------|
| DAZStudio4.5+ SDK | Daz Studio C++ SDK for plugin development | Proprietary (DAZ 3D) | ✅ Used | Required for building a custom bridge plugin |
| doctest | C++ testing framework | MIT | ✅ Used | Unit tests for bridge plugin via CTest |
| glm | OpenGL Mathematics | MIT/GPL | ✅ Used | Vector/matrix/quaternion math for bridge plugin |
| imgui | Immediate Mode GUI | MIT | ❌ Not used | Requires complex backend integration with DAZ renderer |
| json (nlohmann_json) | JSON for Modern C++ | MIT | ✅ Used | JSON parsing for bridge plugin config and protocol |
| simdjson | High-performance JSON parser | Apache-2.0 | ✅ Used | High-performance JSON parsing for large payloads |
| spdlog | Fast, flexible logging library | MIT | ✅ Used | Logging for bridge plugin (Log.h/Log.cpp) |
| stb | Single-file public domain libraries | Public domain/MIT | ✅ Used | Image loading via stb_image.h |
| utfcpp | UTF-8 library | Boost | ✅ Used | UTF-8 validation and codepoint iteration |

## Detailed Library Information

### DAZStudio4.5+ SDK
- **Location**: `thirdparty/DAZStudio4.5+ SDK`
- **Purpose**: Provides the Daz Studio C++ SDK headers and libraries needed to develop plugins for Daz Studio
- **Usage**: Used by the bridge plugin to interface with Daz Studio (see `plugins/daz3d-bridge/`)
- **Build Integration**: Configured in `plugins/daz3d-bridge/CMakeLists.txt`
- **Note**: This is proprietary software and must be obtained separately from DAZ 3D

### doctest
- **Location**: `thirdparty/doctest`
- **Purpose**: Lightweight C++ testing framework that allows writing tests directly in production code
- **License**: MIT
- **Current Usage**: Used for unit testing the bridge plugin
- **Integration**: Integrated into `plugins/daz3d-bridge/CMakeLists.txt`. Tests live in `plugins/daz3d-bridge/tests/`.
- **Header**: `thirdparty/doctest/doctest/doctest.h`

### glm
- **Location**: `thirdparty/glm`
- **Purpose**: Header-only C++ mathematics library for graphics software based on the OpenGL Shading Language (GLSL) specifications
- **License**: MIT (Modified) / GPL
- **Current Usage**: Used for vector, matrix, and quaternion math in the bridge plugin
- **Integration**: Included in `plugins/daz3d-bridge/CMakeLists.txt`. Replaces/complements `DzVec3`/`DzQuat` from the DAZ SDK.
- **Header**: `thirdparty/glm/glm/glm.hpp`

### imgui
- **Location**: `thirdparty/imgui`
- **Purpose**: Bloat-free Immediate Mode Graphical User Interface library with minimal dependencies
- **License**: MIT
- **Current Usage**: Not currently used in the project
- **Potential Use**: Could be used to create a debug overlay or simple UI for the bridge plugin during development
- **Integration Notes**: Requires backend implementation for the target platform (DirectX, OpenGL, etc.)

### json (nlohmann_json)
- **Location**: `thirdparty/json`
- **Purpose**: JSON for Modern C++ - intuitive syntax, trivial integration, tested, lightweight
- **License**: MIT
- **Current Usage**: Used for JSON parsing in the bridge plugin (replacement for Qt JSON which is unavailable in the SDK's Qt4)
- **Integration**: Included in `plugins/daz3d-bridge/CMakeLists.txt`. Header `JsonUtil.h` provides utility functions.
- **Header**: `thirdparty/json/include/nlohmann/json.hpp`
- **Note**: Replaces `QJsonDocument`/`QJsonObject` usage since the DAZ SDK bundles Qt4 which lacks these Qt5+ classes

### simdjson
- **Location**: `thirdparty/simdjson`
- **Purpose**: High-performance JSON parser that uses SIMD instructions
- **License**: Apache-2.0
- **Current Usage**: Used for high-performance JSON parsing in the bridge plugin
- **Integration**: Included in `plugins/daz3d-bridge/CMakeLists.txt`. Requires compiling `singleheader/simdjson.cpp` alongside the project.
- **Headers**: `thirdparty/simdjson/singleheader/simdjson.h`
- **Note**: On-demand (streaming) DOM API for minimal memory allocation during parsing

### spdlog
- **Location**: `thirdparty/spdlog`
- **Purpose**: Fast, flexible, header-only C++ logging library
- **License**: MIT
- **Current Usage**: Used for logging in the bridge plugin
- **Integration**: Integrated into `plugins/daz3d-bridge/CMakeLists.txt`. Provides `Log.h`/`Log.cpp` with macros `LOG_TRACE`, `LOG_DEBUG`, `LOG_INFO`, `LOG_WARN`, `LOG_ERROR`, `LOG_CRITICAL`.
- **Header**: `thirdparty/spdlog/include/spdlog/spdlog.h`

### stb
- **Location**: `thirdparty/stb`
- **Purpose**: Single-file public domain (or MIT licensed) libraries for C/C++
- **License**: Public domain/MIT
- **Current Usage**: Used for image loading via stb_image.h
- **Integration**: Included in `plugins/daz3d-bridge/CMakeLists.txt`. Define `STB_IMAGE_IMPLEMENTATION` in exactly one `.cpp` file.
- **Headers**: `thirdparty/stb/stb_image.h`, `thirdparty/stb/stb_image_write.h`
- **Note**: stb_image.h can decode PNG, JPEG, BMP, GIF, and other formats without external dependencies

### utfcpp
- **Location**: `thirdparty/utfcpp`
- **Purpose**: UTF-8 library in C++ - validation, iteration, conversion
- **License**: Boost
- **Current Usage**: Used for UTF-8 validation and codepoint iteration in the bridge plugin
- **Integration**: Included in `plugins/daz3d-bridge/CMakeLists.txt`. Header-only, no compilation required.
- **Headers**: `thirdparty/utfcpp/source/utf8.h`, `thirdparty/utfcpp/source/utf8/checked.h`

## Build System Integration

For header-only libraries (doctest, glm, imgui, json, spdlog, stb, utfcpp):
- Add the appropriate include directory to the bridge plugin's CMakeLists.txt
- No additional linking required

For libraries requiring compilation (simdjson):
- Add the library as a subdirectory or pre-built dependency
- Link against the appropriate targets

## Licensing Considerations

All third-party libraries (except the DAZ SDK) use permissive licenses compatible with the MIT license used by DAZPilot:
- MIT: doctest, glm, imgui, json, spdlog
- Apache-2.0: simdjson
- Public domain/MIT: stb
- Boost: utfcpp

The DAZStudio4.5+ SDK is proprietary and subject to DAZ 3D's licensing terms.

## Recommendations

1. **Completed**: doctest integrated for bridge plugin unit testing (see `plugins/daz3d-bridge/tests/`)
2. **Completed**: spdlog integrated for bridge plugin logging (see `plugins/daz3d-bridge/Log.h`)
3. **Completed**: nlohmann/json integrated for JSON parsing (see `plugins/daz3d-bridge/JsonUtil.h`)
4. **Completed**: glm integrated for vector/matrix math
5. **Completed**: stb integrated for image loading
6. **Completed**: simdjson integrated for high-performance JSON parsing
7. **Completed**: utfcpp integrated for UTF-8 handling
8. **Not integrated**: imgui - requires complex backend integration with DAZ Studio's render pipeline

Last updated: 2026-05-21