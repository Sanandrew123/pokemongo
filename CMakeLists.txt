# 高性能宝可梦游戏 C++模块构建配置
# 开发心理：C++负责性能关键路径，使用现代C++17特性和SIMD优化
# 重点关注内存局部性和缓存友好的数据结构设计

cmake_minimum_required(VERSION 3.20)
project(PokemonGO_Native VERSION 1.0.0 LANGUAGES CXX)

# 编译器要求
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# 跨平台优化标志
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    if(MSVC)
        # Microsoft Visual C++ 编译器
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /O2 /Oi /Ot /Oy /GL")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /arch:AVX2")  # SIMD优化
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /DNDEBUG")
        set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} /LTCG")  # 链接时代码生成
    else()
        # GCC/Clang 编译器
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -O3 -march=native -mtune=native")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -ffast-math -funroll-loops")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -DNDEBUG")
    endif()
endif()

# 调试标志  
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    if(MSVC)
        # Microsoft Visual C++ 调试标志
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /Od /Zi /RTC1")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /W4")  # 警告级别4
    else()
        # GCC/Clang 调试标志
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -g -O0 -fsanitize=address")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -Wpedantic")
    endif()
endif()

# 包含目录
include_directories(${CMAKE_SOURCE_DIR}/native/include)

# 数学引擎库
add_library(math_engine STATIC
    native/src/math/vector_math.cpp
    native/src/math/matrix_ops.cpp  
    native/src/math/damage_calc.cpp
    native/src/math/pathfinding.cpp
)

# 音频DSP库
add_library(audio_dsp STATIC
    native/src/audio/audio_engine.cpp
    native/src/audio/effects.cpp
    native/src/audio/spatial_audio.cpp
    native/src/audio/compression.cpp
)

# 图形处理库
add_library(graphics_proc STATIC
    native/src/graphics/image_loader.cpp
    native/src/graphics/texture_comp.cpp
    native/src/graphics/sprite_batch.cpp
    native/src/graphics/shader_cache.cpp
)

# 物理引擎库
add_library(physics_engine STATIC
    native/src/physics/collision.cpp
    native/src/physics/spatial_hash.cpp
    native/src/physics/rigid_body.cpp
)

# 网络优化库
add_library(network_opt STATIC
    native/src/network/packet_pool.cpp
    native/src/network/compression.cpp
    native/src/network/encryption.cpp
)

# 统一的C接口库 (用于FFI)
add_library(pokemongo_native SHARED
    native/src/ffi_interface.cpp
)

# 链接所有静态库到共享库
target_link_libraries(pokemongo_native 
    math_engine
    audio_dsp
    graphics_proc
    physics_engine
    network_opt
)

# 设置SIMD支持
target_compile_definitions(math_engine PRIVATE USE_SIMD=1)
target_compile_definitions(graphics_proc PRIVATE USE_SIMD=1)

# 平台特定设置
if(WIN32)
    target_compile_definitions(pokemongo_native PRIVATE WINDOWS_PLATFORM=1)
elseif(UNIX AND NOT APPLE)
    target_compile_definitions(pokemongo_native PRIVATE LINUX_PLATFORM=1)
    target_link_libraries(pokemongo_native pthread)
elseif(APPLE)
    target_compile_definitions(pokemongo_native PRIVATE MACOS_PLATFORM=1)
endif()

# 安装规则
install(TARGETS pokemongo_native
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
)