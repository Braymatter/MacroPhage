use bevy::prelude::*;
use std::fs::File;
use std::io::BufReader;

use super::{GameState, LevelManagerRes, Mutation, MutationFailed, PlayerMutationEvent};

pub fn process_map_mutations(
    mut mutation_events: EventReader<PlayerMutationEvent>,
    mut map_query: Query<(&mut GameState, Entity)>,
    mut mutation_failure_ev: EventWriter<MutationFailed>,
) {
    let (mut map, ent) = map_query.single_mut();

    for mutation_ev in mutation_events.iter() {
        match &mutation_ev.mutation {
            //Recombinators Trigger at the beginning of the next interval
            Mutation::TriggerRecombinator { target, cost } => {}

            //Vectors are removed at time of mutation
            Mutation::RemoveVector { relation, cost } => {
                if let Ok(()) = map.remove_vector(*relation) {
                } else {
                    mutation_failure_ev.send(MutationFailed {
                        mutation: mutation_ev.mutation.clone(),
                    })
                }
            }

            //Vectors are added at time of mutation
            Mutation::AddVector { relation, cost } => {
                if let Ok(()) = map.add_vector(*relation) {
                } else {
                    mutation_failure_ev.send(MutationFailed {
                        mutation: mutation_ev.mutation.clone(),
                    })
                }
            }

            //Replicator Output is changed at time of mutation and counter is reset
            Mutation::ChangeReplicatorType {
                replicator,
                new_type,
                cost,
            } => {}
        }
    }
}

pub fn spawn_map(
    mut commands: Commands,
    _assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_manager: Res<LevelManagerRes>,
    mut maps: Query<(&GameState, Entity)>,
) {
    if !level_manager.is_changed() {
        return;
    }

    maps.for_each_mut(|thing| {
        commands.entity(thing.1).despawn_recursive();
    });

    if level_manager.current_level == None {
        return;
    }

    let path = format!(
        "assets/maps/{}",
        level_manager.current_level.as_ref().unwrap()
    );
    debug!(
        "Loading level {}",
        level_manager.current_level.as_ref().unwrap()
    );

    let file = File::open(path).unwrap();

    let reader = BufReader::new(file);

    let map: GameState = serde_json::from_reader(reader).unwrap();

    let map_ent = commands
        .spawn_bundle(TransformBundle::default())
        .insert(Name::new("Map"))
        .id();
    let mut node_ents = Vec::default();
    for node in map.nodes.values() {
        node_ents.push(map.spawn_node(node, &mut commands, &mut meshes, &mut materials));
    }

    let mut vector_ents = Vec::default();
    for vector in map.vectors.iter() {
        let node_0 = map.nodes.get(&vector.0).unwrap();
        let node_1 = map.nodes.get(&vector.1).unwrap();

        let pos = (node_0.position + node_1.position) / 2.0;
        let mut transform = Transform::from_translation(pos).looking_at(node_1.position, Vec3::Y);
        transform.scale.z = Vec3::distance(node_0.position, node_1.position) * 2.0;

        vector_ents.push(
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
                    material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
                    transform,
                    ..default()
                })
                .insert(Name::new("Vector"))
                .insert(*vector)
                .id(),
        );
    }
    commands.entity(map_ent).push_children(&node_ents);
    commands.entity(map_ent).push_children(&vector_ents);
    commands.entity(map_ent).insert(map);
}

#[cfg(test)]
mod tests {
    use bevy::utils::HashMap;

    use crate::game::{Force, Generator, Nexus, NodeId, NodeTenant, PhageType, Replicator, Vector};

    use super::*;

    #[test]
    pub fn create_test_map() {
        use std::fs::File;
        use std::io::prelude::*;

        let nodes = HashMap::new();
        let vectors = vec![];
        let num_players = 5;

        let name: String = "Hello Map".to_string();
        let mut map = GameState {
            nodes,
            next_free_id: NodeId(0),
            vectors,
            name,
            num_players,
        };

        let node_1 = map.create_node(Force(0), Vec3::ZERO);
        let node_2 = map.create_node(Force(1), Vec3::new(1., 0., 3.));
        let node_3 = map.create_node(Force(2), Vec3::new(-2., 0., 4.));
        let node_4 = map.create_node(Force(3), Vec3::new(-20., 0., 12.));
        let node_5 = map.create_node(Force(4), Vec3::new(5.0, 0.0, 5.0));
        let node_6 = map.create_node(Force(5), Vec3::new(-5.0, 0.0, 5.0));
        let node_7 = map.create_node(Force(9), Vec3::new(-5.0, 0.0, 0.0));

        let replicator_node = map.nodes.get_mut(&node_5).expect("fuck");

        replicator_node.tenant = NodeTenant::Replicator {
            replicator: Replicator {
                output: PhageType::Electro,
                speed: 3,
            },
        };

        let nexus_node = map.nodes.get_mut(&node_6).expect("fuck 2");
        nexus_node.tenant = NodeTenant::Nexus { nexus: Nexus {} };

        let generator_node = map.nodes.get_mut(&node_7).expect("fuck 3");
        generator_node.tenant = NodeTenant::Generator {
            generator: Generator { amt: 50, speed: 1 },
        };

        map.add_vector(Vector::new(node_4, node_1)).unwrap();

        map.add_vector(Vector::new(node_1, node_2)).unwrap();
        map.remove_vector(Vector::new(node_1, node_2)).unwrap();
        map.add_vector(Vector::new(node_3, node_1)).unwrap();
        map.remove_vector(Vector::new(node_1, node_3)).unwrap();

        map.add_vector(Vector::new(node_3, node_2)).unwrap();
        map.add_vector(Vector::new(node_3, node_1)).unwrap();
        map.add_vector(Vector::new(node_1, node_2)).unwrap();
        map.remove_vector(Vector::new(node_3, node_2)).unwrap();
        map.add_vector(Vector::new(node_5, node_6)).unwrap();
        map.add_vector(Vector::new(node_5, node_1)).unwrap();
        map.add_vector(Vector::new(node_7, node_5)).unwrap();
        assert!(map.vectors.len() == 6);

        let map_json = serde_json::to_string(&map).unwrap();
        let mut input = File::create("assets/maps/test_map.json").unwrap();
        // https://doc.rust-lang.org/std/fs/struct.File.html
        input.write_all(map_json.as_bytes()).unwrap();

        println!("File Created Luv u -- ur pc");
    }
}

// Sphere = Cell
// TetraHedron = Replicator
// Pyramid = ReCombinator
// 3d Line = Vector
// Cylinder = Nexus
// Torus = Generator
