//! Headless regression coverage against the production Ahoy + Avian schedules.
#[cfg(test)]
mod tests {
    use avian3d::prelude::*;
    use bevy::prelude::*;
    use bevy_ahoy::{AirJumpState, DashState, WallRunState, prelude::*};
    use bevy_enhanced_input::{EnhancedInputPlugin, prelude::*};
    use std::time::Duration;

    enum SceneKind {
        Ground,
        Air,
        Wall,
        Water,
    }
    fn scene(kind: SceneKind) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            bevy::input::InputPlugin,
            PhysicsPlugins::default(),
            EnhancedInputPlugin,
            AhoyPlugins::default(),
        ))
        .init_asset::<Mesh>()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f64(1.0 / 60.0),
        ));
        if matches!(kind, SceneKind::Ground | SceneKind::Water) {
            app.world_mut().spawn((
                RigidBody::Static,
                Collider::cuboid(100., 1., 100.),
                Transform::from_xyz(0., -0.5, 0.),
            ));
        }
        if matches!(kind, SceneKind::Wall) {
            app.world_mut().spawn((
                RigidBody::Static,
                Collider::cuboid(1., 20., 100.),
                Transform::from_xyz(1., 5., 0.),
            ));
        }
        if matches!(kind, SceneKind::Water) {
            app.world_mut().spawn((
                RigidBody::Static,
                Water { speed: 0.4 },
                Collider::cuboid(20., 20., 20.),
                Transform::from_xyz(0., 5., 0.),
            ));
        }
        let player = app
            .world_mut()
            .spawn((
                CharacterController::default(),
                Dashing::default(),
                AirJumping::default(),
                WallRunning::default(),
                Collider::capsule(0.3, 1.2),
                Transform::from_xyz(0., 5., 0.),
            ))
            .id();
        app.finish();
        app.cleanup();
        step(&mut app, 3);
        (app, player)
    }
    fn step(app: &mut App, count: usize) {
        for _ in 0..count {
            app.update();
        }
    }
    fn fire<A: InputAction>(app: &mut App, player: Entity, value: A::Output) {
        app.world_mut().trigger(Fire::<A> {
            context: player,
            action: Entity::PLACEHOLDER,
            value,
            state: TriggerState::Fired,
            fired_secs: 0.,
            elapsed_secs: 0.,
        });
    }
    fn forward(app: &mut App, player: Entity, count: usize) {
        for _ in 0..count {
            fire::<Movement>(app, player, Vec2::Y);
            app.update();
        }
    }
    fn velocity(app: &App, player: Entity) -> Vec3 {
        app.world().get::<LinearVelocity>(player).unwrap().0
    }
    fn start_wall_run(app: &mut App, player: Entity) {
        app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec3::new(0., 0., -10.);
        forward(app, player, 3);
        let run = app.world().get::<WallRunState>(player).unwrap();
        assert!(
            run.progress.is_some(),
            "real wall query must admit a wall run: {run:?}"
        );
        assert!(run.progress.as_ref().unwrap().normal.x < -0.9);
    }

    #[test]
    fn floor_landing_has_no_false_water_contact() {
        let (mut app, player) = scene(SceneKind::Ground);
        step(&mut app, 240);
        let position = app.world().get::<Transform>(player).unwrap().translation;
        assert!(
            position.is_finite() && position.y > 0.8 && position.y < 1.3,
            "{position:?}"
        );
        assert!(
            app.world()
                .get::<CharacterControllerState>(player)
                .unwrap()
                .grounded
                .is_some()
        );
        assert_eq!(
            app.world().get::<WaterState>(player).unwrap().level,
            WaterLevel::None
        );
    }
    #[test]
    fn water_volume_reports_head_and_configured_speed() {
        let (mut app, player) = scene(SceneKind::Water);
        step(&mut app, 120);
        let water = app.world().get::<WaterState>(player).unwrap();
        assert_eq!(water.level, WaterLevel::Head);
        assert_eq!(water.speed, 0.4);
    }
    #[test]
    fn dash_moves_forward_and_rejects_input_during_cooldown() {
        let (mut app, player) = scene(SceneKind::Ground);
        step(&mut app, 120);
        let start = app.world().get::<Transform>(player).unwrap().translation;
        fire::<Dash>(&mut app, player, true);
        step(&mut app, 1);
        assert!(
            app.world()
                .get::<DashState>(player)
                .unwrap()
                .progress
                .is_some()
        );
        assert!(velocity(&app, player).with_y(0.).length() > 30.);
        step(&mut app, 14);
        let end = app.world().get::<Transform>(player).unwrap().translation;
        let distance = (end - start).with_y(0.).length();
        assert!(distance > 6. && distance < 10., "dash distance={distance}");
        assert!(
            app.world()
                .get::<DashState>(player)
                .unwrap()
                .progress
                .is_none()
        );
        fire::<Dash>(&mut app, player, true);
        step(&mut app, 1);
        assert!(
            app.world()
                .get::<DashState>(player)
                .unwrap()
                .progress
                .is_none()
        );
    }
    #[test]
    fn air_jump_lifts_falling_character_and_enforces_budget() {
        let (mut app, player) = scene(SceneKind::Air);
        assert!(velocity(&app, player).y < 0.);
        fire::<Jump>(&mut app, player, true);
        step(&mut app, 1);
        let first = velocity(&app, player).y;
        assert!(first > 0.);
        assert_eq!(app.world().get::<AirJumpState>(player).unwrap().used, 1);
        fire::<Jump>(&mut app, player, true);
        step(&mut app, 1);
        assert_eq!(app.world().get::<AirJumpState>(player).unwrap().used, 1);
        assert!(velocity(&app, player).y < first);
    }
    #[test]
    fn landing_resets_air_jump_and_air_dash_budgets() {
        let (mut app, player) = scene(SceneKind::Ground);
        fire::<Jump>(&mut app, player, true);
        step(&mut app, 1);
        assert_eq!(app.world().get::<AirJumpState>(player).unwrap().used, 1);
        fire::<Dash>(&mut app, player, true);
        step(&mut app, 1);
        assert_eq!(
            app.world()
                .get::<DashState>(player)
                .unwrap()
                .air_dashes_used,
            1
        );
        step(&mut app, 240);
        assert!(
            app.world()
                .get::<CharacterControllerState>(player)
                .unwrap()
                .grounded
                .is_some()
        );
        assert_eq!(app.world().get::<AirJumpState>(player).unwrap().used, 0);
        assert_eq!(
            app.world()
                .get::<DashState>(player)
                .unwrap()
                .air_dashes_used,
            0
        );
    }
    #[test]
    fn wall_run_requires_a_real_wall_and_preserves_wall_normal() {
        let (mut control, player) = scene(SceneKind::Air);
        control
            .world_mut()
            .get_mut::<LinearVelocity>(player)
            .unwrap()
            .0 = Vec3::new(0., 0., -10.);
        forward(&mut control, player, 3);
        assert!(
            control
                .world()
                .get::<WallRunState>(player)
                .unwrap()
                .progress
                .is_none()
        );
        let (mut app, player) = scene(SceneKind::Wall);
        start_wall_run(&mut app, player);
    }
    #[test]
    fn wall_jump_leaves_wall_and_produces_upward_outward_velocity() {
        let (mut app, player) = scene(SceneKind::Wall);
        start_wall_run(&mut app, player);
        fire::<Jump>(&mut app, player, true);
        forward(&mut app, player, 1);
        assert!(
            app.world()
                .get::<WallRunState>(player)
                .unwrap()
                .progress
                .is_none()
        );
        let v = velocity(&app, player);
        assert!(v.x < -1. && v.y > 0., "wall jump velocity={v:?}");
    }
    #[test]
    fn wall_run_times_out_and_cannot_reattach_until_reset() {
        let (mut app, player) = scene(SceneKind::Wall);
        app.world_mut()
            .get_mut::<WallRunning>(player)
            .unwrap()
            .max_duration = Duration::from_millis(100);
        start_wall_run(&mut app, player);
        forward(&mut app, player, 20);
        let state = app.world().get::<WallRunState>(player).unwrap();
        assert!(state.progress.is_none() && state.exhausted, "{state:?}");
    }
    #[cfg(feature = "serialize")]
    #[test]
    fn movement_configuration_round_trips_with_serialize_feature() {
        let dash = Dashing::default();
        assert_eq!(
            serde_json::from_str::<Dashing>(&serde_json::to_string(&dash).unwrap()).unwrap(),
            dash
        );
        let jump = AirJumping::default();
        assert_eq!(
            serde_json::from_str::<AirJumping>(&serde_json::to_string(&jump).unwrap()).unwrap(),
            jump
        );
        let wall = WallRunning::default();
        assert_eq!(
            serde_json::from_str::<WallRunning>(&serde_json::to_string(&wall).unwrap()).unwrap(),
            wall
        );
    }
}
