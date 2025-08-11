/*
 * FFI绑定层 - Foreign Function Interface Bindings
 * 
 * 开发心理过程：
 * 设计Rust与C/C++模块的接口层，需要处理内存安全、错误传播、数据转换等问题
 * 重点关注零拷贝优化、内存管理和ABI兼容性
 * 提供类型安全的Rust包装器来隐藏底层C接口的复杂性
 */

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_float, c_double, c_void};
use std::slice;
use std::ptr;

// ============================================================================
// 数学引擎FFI绑定
// ============================================================================

#[repr(C)]
pub struct CVec2 {
    pub x: c_float,
    pub y: c_float,
}

#[repr(C)]
pub struct CVec3 {
    pub x: c_float,
    pub y: c_float,
    pub z: c_float,
}

#[repr(C)]
pub struct CMatrix4 {
    pub data: [c_float; 16],
}

#[repr(C)]
pub struct CDamageCalcParams {
    pub attacker_level: c_int,
    pub attacker_attack: c_int,
    pub defender_defense: c_int,
    pub move_power: c_int,
    pub type_effectiveness: c_float,
    pub critical_hit: c_int,
    pub weather_modifier: c_float,
}

#[repr(C)]
pub struct CPathNode {
    pub x: c_int,
    pub y: c_int,
    pub g_cost: c_float,
    pub h_cost: c_float,
    pub f_cost: c_float,
}

extern "C" {
    // 向量数学操作
    pub fn vec2_add(a: *const CVec2, b: *const CVec2, result: *mut CVec2);
    pub fn vec2_subtract(a: *const CVec2, b: *const CVec2, result: *mut CVec2);
    pub fn vec2_multiply_scalar(vec: *const CVec2, scalar: c_float, result: *mut CVec2);
    pub fn vec2_dot_product(a: *const CVec2, b: *const CVec2) -> c_float;
    pub fn vec2_magnitude(vec: *const CVec2) -> c_float;
    pub fn vec2_normalize(vec: *const CVec2, result: *mut CVec2);
    pub fn vec2_distance(a: *const CVec2, b: *const CVec2) -> c_float;

    // 3D向量操作
    pub fn vec3_add(a: *const CVec3, b: *const CVec3, result: *mut CVec3);
    pub fn vec3_cross_product(a: *const CVec3, b: *const CVec3, result: *mut CVec3);
    pub fn vec3_transform_matrix4(vec: *const CVec3, matrix: *const CMatrix4, result: *mut CVec3);

    // 矩阵运算
    pub fn matrix4_identity(result: *mut CMatrix4);
    pub fn matrix4_multiply(a: *const CMatrix4, b: *const CMatrix4, result: *mut CMatrix4);
    pub fn matrix4_translate(matrix: *mut CMatrix4, x: c_float, y: c_float, z: c_float);
    pub fn matrix4_rotate_z(matrix: *mut CMatrix4, angle: c_float);
    pub fn matrix4_scale(matrix: *mut CMatrix4, x: c_float, y: c_float, z: c_float);

    // 伤害计算优化
    pub fn calculate_damage_fast(params: *const CDamageCalcParams) -> c_int;
    pub fn calculate_damage_range(params: *const CDamageCalcParams, results: *mut c_int, count: c_int);

    // A*寻路算法
    pub fn pathfinding_create_grid(width: c_int, height: c_int) -> *mut c_void;
    pub fn pathfinding_destroy_grid(grid: *mut c_void);
    pub fn pathfinding_set_obstacle(grid: *mut c_void, x: c_int, y: c_int, is_obstacle: c_int);
    pub fn pathfinding_find_path(
        grid: *mut c_void,
        start_x: c_int,
        start_y: c_int,
        end_x: c_int,
        end_y: c_int,
        path: *mut CPathNode,
        max_nodes: c_int,
    ) -> c_int;
}

// ============================================================================
// 音频DSP FFI绑定
// ============================================================================

#[repr(C)]
pub struct CAudioBuffer {
    pub data: *mut c_float,
    pub size: c_int,
    pub channels: c_int,
    pub sample_rate: c_int,
}

#[repr(C)]
pub struct C3DAudioParams {
    pub listener_pos: CVec3,
    pub listener_forward: CVec3,
    pub listener_up: CVec3,
    pub source_pos: CVec3,
    pub max_distance: c_float,
    pub rolloff_factor: c_float,
}

#[repr(C)]
pub struct CReverbParams {
    pub room_size: c_float,
    pub damping: c_float,
    pub wet_level: c_float,
    pub dry_level: c_float,
    pub width: c_float,
}

extern "C" {
    // 音频引擎
    pub fn audio_engine_create(sample_rate: c_int, buffer_size: c_int) -> *mut c_void;
    pub fn audio_engine_destroy(engine: *mut c_void);
    pub fn audio_engine_process(engine: *mut c_void, input: *const CAudioBuffer, output: *mut CAudioBuffer);

    // 音频效果
    pub fn audio_apply_reverb(
        input: *const CAudioBuffer,
        output: *mut CAudioBuffer,
        params: *const CReverbParams,
    ) -> c_int;
    pub fn audio_apply_low_pass_filter(
        input: *const CAudioBuffer,
        output: *mut CAudioBuffer,
        cutoff_freq: c_float,
        q_factor: c_float,
    ) -> c_int;
    pub fn audio_apply_high_pass_filter(
        input: *const CAudioBuffer,
        output: *mut CAudioBuffer,
        cutoff_freq: c_float,
        q_factor: c_float,
    ) -> c_int;

    // 3D音频定位
    pub fn audio_calculate_3d_position(
        params: *const C3DAudioParams,
        gain: *mut c_float,
        pan: *mut c_float,
    ) -> c_int;

    // 音频压缩
    pub fn audio_compress_ogg(
        input: *const CAudioBuffer,
        output: *mut *mut c_void,
        output_size: *mut c_int,
        quality: c_float,
    ) -> c_int;
    pub fn audio_decompress_ogg(
        input: *const c_void,
        input_size: c_int,
        output: *mut CAudioBuffer,
    ) -> c_int;
}

// ============================================================================
// 图形处理 FFI绑定
// ============================================================================

#[repr(C)]
pub struct CImage {
    pub data: *mut u8,
    pub width: c_int,
    pub height: c_int,
    pub channels: c_int,
    pub format: c_int,
}

#[repr(C)]
pub struct CTextureCompression {
    pub format: c_int,
    pub quality: c_int,
    pub use_alpha: c_int,
    pub generate_mipmaps: c_int,
}

#[repr(C)]
pub struct CSpriteBatch {
    pub sprites: *mut CSprite,
    pub count: c_int,
    pub capacity: c_int,
}

#[repr(C)]
pub struct CSprite {
    pub position: CVec2,
    pub size: CVec2,
    pub rotation: c_float,
    pub texture_id: c_int,
    pub uv_rect: [c_float; 4],
    pub color: [c_float; 4],
}

extern "C" {
    // 图像加载和处理
    pub fn image_load_from_file(filename: *const c_char) -> *mut CImage;
    pub fn image_load_from_memory(data: *const u8, size: c_int) -> *mut CImage;
    pub fn image_destroy(image: *mut CImage);
    pub fn image_resize(image: *mut CImage, new_width: c_int, new_height: c_int) -> c_int;
    pub fn image_flip_vertical(image: *mut CImage) -> c_int;
    pub fn image_flip_horizontal(image: *mut CImage) -> c_int;

    // 纹理压缩
    pub fn texture_compress_dxt(
        image: *const CImage,
        params: *const CTextureCompression,
        compressed_data: *mut *mut u8,
        compressed_size: *mut c_int,
    ) -> c_int;
    pub fn texture_compress_etc(
        image: *const CImage,
        params: *const CTextureCompression,
        compressed_data: *mut *mut u8,
        compressed_size: *mut c_int,
    ) -> c_int;

    // 批量渲染
    pub fn sprite_batch_create(capacity: c_int) -> *mut CSpriteBatch;
    pub fn sprite_batch_destroy(batch: *mut CSpriteBatch);
    pub fn sprite_batch_add_sprite(batch: *mut CSpriteBatch, sprite: *const CSprite) -> c_int;
    pub fn sprite_batch_clear(batch: *mut CSpriteBatch);
    pub fn sprite_batch_sort_by_depth(batch: *mut CSpriteBatch);
    pub fn sprite_batch_render(batch: *const CSpriteBatch, view_matrix: *const CMatrix4) -> c_int;

    // 着色器缓存
    pub fn shader_cache_create() -> *mut c_void;
    pub fn shader_cache_destroy(cache: *mut c_void);
    pub fn shader_cache_compile_shader(
        cache: *mut c_void,
        vertex_source: *const c_char,
        fragment_source: *const c_char,
        shader_id: *mut c_int,
    ) -> c_int;
    pub fn shader_cache_get_shader(cache: *mut c_void, shader_id: c_int) -> *mut c_void;
}

// ============================================================================
// 物理引擎 FFI绑定
// ============================================================================

#[repr(C)]
pub struct CCollisionShape {
    pub shape_type: c_int,
    pub position: CVec2,
    pub size: CVec2,
    pub rotation: c_float,
}

#[repr(C)]
pub struct CRigidBody {
    pub position: CVec2,
    pub velocity: CVec2,
    pub acceleration: CVec2,
    pub mass: c_float,
    pub friction: c_float,
    pub restitution: c_float,
    pub is_static: c_int,
}

#[repr(C)]
pub struct CCollisionResult {
    pub has_collision: c_int,
    pub normal: CVec2,
    pub penetration: c_float,
    pub contact_point: CVec2,
}

extern "C" {
    // 碰撞检测
    pub fn collision_check_aabb(a: *const CCollisionShape, b: *const CCollisionShape) -> c_int;
    pub fn collision_check_circle(a: *const CCollisionShape, b: *const CCollisionShape) -> c_int;
    pub fn collision_check_detailed(
        a: *const CCollisionShape,
        b: *const CCollisionShape,
        result: *mut CCollisionResult,
    ) -> c_int;

    // 空间哈希
    pub fn spatial_hash_create(cell_size: c_float, initial_capacity: c_int) -> *mut c_void;
    pub fn spatial_hash_destroy(hash: *mut c_void);
    pub fn spatial_hash_insert(hash: *mut c_void, id: c_int, shape: *const CCollisionShape) -> c_int;
    pub fn spatial_hash_remove(hash: *mut c_void, id: c_int) -> c_int;
    pub fn spatial_hash_query(
        hash: *mut c_void,
        area: *const CCollisionShape,
        results: *mut c_int,
        max_results: c_int,
    ) -> c_int;
    pub fn spatial_hash_clear(hash: *mut c_void);

    // 刚体物理
    pub fn rigidbody_integrate(body: *mut CRigidBody, delta_time: c_float);
    pub fn rigidbody_apply_force(body: *mut CRigidBody, force: *const CVec2);
    pub fn rigidbody_apply_impulse(body: *mut CRigidBody, impulse: *const CVec2);
    pub fn rigidbody_resolve_collision(
        a: *mut CRigidBody,
        b: *mut CRigidBody,
        collision: *const CCollisionResult,
    );
}

// ============================================================================
// 网络优化 FFI绑定
// ============================================================================

#[repr(C)]
pub struct CPacket {
    pub data: *mut u8,
    pub size: c_int,
    pub capacity: c_int,
    pub type_id: c_int,
}

#[repr(C)]
pub struct CCompressionParams {
    pub algorithm: c_int,
    pub compression_level: c_int,
    pub dictionary_size: c_int,
}

extern "C" {
    // 数据包池
    pub fn packet_pool_create(initial_capacity: c_int, packet_size: c_int) -> *mut c_void;
    pub fn packet_pool_destroy(pool: *mut c_void);
    pub fn packet_pool_acquire(pool: *mut c_void) -> *mut CPacket;
    pub fn packet_pool_release(pool: *mut c_void, packet: *mut CPacket) -> c_int;

    // 网络压缩
    pub fn network_compress(
        input: *const u8,
        input_size: c_int,
        output: *mut u8,
        output_capacity: c_int,
        params: *const CCompressionParams,
    ) -> c_int;
    pub fn network_decompress(
        input: *const u8,
        input_size: c_int,
        output: *mut u8,
        output_capacity: c_int,
    ) -> c_int;

    // 加密算法
    pub fn encrypt_aes256(
        data: *const u8,
        data_size: c_int,
        key: *const u8,
        iv: *const u8,
        encrypted: *mut u8,
        encrypted_capacity: c_int,
    ) -> c_int;
    pub fn decrypt_aes256(
        encrypted: *const u8,
        encrypted_size: c_int,
        key: *const u8,
        iv: *const u8,
        data: *mut u8,
        data_capacity: c_int,
    ) -> c_int;
}

// ============================================================================
// Rust包装器实现
// ============================================================================

use crate::core::math::{Vec2, Vec3, Matrix4};
use crate::core::error::{GameResult, GameError};

pub struct MathEngine;

impl MathEngine {
    pub fn vec2_add_safe(a: Vec2, b: Vec2) -> Vec2 {
        let c_a = CVec2 { x: a.x, y: a.y };
        let c_b = CVec2 { x: b.x, y: b.y };
        let mut result = CVec2 { x: 0.0, y: 0.0 };
        
        unsafe {
            vec2_add(&c_a, &c_b, &mut result);
        }
        
        Vec2::new(result.x, result.y)
    }

    pub fn vec2_distance_safe(a: Vec2, b: Vec2) -> f32 {
        let c_a = CVec2 { x: a.x, y: a.y };
        let c_b = CVec2 { x: b.x, y: b.y };
        
        unsafe { vec2_distance(&c_a, &c_b) }
    }

    pub fn calculate_pokemon_damage(
        attacker_level: u8,
        attacker_attack: u16,
        defender_defense: u16,
        move_power: u8,
        type_effectiveness: f32,
        is_critical: bool,
        weather_modifier: f32,
    ) -> GameResult<u16> {
        let params = CDamageCalcParams {
            attacker_level: attacker_level as c_int,
            attacker_attack: attacker_attack as c_int,
            defender_defense: defender_defense as c_int,
            move_power: move_power as c_int,
            type_effectiveness,
            critical_hit: if is_critical { 1 } else { 0 },
            weather_modifier,
        };

        let damage = unsafe { calculate_damage_fast(&params) };
        
        if damage < 0 {
            Err(GameError::Calculation("伤害计算出错".to_string()))
        } else {
            Ok(damage as u16)
        }
    }
}

pub struct PathfindingEngine {
    grid_ptr: *mut c_void,
    width: i32,
    height: i32,
}

impl PathfindingEngine {
    pub fn new(width: i32, height: i32) -> GameResult<Self> {
        let grid_ptr = unsafe { pathfinding_create_grid(width, height) };
        
        if grid_ptr.is_null() {
            return Err(GameError::Memory("无法创建寻路网格".to_string()));
        }

        Ok(Self {
            grid_ptr,
            width,
            height,
        })
    }

    pub fn set_obstacle(&mut self, x: i32, y: i32, is_obstacle: bool) -> GameResult<()> {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return Err(GameError::InvalidInput("坐标超出范围".to_string()));
        }

        unsafe {
            pathfinding_set_obstacle(
                self.grid_ptr,
                x,
                y,
                if is_obstacle { 1 } else { 0 },
            );
        }

        Ok(())
    }

    pub fn find_path(&self, start: Vec2, end: Vec2) -> GameResult<Vec<Vec2>> {
        let mut path_nodes = vec![CPathNode {
            x: 0,
            y: 0,
            g_cost: 0.0,
            h_cost: 0.0,
            f_cost: 0.0,
        }; 1000]; // 最大路径长度

        let path_length = unsafe {
            pathfinding_find_path(
                self.grid_ptr,
                start.x as c_int,
                start.y as c_int,
                end.x as c_int,
                end.y as c_int,
                path_nodes.as_mut_ptr(),
                path_nodes.len() as c_int,
            )
        };

        if path_length < 0 {
            return Err(GameError::Pathfinding("寻路失败".to_string()));
        }

        let mut result = Vec::new();
        for i in 0..(path_length as usize) {
            let node = &path_nodes[i];
            result.push(Vec2::new(node.x as f32, node.y as f32));
        }

        Ok(result)
    }
}

impl Drop for PathfindingEngine {
    fn drop(&mut self) {
        if !self.grid_ptr.is_null() {
            unsafe {
                pathfinding_destroy_grid(self.grid_ptr);
            }
        }
    }
}

pub struct AudioEngine {
    engine_ptr: *mut c_void,
    sample_rate: i32,
    buffer_size: i32,
}

impl AudioEngine {
    pub fn new(sample_rate: i32, buffer_size: i32) -> GameResult<Self> {
        let engine_ptr = unsafe { audio_engine_create(sample_rate, buffer_size) };
        
        if engine_ptr.is_null() {
            return Err(GameError::Audio("无法创建音频引擎".to_string()));
        }

        Ok(Self {
            engine_ptr,
            sample_rate,
            buffer_size,
        })
    }

    pub fn apply_3d_audio(&self, listener_pos: Vec3, source_pos: Vec3) -> GameResult<(f32, f32)> {
        let params = C3DAudioParams {
            listener_pos: CVec3 { x: listener_pos.x, y: listener_pos.y, z: listener_pos.z },
            listener_forward: CVec3 { x: 0.0, y: 0.0, z: -1.0 },
            listener_up: CVec3 { x: 0.0, y: 1.0, z: 0.0 },
            source_pos: CVec3 { x: source_pos.x, y: source_pos.y, z: source_pos.z },
            max_distance: 100.0,
            rolloff_factor: 1.0,
        };

        let mut gain = 0.0;
        let mut pan = 0.0;

        let result = unsafe {
            audio_calculate_3d_position(&params, &mut gain, &mut pan)
        };

        if result != 0 {
            return Err(GameError::Audio("3D音频计算失败".to_string()));
        }

        Ok((gain, pan))
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        if !self.engine_ptr.is_null() {
            unsafe {
                audio_engine_destroy(self.engine_ptr);
            }
        }
    }
}

// 安全的C字符串转换
pub fn to_c_string(s: &str) -> GameResult<CString> {
    CString::new(s).map_err(|_| GameError::InvalidInput("字符串包含空字节".to_string()))
}

pub fn from_c_string(ptr: *const c_char) -> GameResult<String> {
    if ptr.is_null() {
        return Err(GameError::NullPointer("C字符串指针为空".to_string()));
    }

    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| GameError::InvalidInput("无效的UTF-8字符串".to_string()))
    }
}

// 内存管理辅助函数
pub unsafe fn free_c_memory(ptr: *mut c_void) {
    if !ptr.is_null() {
        libc::free(ptr);
    }
}

// 错误代码转换
pub fn c_result_to_game_result(code: c_int, operation: &str) -> GameResult<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(GameError::NativeOperation(format!("{} 失败，错误代码: {}", operation, code)))
    }
}

// SIMD优化检查
pub fn check_simd_support() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::x86_64::*;
        unsafe {
            // 检查AVX2支持
            let cpuid = __cpuid(7);
            (cpuid.ebx & (1 << 5)) != 0
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

// 批量数据处理包装器
pub struct BatchProcessor {
    batch_size: usize,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }

    pub fn process_damage_calculations(&self, calculations: &[(CDamageCalcParams)]) -> GameResult<Vec<u16>> {
        let mut results = Vec::with_capacity(calculations.len());
        let mut temp_results = vec![0; self.batch_size];

        for chunk in calculations.chunks(self.batch_size) {
            let chunk_size = chunk.len();
            
            unsafe {
                calculate_damage_range(
                    chunk.as_ptr(),
                    temp_results.as_mut_ptr(),
                    chunk_size as c_int,
                );
            }

            for i in 0..chunk_size {
                results.push(temp_results[i] as u16);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_operations() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        let result = MathEngine::vec2_add_safe(a, b);
        
        assert_eq!(result.x, 4.0);
        assert_eq!(result.y, 6.0);
    }

    #[test]
    fn test_pathfinding_engine() {
        let mut engine = PathfindingEngine::new(10, 10).unwrap();
        engine.set_obstacle(5, 5, true).unwrap();
        
        let path = engine.find_path(Vec2::new(0.0, 0.0), Vec2::new(9.0, 9.0));
        assert!(path.is_ok());
    }

    #[test]
    fn test_damage_calculation() {
        let damage = MathEngine::calculate_pokemon_damage(
            50, 100, 80, 90, 1.0, false, 1.0
        ).unwrap();
        
        assert!(damage > 0);
    }

    #[test]
    fn test_simd_support() {
        let has_simd = check_simd_support();
        println!("SIMD support: {}", has_simd);
    }
}