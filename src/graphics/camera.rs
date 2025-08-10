// 相机系统
// 开发心理：相机是3D世界与2D屏幕的桥梁，需要灵活的视角控制和平滑的过渡
// 设计原则：数学精确性、多投影支持、平滑插值、可序列化状态

use crate::core::{GameError, Result};
use serde::{Deserialize, Serialize};
use log::{debug, warn};

// 投影类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProjectionType {
    Perspective {
        fovy: f32,   // Y轴视角（弧度）
        aspect: f32, // 宽高比
        near: f32,   // 近裁剪平面
        far: f32,    // 远裁剪平面
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
    OrthographicCentered {
        width: f32,
        height: f32,
        near: f32,
        far: f32,
    },
}

impl Default for ProjectionType {
    fn default() -> Self {
        Self::Perspective {
            fovy: 60.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

// 相机类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraType {
    FirstPerson,  // 第一人称
    ThirdPerson,  // 第三人称
    Orbital,      // 轨道相机
    Fixed,        // 固定相机
    Follow,       // 跟随相机
    Cutscene,     // 过场动画相机
    Debug,        // 调试相机
}

// 相机运动约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConstraints {
    pub min_distance: Option<f32>,
    pub max_distance: Option<f32>,
    pub min_pitch: Option<f32>, // 俯仰角限制
    pub max_pitch: Option<f32>,
    pub min_yaw: Option<f32>,   // 偏航角限制
    pub max_yaw: Option<f32>,
    pub bounds: Option<BoundingBox>, // 位置边界
}

impl Default for CameraConstraints {
    fn default() -> Self {
        Self {
            min_distance: Some(1.0),
            max_distance: Some(100.0),
            min_pitch: Some(-89.0_f32.to_radians()),
            max_pitch: Some(89.0_f32.to_radians()),
            min_yaw: None,
            max_yaw: None,
            bounds: None,
        }
    }
}

// 边界盒
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl BoundingBox {
    pub fn new(min: glam::Vec3, max: glam::Vec3) -> Self {
        Self { min, max }
    }
    
    pub fn contains(&self, point: &glam::Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    
    pub fn clamp_point(&self, point: &glam::Vec3) -> glam::Vec3 {
        glam::Vec3::new(
            point.x.clamp(self.min.x, self.max.x),
            point.y.clamp(self.min.y, self.max.y),
            point.z.clamp(self.min.z, self.max.z),
        )
    }
}

// 相机状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    // 基本变换
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: f32,
    
    // 目标和方向（用于某些相机类型）
    pub target: Option<glam::Vec3>,
    pub up: glam::Vec3,
    
    // 投影设置
    pub projection: ProjectionType,
    
    // 相机类型和约束
    pub camera_type: CameraType,
    pub constraints: CameraConstraints,
    
    // 缓存的矩阵
    view_matrix: glam::Mat4,
    projection_matrix: glam::Mat4,
    view_projection_matrix: glam::Mat4,
    
    // 状态标志
    matrices_dirty: bool,
    
    // 运动参数
    pub move_speed: f32,
    pub rotation_speed: f32,
    pub zoom_speed: f32,
    pub smooth_factor: f32, // 平滑插值因子
    
    // 第三人称相机参数
    pub distance: f32,
    pub pitch: f32,  // 俯仰角（弧度）
    pub yaw: f32,    // 偏航角（弧度）
    
    // 震动效果
    shake_amplitude: f32,
    shake_duration: f32,
    shake_time: f32,
    shake_frequency: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(ProjectionType::default())
    }
}

impl Camera {
    pub fn new(projection: ProjectionType) -> Self {
        let mut camera = Self {
            position: glam::Vec3::new(0.0, 0.0, 5.0),
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
            target: None,
            up: glam::Vec3::Y,
            projection,
            camera_type: CameraType::FirstPerson,
            constraints: CameraConstraints::default(),
            view_matrix: glam::Mat4::IDENTITY,
            projection_matrix: glam::Mat4::IDENTITY,
            view_projection_matrix: glam::Mat4::IDENTITY,
            matrices_dirty: true,
            move_speed: 5.0,
            rotation_speed: 2.0,
            zoom_speed: 1.0,
            smooth_factor: 10.0,
            distance: 5.0,
            pitch: 0.0,
            yaw: 0.0,
            shake_amplitude: 0.0,
            shake_duration: 0.0,
            shake_time: 0.0,
            shake_frequency: 10.0,
        };
        
        camera.update_matrices();
        camera
    }
    
    // 创建透视投影相机
    pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::new(ProjectionType::Perspective { fovy, aspect, near, far })
    }
    
    // 创建正交投影相机
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self::new(ProjectionType::Orthographic { left, right, bottom, top, near, far })
    }
    
    // 创建2D相机（居中正交投影）
    pub fn ortho_2d(width: f32, height: f32) -> Self {
        Self::new(ProjectionType::OrthographicCentered { 
            width, 
            height, 
            near: -1000.0, 
            far: 1000.0 
        })
    }
    
    // 设置位置
    pub fn set_position(&mut self, position: glam::Vec3) {
        self.position = self.apply_position_constraints(position);
        self.matrices_dirty = true;
    }
    
    // 移动相机
    pub fn translate(&mut self, delta: glam::Vec3) {
        self.set_position(self.position + delta);
    }
    
    // 设置旋转
    pub fn set_rotation(&mut self, rotation: glam::Quat) {
        self.rotation = rotation;
        self.matrices_dirty = true;
    }
    
    // 旋转相机
    pub fn rotate(&mut self, delta: glam::Quat) {
        self.set_rotation(self.rotation * delta);
    }
    
    // 根据欧拉角设置旋转
    pub fn set_euler_angles(&mut self, pitch: f32, yaw: f32, roll: f32) {
        let constrained_pitch = self.apply_pitch_constraint(pitch);
        let constrained_yaw = self.apply_yaw_constraint(yaw);
        
        self.pitch = constrained_pitch;
        self.yaw = constrained_yaw;
        
        self.rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, constrained_yaw, constrained_pitch, roll);
        self.matrices_dirty = true;
    }
    
    // 添加欧拉角旋转
    pub fn add_euler_angles(&mut self, delta_pitch: f32, delta_yaw: f32, delta_roll: f32) {
        let (current_yaw, current_pitch, current_roll) = self.rotation.to_euler(glam::EulerRot::YXZ);
        self.set_euler_angles(
            current_pitch + delta_pitch,
            current_yaw + delta_yaw,
            current_roll + delta_roll
        );
    }
    
    // 看向目标
    pub fn look_at(&mut self, target: glam::Vec3, up: glam::Vec3) {
        self.target = Some(target);
        self.up = up;
        
        let direction = (target - self.position).normalize();
        if direction.length_squared() > 0.001 {
            self.rotation = glam::Quat::from_rotation_arc(glam::Vec3::NEG_Z, direction);
            self.matrices_dirty = true;
        }
    }
    
    // 设置第三人称相机参数
    pub fn set_third_person(&mut self, target: glam::Vec3, distance: f32, pitch: f32, yaw: f32) {
        self.camera_type = CameraType::ThirdPerson;
        self.target = Some(target);
        self.distance = self.apply_distance_constraint(distance);
        self.pitch = self.apply_pitch_constraint(pitch);
        self.yaw = self.apply_yaw_constraint(yaw);
        
        self.update_third_person_position();
    }
    
    // 更新第三人称相机位置
    fn update_third_person_position(&mut self) {
        if let Some(target) = self.target {
            let offset = glam::Vec3::new(
                self.distance * self.pitch.cos() * self.yaw.sin(),
                self.distance * self.pitch.sin(),
                self.distance * self.pitch.cos() * self.yaw.cos(),
            );
            
            self.position = target + offset;
            self.look_at(target, glam::Vec3::Y);
        }
    }
    
    // 设置投影参数
    pub fn set_projection(&mut self, projection: ProjectionType) {
        self.projection = projection;
        self.matrices_dirty = true;
    }
    
    // 更新宽高比
    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        match &mut self.projection {
            ProjectionType::Perspective { aspect: ref mut a, .. } => {
                *a = aspect;
                self.matrices_dirty = true;
            },
            ProjectionType::OrthographicCentered { width, height, .. } => {
                *height = *width / aspect;
                self.matrices_dirty = true;
            },
            _ => {}
        }
    }
    
    // 缩放（调整FOV或正交投影大小）
    pub fn zoom(&mut self, delta: f32) {
        match &mut self.projection {
            ProjectionType::Perspective { fovy, .. } => {
                *fovy = (*fovy + delta * self.zoom_speed * 0.1).clamp(
                    10.0_f32.to_radians(),
                    120.0_f32.to_radians()
                );
                self.matrices_dirty = true;
            },
            ProjectionType::OrthographicCentered { width, height, .. } => {
                let scale_factor = 1.0 + delta * self.zoom_speed * 0.1;
                *width *= scale_factor;
                *height *= scale_factor;
                self.matrices_dirty = true;
            },
            _ => {}
        }
    }
    
    // 更新相机（每帧调用）
    pub fn update(&mut self, delta_time: f32) {
        // 更新震动效果
        if self.shake_duration > 0.0 {
            self.shake_time += delta_time;
            if self.shake_time >= self.shake_duration {
                self.shake_amplitude = 0.0;
                self.shake_duration = 0.0;
                self.shake_time = 0.0;
            }
        }
        
        // 第三人称相机位置更新
        if self.camera_type == CameraType::ThirdPerson {
            self.update_third_person_position();
        }
        
        // 更新矩阵
        if self.matrices_dirty {
            self.update_matrices();
        }
    }
    
    // 平滑移动到目标位置
    pub fn smooth_move_to(&mut self, target_position: glam::Vec3, delta_time: f32) {
        let t = (self.smooth_factor * delta_time).min(1.0);
        let new_position = self.position.lerp(target_position, t);
        self.set_position(new_position);
    }
    
    // 平滑旋转到目标方向
    pub fn smooth_rotate_to(&mut self, target_rotation: glam::Quat, delta_time: f32) {
        let t = (self.smooth_factor * delta_time).min(1.0);
        let new_rotation = self.rotation.slerp(target_rotation, t);
        self.set_rotation(new_rotation);
    }
    
    // 添加震动效果
    pub fn add_shake(&mut self, amplitude: f32, duration: f32, frequency: f32) {
        self.shake_amplitude = amplitude;
        self.shake_duration = duration;
        self.shake_frequency = frequency;
        self.shake_time = 0.0;
    }
    
    // 获取震动偏移
    fn get_shake_offset(&self) -> glam::Vec3 {
        if self.shake_amplitude <= 0.0 {
            return glam::Vec3::ZERO;
        }
        
        let progress = self.shake_time / self.shake_duration;
        let decay = 1.0 - progress;
        let current_amplitude = self.shake_amplitude * decay;
        
        let shake_x = (self.shake_time * self.shake_frequency).sin() * current_amplitude;
        let shake_y = (self.shake_time * self.shake_frequency * 1.3).cos() * current_amplitude;
        let shake_z = (self.shake_time * self.shake_frequency * 0.7).sin() * current_amplitude * 0.5;
        
        glam::Vec3::new(shake_x, shake_y, shake_z)
    }
    
    // 获取相机方向向量
    pub fn get_forward(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::NEG_Z
    }
    
    pub fn get_right(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::X
    }
    
    pub fn get_up(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::Y
    }
    
    // 获取视图矩阵
    pub fn get_view_matrix(&self) -> glam::Mat4 {
        self.view_matrix
    }
    
    // 获取投影矩阵
    pub fn get_projection_matrix(&self) -> glam::Mat4 {
        self.projection_matrix
    }
    
    // 获取视图投影矩阵
    pub fn get_view_projection_matrix(&self) -> glam::Mat4 {
        self.view_projection_matrix
    }
    
    // 屏幕坐标转世界射线
    pub fn screen_point_to_ray(&self, screen_pos: glam::Vec2, screen_size: glam::Vec2) -> Ray {
        // 将屏幕坐标转换为NDC（-1到1）
        let ndc_x = (2.0 * screen_pos.x) / screen_size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / screen_size.y;
        
        // 投影矩阵的逆矩阵
        let inv_projection = self.projection_matrix.inverse();
        let inv_view = self.view_matrix.inverse();
        
        // NDC空间的近平面点
        let near_point = glam::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_point = glam::Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        
        // 转换到世界空间
        let world_near = inv_view * inv_projection * near_point;
        let world_far = inv_view * inv_projection * far_point;
        
        let world_near = world_near.xyz() / world_near.w;
        let world_far = world_far.xyz() / world_far.w;
        
        Ray {
            origin: world_near,
            direction: (world_far - world_near).normalize(),
        }
    }
    
    // 世界坐标转屏幕坐标
    pub fn world_to_screen(&self, world_pos: glam::Vec3, screen_size: glam::Vec2) -> Option<glam::Vec2> {
        let clip_pos = self.view_projection_matrix * glam::Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
        
        if clip_pos.w <= 0.0 {
            return None; // 在相机后面
        }
        
        let ndc = clip_pos.xyz() / clip_pos.w;
        
        // NDC转屏幕坐标
        let screen_x = (ndc.x + 1.0) * 0.5 * screen_size.x;
        let screen_y = (1.0 - ndc.y) * 0.5 * screen_size.y;
        
        if ndc.z >= -1.0 && ndc.z <= 1.0 {
            Some(glam::Vec2::new(screen_x, screen_y))
        } else {
            None // 超出深度范围
        }
    }
    
    // 获取视锥体平面（用于裁剪）
    pub fn get_frustum_planes(&self) -> [Plane; 6] {
        let vp = self.view_projection_matrix;
        
        [
            // Left
            Plane::from_coefficients(vp.w_axis + vp.x_axis),
            // Right
            Plane::from_coefficients(vp.w_axis - vp.x_axis),
            // Bottom
            Plane::from_coefficients(vp.w_axis + vp.y_axis),
            // Top
            Plane::from_coefficients(vp.w_axis - vp.y_axis),
            // Near
            Plane::from_coefficients(vp.w_axis + vp.z_axis),
            // Far
            Plane::from_coefficients(vp.w_axis - vp.z_axis),
        ]
    }
    
    // 更新矩阵
    fn update_matrices(&mut self) {
        // 计算有效位置（包括震动偏移）
        let effective_position = self.position + self.get_shake_offset();
        
        // 计算视图矩阵
        self.view_matrix = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(1.0 / self.scale),
            self.rotation.inverse(),
            -effective_position,
        );
        
        // 计算投影矩阵
        self.projection_matrix = match self.projection {
            ProjectionType::Perspective { fovy, aspect, near, far } => {
                glam::Mat4::perspective_rh(fovy, aspect, near, far)
            },
            ProjectionType::Orthographic { left, right, bottom, top, near, far } => {
                glam::Mat4::orthographic_rh(left, right, bottom, top, near, far)
            },
            ProjectionType::OrthographicCentered { width, height, near, far } => {
                let half_width = width * 0.5;
                let half_height = height * 0.5;
                glam::Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, near, far)
            },
        };
        
        // 计算视图投影矩阵
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;
        
        self.matrices_dirty = false;
    }
    
    // 约束应用函数
    fn apply_position_constraints(&self, position: glam::Vec3) -> glam::Vec3 {
        if let Some(ref bounds) = self.constraints.bounds {
            bounds.clamp_point(&position)
        } else {
            position
        }
    }
    
    fn apply_distance_constraint(&self, distance: f32) -> f32 {
        let mut result = distance;
        if let Some(min_dist) = self.constraints.min_distance {
            result = result.max(min_dist);
        }
        if let Some(max_dist) = self.constraints.max_distance {
            result = result.min(max_dist);
        }
        result
    }
    
    fn apply_pitch_constraint(&self, pitch: f32) -> f32 {
        let mut result = pitch;
        if let Some(min_pitch) = self.constraints.min_pitch {
            result = result.max(min_pitch);
        }
        if let Some(max_pitch) = self.constraints.max_pitch {
            result = result.min(max_pitch);
        }
        result
    }
    
    fn apply_yaw_constraint(&self, yaw: f32) -> f32 {
        let mut result = yaw;
        if let Some(min_yaw) = self.constraints.min_yaw {
            result = result.max(min_yaw);
        }
        if let Some(max_yaw) = self.constraints.max_yaw {
            result = result.min(max_yaw);
        }
        result
    }
}

// 射线（用于鼠标选择）
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: glam::Vec3,
    pub direction: glam::Vec3,
}

impl Ray {
    pub fn new(origin: glam::Vec3, direction: glam::Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
    
    pub fn point_at(&self, t: f32) -> glam::Vec3 {
        self.origin + self.direction * t
    }
    
    // 射线与球体相交检测
    pub fn intersect_sphere(&self, center: glam::Vec3, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        let a = self.direction.length_squared();
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.length_squared() - radius * radius;
        
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.0 { Some(t) } else { None }
        }
    }
    
    // 射线与平面相交检测
    pub fn intersect_plane(&self, plane: &Plane) -> Option<f32> {
        let denom = plane.normal.dot(self.direction);
        if denom.abs() < 1e-6 {
            return None; // 平行
        }
        
        let t = (plane.distance - plane.normal.dot(self.origin)) / denom;
        if t >= 0.0 { Some(t) } else { None }
    }
}

// 平面
#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: glam::Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: glam::Vec3, distance: f32) -> Self {
        Self {
            normal: normal.normalize(),
            distance,
        }
    }
    
    pub fn from_point_normal(point: glam::Vec3, normal: glam::Vec3) -> Self {
        let normal = normal.normalize();
        Self {
            normal,
            distance: normal.dot(point),
        }
    }
    
    pub fn from_coefficients(coefficients: glam::Vec4) -> Self {
        let normal = coefficients.xyz();
        let length = normal.length();
        
        if length > 0.0 {
            Self {
                normal: normal / length,
                distance: -coefficients.w / length,
            }
        } else {
            Self {
                normal: glam::Vec3::Y,
                distance: 0.0,
            }
        }
    }
    
    pub fn distance_to_point(&self, point: glam::Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }
    
    pub fn is_point_in_front(&self, point: glam::Vec3) -> bool {
        self.distance_to_point(point) >= 0.0
    }
}

// 相机控制器
pub struct CameraController {
    pub camera: Camera,
    pub input_enabled: bool,
    
    // 输入状态
    mouse_sensitivity: f32,
    keyboard_speed: f32,
    mouse_last_pos: Option<glam::Vec2>,
    
    // 轨道相机参数
    orbit_target: glam::Vec3,
    orbit_distance: f32,
    orbit_speed: f32,
    
    // 跟随相机参数
    follow_target: Option<glam::Vec3>,
    follow_offset: glam::Vec3,
    follow_smooth: f32,
}

impl CameraController {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            input_enabled: true,
            mouse_sensitivity: 0.002,
            keyboard_speed: 5.0,
            mouse_last_pos: None,
            orbit_target: glam::Vec3::ZERO,
            orbit_distance: 5.0,
            orbit_speed: 2.0,
            follow_target: None,
            follow_offset: glam::Vec3::new(0.0, 2.0, -5.0),
            follow_smooth: 5.0,
        }
    }
    
    // 处理鼠标移动
    pub fn handle_mouse_move(&mut self, mouse_pos: glam::Vec2) {
        if !self.input_enabled {
            return;
        }
        
        if let Some(last_pos) = self.mouse_last_pos {
            let delta = mouse_pos - last_pos;
            
            match self.camera.camera_type {
                CameraType::FirstPerson => {
                    self.camera.add_euler_angles(
                        -delta.y * self.mouse_sensitivity,
                        -delta.x * self.mouse_sensitivity,
                        0.0
                    );
                },
                CameraType::Orbital => {
                    self.camera.yaw -= delta.x * self.mouse_sensitivity * self.orbit_speed;
                    self.camera.pitch -= delta.y * self.mouse_sensitivity * self.orbit_speed;
                    self.update_orbital_camera();
                },
                _ => {}
            }
        }
        
        self.mouse_last_pos = Some(mouse_pos);
    }
    
    // 处理鼠标滚轮
    pub fn handle_mouse_scroll(&mut self, delta: f32) {
        if !self.input_enabled {
            return;
        }
        
        match self.camera.camera_type {
            CameraType::FirstPerson | CameraType::Debug => {
                self.camera.zoom(-delta * 0.1);
            },
            CameraType::Orbital => {
                self.orbit_distance = (self.orbit_distance - delta * 0.5).max(0.5);
                self.update_orbital_camera();
            },
            _ => {}
        }
    }
    
    // 处理键盘输入
    pub fn handle_keyboard(&mut self, forward: f32, right: f32, up: f32, delta_time: f32) {
        if !self.input_enabled {
            return;
        }
        
        let movement_speed = self.keyboard_speed * delta_time;
        
        match self.camera.camera_type {
            CameraType::FirstPerson | CameraType::Debug => {
                let forward_dir = self.camera.get_forward();
                let right_dir = self.camera.get_right();
                let up_dir = self.camera.get_up();
                
                let movement = forward_dir * forward * movement_speed +
                              right_dir * right * movement_speed +
                              up_dir * up * movement_speed;
                
                self.camera.translate(movement);
            },
            _ => {}
        }
    }
    
    // 设置轨道相机目标
    pub fn set_orbit_target(&mut self, target: glam::Vec3) {
        self.camera.camera_type = CameraType::Orbital;
        self.orbit_target = target;
        self.update_orbital_camera();
    }
    
    // 设置跟随目标
    pub fn set_follow_target(&mut self, target: Option<glam::Vec3>) {
        self.follow_target = target;
        if target.is_some() {
            self.camera.camera_type = CameraType::Follow;
        }
    }
    
    // 更新控制器
    pub fn update(&mut self, delta_time: f32) {
        // 跟随相机逻辑
        if self.camera.camera_type == CameraType::Follow {
            if let Some(target) = self.follow_target {
                let desired_position = target + self.follow_offset;
                self.camera.smooth_move_to(desired_position, delta_time * self.follow_smooth);
                self.camera.look_at(target, glam::Vec3::Y);
            }
        }
        
        self.camera.update(delta_time);
    }
    
    // 更新轨道相机位置
    fn update_orbital_camera(&mut self) {
        let offset = glam::Vec3::new(
            self.orbit_distance * self.camera.pitch.cos() * self.camera.yaw.sin(),
            self.orbit_distance * self.camera.pitch.sin(),
            self.orbit_distance * self.camera.pitch.cos() * self.camera.yaw.cos(),
        );
        
        self.camera.set_position(self.orbit_target + offset);
        self.camera.look_at(self.orbit_target, glam::Vec3::Y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_camera_creation() {
        let camera = Camera::perspective(60.0_f32.to_radians(), 16.0/9.0, 0.1, 1000.0);
        assert!(matches!(camera.projection, ProjectionType::Perspective { .. }));
        assert_eq!(camera.position, glam::Vec3::new(0.0, 0.0, 5.0));
    }
    
    #[test]
    fn test_camera_movement() {
        let mut camera = Camera::default();
        let initial_pos = camera.position;
        
        camera.translate(glam::Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(camera.position, initial_pos + glam::Vec3::X);
    }
    
    #[test]
    fn test_ray_sphere_intersection() {
        let ray = Ray::new(glam::Vec3::ZERO, glam::Vec3::Z);
        let intersection = ray.intersect_sphere(glam::Vec3::new(0.0, 0.0, 5.0), 1.0);
        assert!(intersection.is_some());
        assert!((intersection.unwrap() - 4.0).abs() < 0.001);
    }
    
    #[test]
    fn test_bounding_box_contains() {
        let bbox = BoundingBox::new(glam::Vec3::new(-1.0, -1.0, -1.0), glam::Vec3::new(1.0, 1.0, 1.0));
        assert!(bbox.contains(&glam::Vec3::ZERO));
        assert!(!bbox.contains(&glam::Vec3::new(2.0, 0.0, 0.0)));
    }
}