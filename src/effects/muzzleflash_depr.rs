use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_hanabi::{EffectAsset, SizeOverLifetimeModifier, Spawner, *};

pub fn setup_muzzleflash(
    mut effects: ResMut<Assets<EffectAsset>>,
    ass: Res<AssetServer>,
    mut commands: Commands,
) {
    let shape = ass.load("effects/muzzleflash.png");
    // dbg!(&shape);
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec2::splat(0.5));
    gradient.add_key(1.0, Vec2::splat(0.8));

    let mut color_gradient1 = Gradient::new();
    // color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    // color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    // color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    // color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    color_gradient1.add_key(0.0, Vec4::splat(1.));
    color_gradient1.add_key(1.0, Vec4::splat(1.));

    color_gradient1.add_key(1.0, Vec4::splat(1.0));
    let lifetime = 0.3;
    let spawner = Spawner::rate(4.0.into());
    let muzzle_flash = effects.add(
        EffectAsset {
            name: "MuzzleFlash".into(),
            capacity: 32768,
            spawner,
            // spawner: Spawner::burst(2500.0.into(), 2.0.into()),
            ..default()
        }
        .init(PositionSphereModifier {
            speed: 0.0.into(),
            dimension: ShapeDimension::Surface,
            ..Default::default()
        })
        .init(ParticleLifetimeModifier { lifetime })
        // .render(SizeOverLifetimeModifier { gradient })
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(BillboardModifier {})
        .render(ParticleTextureModifier { texture: shape })
        .render(SizeOverLifetimeModifier { gradient: gradient }),
    );
    let mut bullet_gradient = Gradient::new();
    bullet_gradient.add_key(0.0, Vec2::new(2.0, 0.1));
    bullet_gradient.add_key(1.0, Vec2::new(2.0, 0.1));

    let mut bullet_gradient_color = Gradient::new();
    bullet_gradient_color.add_key(0.0, Vec4::splat(1.));
    bullet_gradient_color.add_key(1.0, Vec4::new(1.0, 0.9, 0., 1.));

    let bullet_trail = effects.add(
        EffectAsset {
            name: "BulletTrail".into(),
            capacity: 32768,
            spawner,
            ..default()
        }
        .init(PositionCone3dModifier {
            speed: 6.0.into(),
            dimension: ShapeDimension::Volume,
            height: 1.0,
            base_radius: 0.01,
            top_radius: 0.01,
            ..default()
        })
        // .init(Modifier)
        .init(ParticleLifetimeModifier { lifetime })
        // .render(SizeOverLifetimeModifier { gradient })
        // .render(Rotation)
        .render(BillboardModifier {})
        .render(ColorOverLifetimeModifier {
            gradient: bullet_gradient_color,
        })
        .render(SizeOverLifetimeModifier {
            gradient: bullet_gradient,
        }),
    );
    let mut transform = Transform::from_translation(Vec3::splat(2.));
    transform.rotate_local_axis(Vec3::Z, PI / 2.);
    // dbg!(&transform);
    transform.rotate_axis(Vec3::Y, PI / 6.);
    commands
        .spawn(SpatialBundle {
            transform,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("MuzzleFlash"),
                    ParticleEffectBundle {
                        effect: ParticleEffect::new(muzzle_flash.clone()),
                        // transform,
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    // parent.spawn((
                    //     Name::new("MuzzleFlash2"),
                    //     ParticleEffectBundle {
                    //         effect: ParticleEffect::new(muzzle_flash),
                    //         transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.)),
                    //         ..Default::default()
                    //     },
                    // ));
                });

            parent.spawn((
                Name::new("BulletTrail"),
                ParticleEffectBundle {
                    effect: ParticleEffect::new(bullet_trail),
                    // transform,
                    ..Default::default()
                },
            ));
        });
}
