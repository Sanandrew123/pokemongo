// SIMD数学优化模块实现
// 开发心理：利用现代CPU的向量指令集实现高性能数学运算
// 针对游戏引擎的常见操作进行专门优化

#include "simd_operations.h"
#include <immintrin.h>
#include <cmath>
#include <cstring>
#include <chrono>
#include <iostream>

// 快速数学函数实现
float fast_sqrt(float x) {
    #ifdef __AVX__
    __m128 temp = _mm_set_ss(x);
    temp = _mm_sqrt_ss(temp);
    return _mm_cvtss_f32(temp);
    #else
    // 使用牛顿法快速平方根
    union { float f; uint32_t i; } conv = { x };
    conv.i = 0x5f3759df - (conv.i >> 1);
    conv.f *= 1.5f - (x * 0.5f * conv.f * conv.f);
    return x * conv.f;
    #endif
}

float fast_inv_sqrt(float x) {
    // 著名的Quake III快速反平方根
    union { float f; uint32_t i; } conv = { x };
    conv.i = 0x5f3759df - (conv.i >> 1);
    conv.f *= 1.5f - (x * 0.5f * conv.f * conv.f);
    conv.f *= 1.5f - (x * 0.5f * conv.f * conv.f); // 第二次迭代提高精度
    return conv.f;
}

float fast_sin(float x) {
    // 使用泰勒级数近似
    const float PI = 3.14159265359f;
    x = fmodf(x + PI, 2 * PI) - PI; // 规范化到[-π, π]
    
    const float x2 = x * x;
    return x * (1.0f - x2 / 6.0f + x2 * x2 / 120.0f);
}

float fast_cos(float x) {
    return fast_sin(x + 1.57079632679f); // cos(x) = sin(x + π/2)
}

// Vec4 SIMD优化运算
Vec4f vec4_add(const Vec4f* a, const Vec4f* b) {
    #ifdef __SSE__
    __m128 va = _mm_load_ps((float*)a);
    __m128 vb = _mm_load_ps((float*)b);
    __m128 result = _mm_add_ps(va, vb);
    
    Vec4f ret;
    _mm_store_ps((float*)&ret, result);
    return ret;
    #else
    return { a->x + b->x, a->y + b->y, a->z + b->z, a->w + b->w };
    #endif
}

Vec4f vec4_sub(const Vec4f* a, const Vec4f* b) {
    #ifdef __SSE__
    __m128 va = _mm_load_ps((float*)a);
    __m128 vb = _mm_load_ps((float*)b);
    __m128 result = _mm_sub_ps(va, vb);
    
    Vec4f ret;
    _mm_store_ps((float*)&ret, result);
    return ret;
    #else
    return { a->x - b->x, a->y - b->y, a->z - b->z, a->w - b->w };
    #endif
}

Vec4f vec4_mul(const Vec4f* a, const Vec4f* b) {
    #ifdef __SSE__
    __m128 va = _mm_load_ps((float*)a);
    __m128 vb = _mm_load_ps((float*)b);
    __m128 result = _mm_mul_ps(va, vb);
    
    Vec4f ret;
    _mm_store_ps((float*)&ret, result);
    return ret;
    #else
    return { a->x * b->x, a->y * b->y, a->z * b->z, a->w * b->w };
    #endif
}

Vec4f vec4_scale(const Vec4f* v, float scale) {
    #ifdef __SSE__
    __m128 vv = _mm_load_ps((float*)v);
    __m128 vs = _mm_set1_ps(scale);
    __m128 result = _mm_mul_ps(vv, vs);
    
    Vec4f ret;
    _mm_store_ps((float*)&ret, result);
    return ret;
    #else
    return { v->x * scale, v->y * scale, v->z * scale, v->w * scale };
    #endif
}

float vec4_dot(const Vec4f* a, const Vec4f* b) {
    #ifdef __SSE4_1__
    __m128 va = _mm_load_ps((float*)a);
    __m128 vb = _mm_load_ps((float*)b);
    __m128 result = _mm_dp_ps(va, vb, 0xFF);
    return _mm_cvtss_f32(result);
    #elif defined(__SSE__)
    __m128 va = _mm_load_ps((float*)a);
    __m128 vb = _mm_load_ps((float*)b);
    __m128 mul = _mm_mul_ps(va, vb);
    __m128 sum = _mm_hadd_ps(mul, mul);
    sum = _mm_hadd_ps(sum, sum);
    return _mm_cvtss_f32(sum);
    #else
    return a->x * b->x + a->y * b->y + a->z * b->z + a->w * b->w;
    #endif
}

float vec4_length(const Vec4f* v) {
    return fast_sqrt(vec4_dot(v, v));
}

Vec4f vec4_normalize(const Vec4f* v) {
    float len = vec4_length(v);
    if (len > 1e-6f) {
        float inv_len = 1.0f / len;
        return vec4_scale(v, inv_len);
    }
    return { 0.0f, 0.0f, 0.0f, 0.0f };
}

Vec4f vec4_cross(const Vec4f* a, const Vec4f* b) {
    // 4D向量的叉积不是标准定义，这里实现3D叉积并保持w分量
    return {
        a->y * b->z - a->z * b->y,
        a->z * b->x - a->x * b->z,
        a->x * b->y - a->y * b->x,
        0.0f
    };
}

// Vec3运算实现
Vec3f vec3_add(const Vec3f* a, const Vec3f* b) {
    return { a->x + b->x, a->y + b->y, a->z + b->z };
}

Vec3f vec3_sub(const Vec3f* a, const Vec3f* b) {
    return { a->x - b->x, a->y - b->y, a->z - b->z };
}

Vec3f vec3_mul(const Vec3f* a, const Vec3f* b) {
    return { a->x * b->x, a->y * b->y, a->z * b->z };
}

Vec3f vec3_scale(const Vec3f* v, float scale) {
    return { v->x * scale, v->y * scale, v->z * scale };
}

float vec3_dot(const Vec3f* a, const Vec3f* b) {
    return a->x * b->x + a->y * b->y + a->z * b->z;
}

float vec3_length(const Vec3f* v) {
    return fast_sqrt(vec3_dot(v, v));
}

Vec3f vec3_normalize(const Vec3f* v) {
    float len = vec3_length(v);
    if (len > 1e-6f) {
        float inv_len = 1.0f / len;
        return vec3_scale(v, inv_len);
    }
    return { 0.0f, 0.0f, 0.0f };
}

Vec3f vec3_cross(const Vec3f* a, const Vec3f* b) {
    return {
        a->y * b->z - a->z * b->y,
        a->z * b->x - a->x * b->z,
        a->x * b->y - a->y * b->x
    };
}

// Vec2运算实现
Vec2f vec2_add(const Vec2f* a, const Vec2f* b) {
    return { a->x + b->x, a->y + b->y };
}

Vec2f vec2_sub(const Vec2f* a, const Vec2f* b) {
    return { a->x - b->x, a->y - b->y };
}

float vec2_dot(const Vec2f* a, const Vec2f* b) {
    return a->x * b->x + a->y * b->y;
}

float vec2_length(const Vec2f* v) {
    return fast_sqrt(vec2_dot(v, v));
}

Vec2f vec2_normalize(const Vec2f* v) {
    float len = vec2_length(v);
    if (len > 1e-6f) {
        float inv_len = 1.0f / len;
        return { v->x * inv_len, v->y * inv_len };
    }
    return { 0.0f, 0.0f };
}

// 矩阵运算实现
Matrix4f matrix4_identity(void) {
    Matrix4f m = {0};
    m.m[0][0] = m.m[1][1] = m.m[2][2] = m.m[3][3] = 1.0f;
    return m;
}

Matrix4f matrix4_multiply(const Matrix4f* a, const Matrix4f* b) {
    Matrix4f result = {0};
    
    #ifdef __AVX__
    // AVX优化的矩阵乘法
    for (int i = 0; i < 4; i++) {
        __m256 row1 = _mm256_broadcast_ss(&a->m[i][0]);
        __m256 row2 = _mm256_broadcast_ss(&a->m[i][1]);
        __m256 row3 = _mm256_broadcast_ss(&a->m[i][2]);
        __m256 row4 = _mm256_broadcast_ss(&a->m[i][3]);
        
        __m256 col1 = _mm256_load_ps(&b->m[0][0]);
        __m256 col2 = _mm256_load_ps(&b->m[1][0]);
        __m256 col3 = _mm256_load_ps(&b->m[2][0]);
        __m256 col4 = _mm256_load_ps(&b->m[3][0]);
        
        __m256 res = _mm256_mul_ps(row1, col1);
        res = _mm256_fmadd_ps(row2, col2, res);
        res = _mm256_fmadd_ps(row3, col3, res);
        res = _mm256_fmadd_ps(row4, col4, res);
        
        _mm256_store_ps(&result.m[i][0], res);
    }
    #else
    // 标准矩阵乘法
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            result.m[i][j] = a->m[i][0] * b->m[0][j] +
                           a->m[i][1] * b->m[1][j] +
                           a->m[i][2] * b->m[2][j] +
                           a->m[i][3] * b->m[3][j];
        }
    }
    #endif
    
    return result;
}

Matrix4f matrix4_transpose(const Matrix4f* m) {
    Matrix4f result;
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            result.m[i][j] = m->m[j][i];
        }
    }
    return result;
}

Vec4f matrix4_transform_vec4(const Matrix4f* m, const Vec4f* v) {
    #ifdef __SSE__
    __m128 vv = _mm_load_ps((float*)v);
    __m128 col0 = _mm_load_ps(&m->m[0][0]);
    __m128 col1 = _mm_load_ps(&m->m[1][0]);
    __m128 col2 = _mm_load_ps(&m->m[2][0]);
    __m128 col3 = _mm_load_ps(&m->m[3][0]);
    
    __m128 result = _mm_mul_ps(_mm_shuffle_ps(vv, vv, 0x00), col0);
    result = _mm_fmadd_ps(_mm_shuffle_ps(vv, vv, 0x55), col1, result);
    result = _mm_fmadd_ps(_mm_shuffle_ps(vv, vv, 0xAA), col2, result);
    result = _mm_fmadd_ps(_mm_shuffle_ps(vv, vv, 0xFF), col3, result);
    
    Vec4f ret;
    _mm_store_ps((float*)&ret, result);
    return ret;
    #else
    return {
        m->m[0][0] * v->x + m->m[0][1] * v->y + m->m[0][2] * v->z + m->m[0][3] * v->w,
        m->m[1][0] * v->x + m->m[1][1] * v->y + m->m[1][2] * v->z + m->m[1][3] * v->w,
        m->m[2][0] * v->x + m->m[2][1] * v->y + m->m[2][2] * v->z + m->m[2][3] * v->w,
        m->m[3][0] * v->x + m->m[3][1] * v->y + m->m[3][2] * v->z + m->m[3][3] * v->w
    };
    #endif
}

Vec3f matrix4_transform_vec3(const Matrix4f* m, const Vec3f* v) {
    Vec4f v4 = { v->x, v->y, v->z, 1.0f };
    Vec4f result = matrix4_transform_vec4(m, &v4);
    return { result.x, result.y, result.z };
}

// 变换矩阵生成
Matrix4f matrix4_translation(float x, float y, float z) {
    Matrix4f m = matrix4_identity();
    m.m[0][3] = x;
    m.m[1][3] = y;
    m.m[2][3] = z;
    return m;
}

Matrix4f matrix4_rotation_z(float angle) {
    Matrix4f m = matrix4_identity();
    float c = fast_cos(angle);
    float s = fast_sin(angle);
    
    m.m[0][0] = c; m.m[0][1] = -s;
    m.m[1][0] = s; m.m[1][1] = c;
    return m;
}

Matrix4f matrix4_scale(float x, float y, float z) {
    Matrix4f m = matrix4_identity();
    m.m[0][0] = x;
    m.m[1][1] = y;
    m.m[2][2] = z;
    return m;
}

// 宝可梦专用数学函数
float pokemon_damage_calculation(
    float attack_power,
    float defense,
    uint8_t attacker_level,
    uint8_t defender_level,
    float type_effectiveness,
    float critical_multiplier,
    float random_factor
) {
    // 基础伤害公式 (简化版宝可梦伤害计算)
    float level_factor = (2.0f * attacker_level + 10.0f) / 250.0f;
    float base_damage = level_factor * attack_power / defense + 2.0f;
    
    // 应用各种乘数
    float final_damage = base_damage * type_effectiveness * critical_multiplier * random_factor;
    
    return fmaxf(1.0f, final_damage); // 至少造成1点伤害
}

float pokemon_accuracy_calculation(
    uint8_t move_accuracy,
    int8_t accuracy_stage,
    int8_t evasion_stage,
    float ability_modifier
) {
    // 基础命中率
    float base_accuracy = move_accuracy / 100.0f;
    
    // 能力变化影响
    float stage_multiplier = 1.0f;
    int net_stage = accuracy_stage - evasion_stage;
    
    if (net_stage > 0) {
        stage_multiplier = (3.0f + net_stage) / 3.0f;
    } else if (net_stage < 0) {
        stage_multiplier = 3.0f / (3.0f - net_stage);
    }
    
    return fminf(1.0f, base_accuracy * stage_multiplier * ability_modifier);
}

float pokemon_speed_calculation(
    uint16_t base_speed,
    uint8_t individual_value,
    uint16_t effort_value,
    uint8_t level,
    float nature_modifier,
    float status_modifier
) {
    // 标准宝可梦速度计算公式
    float iv_ev_component = 2.0f * base_speed + individual_value + effort_value / 4.0f;
    float level_component = iv_ev_component * level / 100.0f + 5.0f;
    
    return level_component * nature_modifier * status_modifier;
}

// 批量运算
void vec4_array_add(const Vec4f* a, const Vec4f* b, Vec4f* result, int count) {
    #ifdef __AVX__
    for (int i = 0; i < count; i += 2) {
        __m256 va = _mm256_load_ps((float*)(a + i));
        __m256 vb = _mm256_load_ps((float*)(b + i));
        __m256 res = _mm256_add_ps(va, vb);
        _mm256_store_ps((float*)(result + i), res);
    }
    #else
    for (int i = 0; i < count; i++) {
        result[i] = vec4_add(a + i, b + i);
    }
    #endif
}

void vec4_array_scale(const Vec4f* input, float scale, Vec4f* result, int count) {
    #ifdef __AVX__
    __m256 vs = _mm256_set1_ps(scale);
    for (int i = 0; i < count; i += 2) {
        __m256 vi = _mm256_load_ps((float*)(input + i));
        __m256 res = _mm256_mul_ps(vi, vs);
        _mm256_store_ps((float*)(result + i), res);
    }
    #else
    for (int i = 0; i < count; i++) {
        result[i] = vec4_scale(input + i, scale);
    }
    #endif
}

// 插值函数
float lerp(float a, float b, float t) {
    return a + t * (b - a);
}

Vec3f vec3_lerp(const Vec3f* a, const Vec3f* b, float t) {
    return {
        lerp(a->x, b->x, t),
        lerp(a->y, b->y, t),
        lerp(a->z, b->z, t)
    };
}

// 碰撞检测
int sphere_sphere_intersect(const Sphere* a, const Sphere* b) {
    Vec3f diff = vec3_sub(&a->center, &b->center);
    float dist_sq = vec3_dot(&diff, &diff);
    float radius_sum = a->radius + b->radius;
    return dist_sq <= radius_sum * radius_sum;
}

int sphere_aabb_intersect(const Sphere* sphere, const AABB* box) {
    Vec3f closest = {
        fmaxf(box->min.x, fminf(sphere->center.x, box->max.x)),
        fmaxf(box->min.y, fminf(sphere->center.y, box->max.y)),
        fmaxf(box->min.z, fminf(sphere->center.z, box->max.z))
    };
    
    Vec3f diff = vec3_sub(&sphere->center, &closest);
    float dist_sq = vec3_dot(&diff, &diff);
    return dist_sq <= sphere->radius * sphere->radius;
}

// CPU特性检测
int has_sse2_support(void) {
    #ifdef _MSC_VER
    int cpuinfo[4];
    __cpuid(cpuinfo, 1);
    return (cpuinfo[3] & (1 << 26)) != 0;
    #else
    return __builtin_cpu_supports("sse2");
    #endif
}

int has_avx_support(void) {
    #ifdef _MSC_VER
    int cpuinfo[4];
    __cpuid(cpuinfo, 1);
    return (cpuinfo[2] & (1 << 28)) != 0;
    #else
    return __builtin_cpu_supports("avx");
    #endif
}

int has_avx2_support(void) {
    #ifdef _MSC_VER
    int cpuinfo[4];
    __cpuidex(cpuinfo, 7, 0);
    return (cpuinfo[1] & (1 << 5)) != 0;
    #else
    return __builtin_cpu_supports("avx2");
    #endif
}

// 性能测试
void benchmark_simd_operations(int iterations) {
    Vec4f a = { 1.0f, 2.0f, 3.0f, 4.0f };
    Vec4f b = { 5.0f, 6.0f, 7.0f, 8.0f };
    Vec4f result;
    
    auto start = std::chrono::high_resolution_clock::now();
    
    for (int i = 0; i < iterations; i++) {
        result = vec4_add(&a, &b);
        result = vec4_mul(&result, &b);
        result = vec4_normalize(&result);
    }
    
    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::microseconds>(end - start);
    
    std::cout << "SIMD operations: " << iterations << " iterations in " 
              << duration.count() << " microseconds\n";
}