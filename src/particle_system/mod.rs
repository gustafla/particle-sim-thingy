mod particle_spawner;

use crate::Scene;
use cgmath::Vector3;
use opengles::glesv2::{self, constants::*, types::*};
pub use particle_spawner::*;

pub struct ParticleSystem {
    position_frames: Vec<Vec<f32>>,
    timestep: f32,
}

impl ParticleSystem {
    pub fn new(
        mut spawner: ParticleSpawner,
        frames: usize,
        timestep: f32,
        wind_field: Option<fn(Vector3<f32>, f32) -> Vector3<f32>>, // fn(pos, time) -> force
    ) -> ParticleSystem {
        let mut position_frames = Vec::with_capacity(frames);
        let mut positions = Vec::with_capacity(spawner.count_hint(frames) * 3);
        let mut velocities = Vec::with_capacity(spawner.count_hint(frames) * 3);
        let mut masses = Vec::with_capacity(spawner.count_hint(frames));

        for frame in 0..frames {
            // Print progress
            if frame % 100 == 0 {
                log::info!("{}%", frame * 100 / frames);
            }

            // Spawn particles
            if let Some(v) = spawner.next() {
                positions.extend(&v);
                velocities.extend(vec![0f32; v.len()]);
                masses.extend(vec![2f32; v.len() / 3]);
            }

            // Simulate wind
            if let Some(wind_field) = wind_field {
                for i in 0..masses.len() {
                    let force = wind_field(
                        Vector3::new(positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]),
                        timestep * frame as f32,
                    );

                    velocities[i * 3] += force.x / masses[i] * timestep;
                    velocities[i * 3 + 1] += force.y / masses[i] * timestep;
                    velocities[i * 3 + 2] += force.z / masses[i] * timestep;
                }
            }

            // Simulate gravity
            for i in 0..masses.len() {
                velocities[i * 3 + 1] -= masses[i] * timestep;
            }

            // Simulate drag
            for v in &mut velocities {
                *v *= 0.98;
            }

            // Integrate position
            for i in 0..positions.len().min(velocities.len()) / 3 {
                positions[i * 3] += velocities[i * 3] * timestep;
                positions[i * 3 + 1] += velocities[i * 3 + 1] * timestep;
                positions[i * 3 + 2] += velocities[i * 3 + 2] * timestep;
            }

            // Store frame state
            position_frames.push(positions.clone());
        }

        ParticleSystem {
            position_frames,
            timestep,
        }
    }

    pub fn render(&self, scene: &Scene, time: f32) {
        let i = (time / self.timestep) as usize;
        let i = i.min(self.position_frames.len() - 1); // clamp to frame count

        let program = scene
            .resources
            .program("./particle.vert ./particle.frag")
            .unwrap();

        glesv2::use_program(program.handle());

        glesv2::uniform_matrix4fv(
            program.uniform_location("u_Projection").unwrap(),
            false,
            &scene.projection,
        );
        glesv2::uniform_matrix4fv(
            program.uniform_location("u_View").unwrap(),
            false,
            &scene.view,
        );

        glesv2::bind_buffer(GL_ARRAY_BUFFER, 0);
        let index_pos = program.attrib_location("a_Pos").unwrap() as GLuint;

        glesv2::enable_vertex_attrib_array(index_pos);
        glesv2::vertex_attrib_pointer(index_pos, 3, GL_FLOAT, false, 0, &self.position_frames[i]);

        glesv2::draw_arrays(GL_POINTS, 0, self.position_frames[i].len() as GLint / 3);
    }
}
