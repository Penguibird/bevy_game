use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, Friction};
use rand::{rngs::ThreadRng, thread_rng, Rng};


pub const MAP_SIZE: f32 = 200.;

// A possible improvement would be to use some noise or sth to get better distributed random values
fn get_random_coordinates(count: u32) -> Vec<(f32, f32)> {
    let mut rng = rand::thread_rng();
    let mut n = || rng.gen::<f32>() * MAP_SIZE * 2. - MAP_SIZE;
    (0..count).map(|_| (n(), n())).collect::<Vec<_>>()
}

// const MAP_COLOR: Color = Color::rgb(255. / 256., 149. / 256., 112. / 256.);
const MAP_COLOR: Color = Color::rgb(166. / 256., 89. / 256.,63. / 256.);

fn get_random_member(rng: &mut ThreadRng, vec: &Vec<Handle<Scene>>) -> Handle<Scene> {
    let n = rng.gen::<f32>();
    let i = (n * vec.len() as f32) as usize;
    return vec.get(i).unwrap().clone();
}
pub fn generate_map(
    ass: Res<AssetServer>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: MAP_SIZE * 2.,
                ..Default::default()
            })),
            material: materials.add(StandardMaterial {
                base_color: MAP_COLOR.into(),
                perceptual_roughness: 1.,
                ..default()
            }),
            ..default()
        },))
        .with_children(|c| {
            c.spawn((
                Transform::from_xyz(0.0, 0., 0.0).with_scale(Vec3::splat(1.4)),
                Collider::cuboid(MAP_SIZE, 0.01, MAP_SIZE),
                Friction::default(),
            ));
        });

    let assets = vec![
        "crater",
        "meteor_half",
        "rock",
        "rock_largeA",
        "rock_largeB",
        "rocks_smallA",
        "rocks_smallB",
    ]
    .iter()
    .map(|model_name| {
        ass.load(format!(
            "spacekit_2/Models/GLTF format/{}.glb#Scene0",
            model_name
        ))
    })
    .collect::<Vec<_>>();

    let mut rng = thread_rng();
    let count = 1000;
    for (x, z) in get_random_coordinates(count).into_iter() {
        commands.spawn(SceneBundle {
            scene: get_random_member(&mut rng, &assets),
            transform: Transform::from_translation(Vec3::new(x, 0., z)),
            ..Default::default()
        });
    }
}
