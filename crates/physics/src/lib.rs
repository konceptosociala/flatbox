use flatbox_core::math::glm;
use serde::{Serialize, Deserialize};
use rapier3d::prelude::*;

use crate::error::PhysicsError;

pub mod error;

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct PhysicsBodyHandle(RigidBodyHandle, ColliderHandle);

impl PhysicsBodyHandle {
    pub fn new() -> Self {
        PhysicsBodyHandle::default()
    }
}

/// Collection for physics simulations
#[derive(Serialize, Deserialize)]
pub struct PhysicsHandler {
    rigidbody_set: RigidBodySet,
    collider_set: ColliderSet,
    
    #[cfg(feature = "render")]
    #[serde(skip_serializing, skip_deserializing)]
    pub render_pipeline: DebugRenderPipeline,

    #[serde(skip_serializing, skip_deserializing)]
    pub physics_pipeline: PhysicsPipeline,
    
    pub gravity: glm::Vec3,
    pub integration_parameters: IntegrationParameters,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhaseMultiSap,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub physics_hooks: (),
    pub event_handler: (),
}

impl PhysicsHandler {
    /// Create new [`PhysicsHandler`] instance
    ///
    /// It's usually not necessary, as it's part of the [`Flatbox`] application
    pub fn new() -> Self {
        PhysicsHandler::default()
    }
    
    /// Create new physical instance from [`RigidBody`] and [`Collider`]
    pub fn new_instance(&mut self, rigidbody: RigidBody, collider: Collider) -> PhysicsBodyHandle {
        let rigidbody = self.rigidbody_set.insert(rigidbody);
        let collider = self.collider_set.insert_with_parent(collider, rigidbody, &mut self.rigidbody_set);
        
        PhysicsBodyHandle(rigidbody, collider)
    }
    
    /// Get RigidBody from set
    pub fn rigidbody(&self, handle: PhysicsBodyHandle) -> Result<&RigidBody, PhysicsError> {
        self.rigidbody_set.get(handle.0).ok_or(PhysicsError::InvalidRigidBody)
    }
    
    /// Get Collider from set
    pub fn collider(&self, handle: PhysicsBodyHandle) -> Result<&Collider, PhysicsError> {
        self.collider_set.get(handle.1).ok_or(PhysicsError::InvalidCollider)
    }
    
    /// Mutably get RigidBody from set
    pub fn rigidbody_mut(&mut self, handle: PhysicsBodyHandle) -> Result<&mut RigidBody, PhysicsError> {
        self.rigidbody_set.get_mut(handle.0).ok_or(PhysicsError::InvalidRigidBody)
    }
    
    /// Mutably get Collider from set
    pub fn collider_mut(&mut self, handle: PhysicsBodyHandle) -> Result<&mut Collider, PhysicsError> {
        self.collider_set.get_mut(handle.1).ok_or(PhysicsError::InvalidCollider)
    }
    
    /// Destroy physical instance and return attached RigidBody and Collider as tuple
    ///
    /// ```rust
    /// let (rigidbody, collider) = physics_handler.remove_instance(handle).unwrap();
    /// ```
    ///
    pub fn remove_instance(&mut self, handle: PhysicsBodyHandle) -> Result<(RigidBody, Collider), PhysicsError> {
        let rigidbody = self.rigidbody_set.remove(
            handle.0,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            false,
        ).ok_or(PhysicsError::InvalidRigidBody)?;
        
        let collider = self.collider_set.remove(
            handle.1,
            &mut self.island_manager,
            &mut self.rigidbody_set,
            false,
        ).ok_or(PhysicsError::InvalidCollider)?;
        
        Ok((rigidbody, collider))
    }
    
    /// Set physics debug rendering style and mode
    #[cfg(feature = "render")]
    pub fn set_debug_renderer(&mut self, style: DebugRenderStyle, mode: DebugRenderMode){
        self.render_pipeline = DebugRenderPipeline::new(style, mode);
    }
    
    #[cfg(feature = "render")]
    pub(crate) fn debug_render(&mut self, renderer: &mut Renderer){
        self.render_pipeline.render(
            renderer,
            &self.rigidbody_set,
            &self.collider_set,
            &self.impulse_joint_set,
            &self.multibody_joint_set,
            &self.narrow_phase,
        );
    }
    
    /// Does a physical simulations step. Run in a loop
    pub fn step(&mut self){
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbody_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            &self.event_handler,
        )
    }
}

impl Default for PhysicsHandler {
    fn default() -> Self {
        PhysicsHandler {
            rigidbody_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            
            #[cfg(feature = "render")]
            render_pipeline: DebugRenderPipeline::new(
                DebugRenderStyle::default(),
                DebugRenderMode::COLLIDER_SHAPES,
            ),
            physics_pipeline: PhysicsPipeline::new(),
            
            gravity: vector![0.0, -2.0, 0.0],
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseMultiSap::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }
}
