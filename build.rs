// 构建脚本 - 编译时配置和C++集成
// 开发心理：构建系统是项目基础设施，需要处理跨平台编译、C++集成、资源处理
// 目标是实现零配置构建，开发者clone后直接cargo build即可

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=cpp/");
    println!("cargo:rerun-if-changed=native/");
    
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    
    println!("cargo:rustc-env=TARGET_OS={}", target_os);
    println!("cargo:rustc-env=TARGET_ARCH={}", target_arch);
    
    // 检查是否启用了native feature
    if env::var("CARGO_FEATURE_NATIVE").is_ok() {
        // 编译C++模块
        build_cpp_modules();
        
        // 生成FFI绑定
        generate_bindings();
    } else {
        println!("cargo:warning=native feature未启用，跳过C++模块编译");
    }
    
    // 平台特定配置
    configure_platform();
    
    println!("构建脚本执行完成");
}

fn build_cpp_modules() {
    println!("开始编译C++模块...");
    
    // 数学优化模块
    if std::path::Path::new("cpp/math/simd_operations.cpp").exists() {
        cc::Build::new()
            .cpp(true)
            .file("cpp/math/simd_operations.cpp")
            .include("cpp/math")
            .flag_if_supported("-O3")
            .flag_if_supported("-mavx2")
            .flag_if_supported("-mfma")
            .compile("pokemongo_math");
        println!("数学优化模块编译完成");
    } else {
        println!("cargo:warning=cpp/math/simd_operations.cpp 不存在，跳过数学模块编译");
    }
    
    // 图形优化模块
    if std::path::Path::new("cpp/graphics/fast_renderer.cpp").exists() {
        cc::Build::new()
            .cpp(true)
            .file("cpp/graphics/fast_renderer.cpp")
            .include("cpp/graphics")
            .flag_if_supported("-O3")
            .compile("pokemongo_graphics");
        println!("图形优化模块编译完成");
    } else {
        println!("cargo:warning=cpp/graphics/fast_renderer.cpp 不存在，跳过图形模块编译");
    }
    
    // 物理碰撞模块
    if std::path::Path::new("cpp/physics/collision.cpp").exists() {
        cc::Build::new()
            .cpp(true)
            .file("cpp/physics/collision.cpp")
            .include("cpp/physics")
            .flag_if_supported("-O3")
            .compile("pokemongo_physics");
        println!("物理碰撞模块编译完成");
    } else {
        println!("cargo:warning=cpp/physics/collision.cpp 不存在，跳过物理模块编译");
    }
    
    println!("C++模块编译检查完成");
}

fn generate_bindings() {
    println!("生成FFI绑定...");
    
    let mut builder = bindgen::Builder::default()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
    
    let mut has_headers = false;
    
    if std::path::Path::new("cpp/math/simd_operations.h").exists() {
        builder = builder.header("cpp/math/simd_operations.h");
        has_headers = true;
        println!("添加数学模块头文件");
    } else {
        println!("cargo:warning=cpp/math/simd_operations.h 不存在，跳过");
    }
    
    if std::path::Path::new("cpp/graphics/fast_renderer.h").exists() {
        builder = builder.header("cpp/graphics/fast_renderer.h");
        has_headers = true;
        println!("添加图形模块头文件");
    } else {
        println!("cargo:warning=cpp/graphics/fast_renderer.h 不存在，跳过");
    }
    
    if std::path::Path::new("cpp/physics/collision.h").exists() {
        builder = builder.header("cpp/physics/collision.h");
        has_headers = true;
        println!("添加物理模块头文件");
    } else {
        println!("cargo:warning=cpp/physics/collision.h 不存在，跳过");
    }
    
    if has_headers {
        let bindings = builder
            .generate()
            .expect("无法生成绑定");
        
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("无法写入绑定文件");
        
        println!("FFI绑定生成完成");
    } else {
        println!("cargo:warning=没有找到C++头文件，跳过绑定生成");
    }
}

fn configure_platform() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    match target_os.as_str() {
        "windows" => {
            configure_windows();
        }
        "linux" => {
            configure_linux();
        }
        "macos" => {
            configure_macos();
        }
        _ => {
            println!("cargo:warning=不支持的平台: {}", target_os);
        }
    }
}

fn configure_windows() {
    println!("配置Windows平台...");
    
    // 基础Windows链接
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=kernel32");
    
    // 仅在启用native feature时链接OpenGL
    if env::var("CARGO_FEATURE_NATIVE").is_ok() {
        println!("cargo:rustc-link-lib=opengl32");
        println!("cargo:rustc-link-lib=gdi32");
        println!("native feature启用，链接OpenGL库");
    }
    
    // MSVC运行时
    if env::var("CARGO_CFG_TARGET_ENV").unwrap() == "msvc" {
        println!("cargo:rustc-link-lib=msvcrt");
    }
    
    println!("Windows平台配置完成");
}

fn configure_linux() {
    println!("配置Linux平台...");
    
    // Linux特定链接
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=GL");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=m");
    
    // pkg-config依赖 (仅在启用时检查)
    if env::var("CARGO_FEATURE_NATIVE").is_ok() {
        match pkg_config::Config::new().probe("alsa") {
            Ok(_) => println!("cargo:rustc-cfg=feature=\"alsa\""),
            Err(_) => println!("cargo:warning=alsa开发包未找到")
        }
        
        match pkg_config::Config::new().probe("pulseaudio") {
            Ok(_) => println!("cargo:rustc-cfg=feature=\"pulseaudio\""),
            Err(_) => println!("cargo:warning=pulseaudio开发包未找到")
        }
    }
    
    println!("Linux平台配置完成");
}

fn configure_macos() {
    println!("配置macOS平台...");
    
    // macOS框架
    println!("cargo:rustc-link-lib=framework=Cocoa");
    println!("cargo:rustc-link-lib=framework=OpenGL");
    println!("cargo:rustc-link-lib=framework=AudioToolbox");
    println!("cargo:rustc-link-lib=framework=CoreAudio");
    
    println!("macOS平台配置完成");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_script() {
        // 测试构建脚本基本功能
        assert!(!env::var("OUT_DIR").unwrap().is_empty());
    }
}