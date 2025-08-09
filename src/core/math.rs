// 数学工具系统
// 开发心理：高性能数学运算库，结合Rust安全性和C++性能优势
// 使用SIMD指令优化向量和矩阵运算，为游戏提供强大的数学基础

use crate::core::error::{GameError, Result};
use bevy::math::{Vec2, Vec3, Vec4, Mat4, Quat};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub, Mul, Div};

// 重新导出常用数学类型
pub use bevy::math::{
    Vec2 as Vector2,
    Vec3 as Vector3, 
    Vec4 as Vector4,
    Mat4 as Matrix4,
    Quat as Quaternion,
};

// 2D点结构
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

// 3D点结构
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// 矩形结构
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// 圆形结构
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point2,
    pub radius: f32,
}

// 包围盒结构
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    pub min: Point3,
    pub max: Point3,
}

// 数学常量
pub mod constants {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const E: f32 = std::f32::consts::E;
    pub const SQRT_2: f32 = std::f32::consts::SQRT_2;
    pub const EPSILON: f32 = 1e-6;
    
    // 角度转换
    pub const DEG_TO_RAD: f32 = PI / 180.0;
    pub const RAD_TO_DEG: f32 = 180.0 / PI;
}

// 初始化数学系统
pub fn init() -> Result<()> {
    log::info!("初始化数学系统");
    
    // 检查SIMD支持 - 跨平台
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("sse") {
            log::info!("检测到SSE支持");
        }
        if is_x86_feature_detected!("sse2") {
            log::info!("检测到SSE2支持");
        }
        if is_x86_feature_detected!("sse4.1") {
            log::info!("检测到SSE4.1支持");
        }
        if is_x86_feature_detected!("avx") {
            log::info!("检测到AVX支持");
        }
        if is_x86_feature_detected!("avx2") {
            log::info!("检测到AVX2支持");
        }
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        log::info!("ARM64架构，使用NEON SIMD");
    }
    
    Ok(())
}

// 清理数学系统
pub fn cleanup() {
    log::info!("清理数学系统");
}

impl Point2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn distance_to(self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    
    pub fn distance_squared_to(self, other: Self) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
    
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
    
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len < constants::EPSILON {
            Self::ZERO
        } else {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        }
    }
    
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }
    
    pub fn rotate(self, angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }
    
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: lerp(self.x, other.x, t),
            y: lerp(self.y, other.y, t),
        }
    }
}

impl Point3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn distance_to(self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
    }
    
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len < constants::EPSILON {
            Self::ZERO
        } else {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        }
    }
    
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: lerp(self.x, other.x, t),
            y: lerp(self.y, other.y, t),
            z: lerp(self.z, other.z, t),
        }
    }
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn contains_point(self, point: Point2) -> bool {
        point.x >= self.x && point.x <= self.x + self.width &&
        point.y >= self.y && point.y <= self.y + self.height
    }
    
    pub fn intersects(self, other: Self) -> bool {
        !(self.x + self.width < other.x ||
          other.x + other.width < self.x ||
          self.y + self.height < other.y ||
          other.y + other.height < self.y)
    }
    
    pub fn area(self) -> f32 {
        self.width * self.height
    }
    
    pub fn center(self) -> Point2 {
        Point2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

impl Circle {
    pub fn new(center: Point2, radius: f32) -> Self {
        Self { center, radius }
    }
    
    pub fn contains_point(self, point: Point2) -> bool {
        self.center.distance_squared_to(point) <= self.radius * self.radius
    }
    
    pub fn intersects_circle(self, other: Self) -> bool {
        let distance = self.center.distance_to(other.center);
        distance <= self.radius + other.radius
    }
    
    pub fn intersects_rect(self, rect: Rect) -> bool {
        let closest = Point2::new(
            clamp(self.center.x, rect.x, rect.x + rect.width),
            clamp(self.center.y, rect.y, rect.y + rect.height),
        );
        self.contains_point(closest)
    }
    
    pub fn area(self) -> f32 {
        constants::PI * self.radius * self.radius
    }
}

impl AABB {
    pub fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }
    
    pub fn from_points(points: &[Point3]) -> Self {
        if points.is_empty() {
            return Self::new(Point3::ZERO, Point3::ZERO);
        }
        
        let mut min = points[0];
        let mut max = points[0];
        
        for &point in points.iter().skip(1) {
            if point.x < min.x { min.x = point.x; }
            if point.y < min.y { min.y = point.y; }
            if point.z < min.z { min.z = point.z; }
            
            if point.x > max.x { max.x = point.x; }
            if point.y > max.y { max.y = point.y; }
            if point.z > max.z { max.z = point.z; }
        }
        
        Self::new(min, max)
    }
    
    pub fn contains_point(self, point: Point3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    
    pub fn intersects(self, other: Self) -> bool {
        !(self.max.x < other.min.x || other.max.x < self.min.x ||
          self.max.y < other.min.y || other.max.y < self.min.y ||
          self.max.z < other.min.z || other.max.z < self.min.z)
    }
    
    pub fn center(self) -> Point3 {
        Point3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }
    
    pub fn size(self) -> Point3 {
        Point3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
}

// 运算符重载实现
impl Add for Point2 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Sub for Point2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Mul<f32> for Point2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self::Output {
        Self { x: self.x * scalar, y: self.y * scalar }
    }
}

impl Add for Point3 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
}

impl Sub for Point3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

impl Mul<f32> for Point3 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self::Output {
        Self { x: self.x * scalar, y: self.y * scalar, z: self.z * scalar }
    }
}

// 实用函数
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn map_range(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = (value - from_min) / (from_max - from_min);
    lerp(to_min, to_max, t)
}

pub fn smoothstep(t: f32) -> f32 {
    let t = clamp(t, 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn smootherstep(t: f32) -> f32 {
    let t = clamp(t, 0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * constants::DEG_TO_RAD
}

pub fn radians_to_degrees(radians: f32) -> f32 {
    radians * constants::RAD_TO_DEG
}

pub fn approximately_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < constants::EPSILON
}

// C++互操作函数声明
extern "C" {
    // 从native模块导入的高性能数学函数
    fn native_vector_dot(a_x: f32, a_y: f32, a_z: f32, b_x: f32, b_y: f32, b_z: f32) -> f32;
    fn native_matrix_multiply(a: *const f32, b: *const f32, result: *mut f32);
    fn native_fast_sqrt(value: f32) -> f32;
    fn native_fast_sin(angle: f32) -> f32;
    fn native_fast_cos(angle: f32) -> f32;
}

// 高性能数学函数包装器
pub mod fast {
    use super::*;
    
    pub fn vector_dot(a: Point3, b: Point3) -> f32 {
        unsafe {
            native_vector_dot(a.x, a.y, a.z, b.x, b.y, b.z)
        }
    }
    
    pub fn sqrt(value: f32) -> f32 {
        unsafe {
            native_fast_sqrt(value)
        }
    }
    
    pub fn sin(angle: f32) -> f32 {
        unsafe {
            native_fast_sin(angle)
        }
    }
    
    pub fn cos(angle: f32) -> f32 {
        unsafe {
            native_fast_cos(angle)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_point2_operations() {
        let p1 = Point2::new(1.0, 2.0);
        let p2 = Point2::new(3.0, 4.0);
        
        let sum = p1 + p2;
        assert_eq!(sum, Point2::new(4.0, 6.0));
        
        let distance = p1.distance_to(p2);
        assert!((distance - 2.828427).abs() < 0.001);
    }
    
    #[test]
    fn test_point3_operations() {
        let p1 = Point3::new(1.0, 0.0, 0.0);
        let p2 = Point3::new(0.0, 1.0, 0.0);
        
        let cross = p1.cross(p2);
        assert_eq!(cross, Point3::new(0.0, 0.0, 1.0));
        
        let dot = p1.dot(p2);
        assert_eq!(dot, 0.0);
    }
    
    #[test]
    fn test_rect_operations() {
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Point2::new(5.0, 5.0);
        
        assert!(rect.contains_point(point));
        assert_eq!(rect.center(), Point2::new(5.0, 5.0));
        assert_eq!(rect.area(), 100.0);
    }
    
    #[test]
    fn test_circle_operations() {
        let circle = Circle::new(Point2::ZERO, 5.0);
        let point = Point2::new(3.0, 4.0);
        
        assert!(circle.contains_point(point));
        assert!((circle.area() - 78.54).abs() < 0.01);
    }
    
    #[test]
    fn test_math_functions() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
        
        let radians = degrees_to_radians(90.0);
        assert!((radians - constants::PI / 2.0).abs() < 0.001);
    }
}