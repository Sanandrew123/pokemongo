// SIMD数学优化模块头文件
// 开发心理：高性能数学计算是游戏引擎的基础，使用SIMD指令集加速
// 支持伤害计算、向量运算、矩阵变换等核心数学操作

#ifndef SIMD_OPERATIONS_H
#define SIMD_OPERATIONS_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

// 平台检测和SIMD支持
#if defined(_MSC_VER)
    #include <intrin.h>
    #define ALIGN(n) __declspec(align(n))
#elif defined(__GNUC__)
    #include <x86intrin.h>
    #define ALIGN(n) __attribute__((aligned(n)))
#endif

// 数据结构定义
typedef struct {
    float x, y, z, w;
} ALIGN(16) Vec4f;

typedef struct {
    float x, y, z;
} ALIGN(16) Vec3f;

typedef struct {
    float x, y;
} ALIGN(8) Vec2f;

typedef struct {
    float m[4][4];
} ALIGN(64) Matrix4f;

// 基础数学函数
float fast_sqrt(float x);
float fast_inv_sqrt(float x);
float fast_sin(float x);
float fast_cos(float x);

// 向量运算 (SIMD优化)
Vec4f vec4_add(const Vec4f* a, const Vec4f* b);
Vec4f vec4_sub(const Vec4f* a, const Vec4f* b);
Vec4f vec4_mul(const Vec4f* a, const Vec4f* b);
Vec4f vec4_scale(const Vec4f* v, float scale);
float vec4_dot(const Vec4f* a, const Vec4f* b);
float vec4_length(const Vec4f* v);
Vec4f vec4_normalize(const Vec4f* v);
Vec4f vec4_cross(const Vec4f* a, const Vec4f* b);

Vec3f vec3_add(const Vec3f* a, const Vec3f* b);
Vec3f vec3_sub(const Vec3f* a, const Vec3f* b);
Vec3f vec3_mul(const Vec3f* a, const Vec3f* b);
Vec3f vec3_scale(const Vec3f* v, float scale);
float vec3_dot(const Vec3f* a, const Vec3f* b);
float vec3_length(const Vec3f* v);
Vec3f vec3_normalize(const Vec3f* v);
Vec3f vec3_cross(const Vec3f* a, const Vec3f* b);

Vec2f vec2_add(const Vec2f* a, const Vec2f* b);
Vec2f vec2_sub(const Vec2f* a, const Vec2f* b);
float vec2_dot(const Vec2f* a, const Vec2f* b);
float vec2_length(const Vec2f* v);
Vec2f vec2_normalize(const Vec2f* v);

// 矩阵运算 (SIMD优化)
Matrix4f matrix4_identity(void);
Matrix4f matrix4_multiply(const Matrix4f* a, const Matrix4f* b);
Matrix4f matrix4_transpose(const Matrix4f* m);
Matrix4f matrix4_inverse(const Matrix4f* m);
Vec4f matrix4_transform_vec4(const Matrix4f* m, const Vec4f* v);
Vec3f matrix4_transform_vec3(const Matrix4f* m, const Vec3f* v);

// 变换矩阵生成
Matrix4f matrix4_translation(float x, float y, float z);
Matrix4f matrix4_rotation_x(float angle);
Matrix4f matrix4_rotation_y(float angle);
Matrix4f matrix4_rotation_z(float angle);
Matrix4f matrix4_rotation_euler(float x, float y, float z);
Matrix4f matrix4_scale(float x, float y, float z);
Matrix4f matrix4_perspective(float fovy, float aspect, float near, float far);
Matrix4f matrix4_orthographic(float left, float right, float bottom, float top, float near, float far);
Matrix4f matrix4_look_at(const Vec3f* eye, const Vec3f* center, const Vec3f* up);

// 游戏特定的数学函数
float pokemon_damage_calculation(
    float attack_power,
    float defense,
    uint8_t attacker_level,
    uint8_t defender_level,
    float type_effectiveness,
    float critical_multiplier,
    float random_factor
);

float pokemon_accuracy_calculation(
    uint8_t move_accuracy,
    int8_t accuracy_stage,
    int8_t evasion_stage,
    float ability_modifier
);

float pokemon_speed_calculation(
    uint16_t base_speed,
    uint8_t individual_value,
    uint16_t effort_value,
    uint8_t level,
    float nature_modifier,
    float status_modifier
);

// 批量向量运算 (数组处理)
void vec4_array_add(const Vec4f* a, const Vec4f* b, Vec4f* result, int count);
void vec4_array_scale(const Vec4f* input, float scale, Vec4f* result, int count);
void vec3_array_transform(const Matrix4f* matrix, const Vec3f* input, Vec3f* result, int count);

// 插值函数
float lerp(float a, float b, float t);
Vec3f vec3_lerp(const Vec3f* a, const Vec3f* b, float t);
Vec4f vec4_lerp(const Vec4f* a, const Vec4f* b, float t);
Vec3f vec3_slerp(const Vec3f* a, const Vec3f* b, float t);

// 噪声函数 (地形生成用)
float perlin_noise_2d(float x, float y, int seed);
float perlin_noise_3d(float x, float y, float z, int seed);
float simplex_noise_2d(float x, float y);
float simplex_noise_3d(float x, float y, float z);

// 碰撞检测用数学函数
typedef struct {
    Vec3f center;
    float radius;
} Sphere;

typedef struct {
    Vec3f min;
    Vec3f max;
} AABB;

typedef struct {
    Vec3f origin;
    Vec3f direction;
} Ray;

int sphere_sphere_intersect(const Sphere* a, const Sphere* b);
int sphere_aabb_intersect(const Sphere* sphere, const AABB* box);
int ray_sphere_intersect(const Ray* ray, const Sphere* sphere, float* distance);
int ray_aabb_intersect(const Ray* ray, const AABB* box, float* distance);

// 性能测试函数
void benchmark_simd_operations(int iterations);
void benchmark_matrix_operations(int iterations);
void benchmark_vector_operations(int iterations);

// CPU特性检测
int has_sse2_support(void);
int has_sse3_support(void);
int has_sse4_support(void);
int has_avx_support(void);
int has_avx2_support(void);
int has_fma_support(void);

// 内存对齐工具
void* aligned_malloc(size_t size, size_t alignment);
void aligned_free(void* ptr);

#ifdef __cplusplus
}
#endif

#endif // SIMD_OPERATIONS_H