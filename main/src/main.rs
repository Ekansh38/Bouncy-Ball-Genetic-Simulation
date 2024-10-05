use ::rand::thread_rng;
use ::rand::Rng; // Add this import to use gen_range
use macroquad::prelude::*;
use std::fs::OpenOptions;
use std::io::Result;
use std::io::Write;

fn update_all_particles_as_balls(
    particles: &mut Vec<Particle>,
    delta_time: f32,
    mouse_tregectory: &mut Vec<Vector>,
) {
    for particle in particles.iter_mut() {
        // particle.throwing_logic(mouse_tregectory);
        particle.update(delta_time);
    }
    for i in 0..particles.len() {
        for j in i + 1..particles.len() {
            let (left, right) = particles.split_at_mut(j);
            let new_particle = left[i].collide(&mut right[0]);
            if new_particle.is_some() {
                particles.push(new_particle.unwrap());
            }
        }
    }
}

fn update_all_particles(
    particles: &mut Vec<Particle>,
    delta_time: f32,
    _mouse_tregectory: &mut Vec<Vector>,
) {
    for particle in particles.iter_mut() {
        // particle.throwing_logic(mouse_tregectory);
        particle.update(delta_time);
    }
}

#[macroquad::main("Physics Engine")]
async fn main() {
    let mut particles = vec![];
    let mut mouse_tregectory: Vec<Vector> = Vec::new();

    request_new_screen_size(5000.0, 5000.0);

    particles.push(Particle::new(1000.0, 100.0, 35.0, RED, 1.0, 1.0, 2.0));
    particles.push(Particle::new(100.0, 200.0, 30.0, YELLOW, 1.0, 1.0, 4.0));
    particles.push(Particle::new(600.0, 50.0, 30.0, BLUE, 1.0, 1.0, 3.0));

    // Fps Logic
    let mut fps = 0;
    let mut update_fps_counter = 0.0;
    let mut can_update_fps = true;

    let mut previous_time = get_time();

    loop {
        let current_time = get_time();
        let mut delta_time = (current_time - previous_time) as f32;
        delta_time = delta_time * 2.0;
        previous_time = current_time;

        clear_background(BLACK);

        update_all_particles_as_balls(&mut particles, delta_time, &mut mouse_tregectory);

        // Logic for FPS

        draw_text(
            &format!("FPS: {}", fps.to_string()),
            10.0,
            20.0,
            32.0,
            WHITE,
        );

        if can_update_fps {
            fps = get_fps();
            update_fps_counter = 0.0;
            can_update_fps = false;
        } else {
            update_fps_counter += 150.0 * delta_time;
            if update_fps_counter > 100.0 {
                can_update_fps = true;
            }
        }
        next_frame().await
    }
}
#[derive(Clone, Copy)]
struct Vector {
    x: f32,
    y: f32,
}

impl Vector {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn dot(&self, other: &Vector) -> f32 {
        self.x * other.x + self.y * other.y
    }

    fn add(&self, other: &Vector) -> Vector {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn subract(&self, other: &Vector) -> Vector {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn divide(&self, scalar: f32) -> Vector {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }

    fn divide_vectors(&self, other: &Vector) -> Vector {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }

    fn multiply_vectors(&self, other: &Vector) -> Vector {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    fn multiply(&self, scalar: f32) -> Vector {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }

    fn dist(&self, other: &Vector) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

struct Particle {
    pos: Vector,
    radius: f32,
    color: Color,
    vel: Vector,
    is_grabing: bool,
    surface_friction: f32,
    retention: f32,
    mass: f32,
    force: Vector,
    max_speed: Vector,
    made_baby: bool,
    made_baby_counter: f32,
}

impl Particle {
    fn new(
        x: f32,
        y: f32,
        radius: f32,
        color: Color,
        surface_friction: f32,
        retention: f32,
        mass: f32,
    ) -> Self {
        let mut rng = thread_rng();
        Self {
            pos: Vector::new(x, y),
            radius,
            color,
            vel: Vector::new(rng.gen_range(-100.0..=100.0), rng.gen_range(-100.0..=100.0)),
            is_grabing: false,
            surface_friction,
            retention,
            mass: mass / 4.0,
            force: Vector::new(0.0, 0.0),
            max_speed: Vector::new(250.0, 250.0),
            made_baby: false,
            made_baby_counter: 0.0,
        }
    }

    fn surface_friction(&mut self) {
        if self.pos.y + self.radius >= screen_height() {
            self.vel.x = self.vel.x * self.surface_friction
        }
    }

    fn euler_integration(&mut self, delta_time: f32) {
        // Newton's second law of motion: F = ma
        let mass_vector = Vector::new(self.mass, self.mass);
        let acc = self.force.divide_vectors(&mass_vector);

        // Update velocity using Euler's method
        self.vel = self.vel.add(&acc.multiply(delta_time));

        self.pos = self.pos.add(&self.vel.multiply(delta_time));
    }

    fn apply_gravity(&mut self, delta_time: f32) {
        let pixels_per_meter = 100.0;
        let universal_gravity_constant = 0.5; // 9.8 m/s^2
        let gravity = universal_gravity_constant * pixels_per_meter;
        self.vel.y += gravity * delta_time;
    }

    fn check_edges(&mut self) {
        if self.pos.y + self.radius > screen_height() {
            self.pos.y = screen_height() - self.radius;
            self.vel.y = self.vel.y * -1.0 * self.retention;
        }

        if self.pos.y - self.radius < 0.0 {
            self.pos.y = self.radius;
            self.vel.y = self.vel.y * -1.0 * self.retention;
        }

        if self.pos.x + self.radius > screen_width() {
            self.pos.x = screen_width() - self.radius;
            self.vel.x = self.vel.x * -1.0 * self.retention;
        } else if self.pos.x - self.radius < 0.0 {
            self.pos.x = self.radius;
            self.vel.x = self.vel.x * -1.0 * self.retention;
        }
    }

    fn throwing_logic(&mut self, mouse_tregectory: &mut Vec<Vector>) {
        let grabing = self.is_grabing();

        if grabing == 1 {
            mouse_tregectory.push(Vector::new(mouse_position().0, mouse_position().1));
            if mouse_tregectory.len() > 20 {
                mouse_tregectory.remove(0);
            }
        } else if grabing == -1 {
            self.vel.x = 0.0;
            self.vel.y = 0.0;
            let push = mouse_tregectory[0].subract(&mouse_tregectory[mouse_tregectory.len() - 1]);

            let push = push.multiply(-1.0);

            let mag = 2000.0;
            let push = push.multiply(mag);

            let force = push;
            self.apply_force(force);

            mouse_tregectory.clear();
        }
    }

    fn update(&mut self, delta_time: f32) -> Result<()> {
        if !self.is_grabing {
            self.apply_gravity(delta_time);
            self.surface_friction();
            self.euler_integration(delta_time);
            self.check_edges();

            if self.made_baby_counter > 0.0 {
                self.made_baby_counter -= 1.0 * delta_time;
            } else {
                self.made_baby_counter = 0.0;
                self.made_baby = false;
            }

            if self.made_baby {
                self.color = RED;
            }
            if !self.made_baby {
                self.color = GREEN;
            }

            if self.vel.x > self.max_speed.x {
                self.vel.x = self.max_speed.x;
            }

            if self.vel.y > self.max_speed.y {
                self.vel.y = self.max_speed.y;
            }
        } else {
            self.pos.x = mouse_position().0;
            self.pos.y = mouse_position().1;
            self.vel = Vector::new(0.0, 0.0);
        }
        // Update the force to 0
        self.force = Vector::new(0.0, 0.0);

        // Draw the particle
        self.draw();
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("data.txt")?;

        file.write_all(format!("x: {}, y: {}\n", self.vel.x, self.vel.y).as_bytes())?;

        Ok(())
    }

    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, self.radius + 2.0, WHITE);
        draw_circle(self.pos.x, self.pos.y, self.radius, self.color);
    }

    fn is_grabing(&mut self) -> i32 {
        let mouse_pos = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left)
            && mouse_pos.0 > self.pos.x - self.radius
            && mouse_pos.0 < self.pos.x + self.radius
            && mouse_pos.1 > self.pos.y - self.radius
            && mouse_pos.1 < self.pos.y + self.radius
        {
            self.is_grabing = true;
        } else if is_mouse_button_released(MouseButton::Left) && self.is_grabing {
            self.is_grabing = false;
            self.vel.y = 0.0;
            return -1;
        }

        if self.is_grabing {
            return 1;
        } else {
            return 0;
        }
    }

    fn apply_force(&mut self, force: Vector) {
        self.force = force;
    }

    fn create_baby(&self, other: &Particle) -> Particle {
        return Particle::new(
            self.pos.x,
            self.pos.y,
            self.radius / 2.0,
            self.color,
            self.surface_friction,
            self.retention,
            self.mass * 2.0,
        );
    }
    fn collide(&mut self, other: &mut Particle) -> Option<Particle> {
        let distance = self.pos.dist(&other.pos);
        let sum_radii = self.radius + other.radius;

        if distance < sum_radii {
            let line_of_impact = other.pos.subract(&self.pos).divide(distance);

            // Position correction
            let overlap = sum_radii - distance;
            let correction = line_of_impact.multiply(overlap / 2.0);

            self.pos = self.pos.subract(&correction);
            other.pos = other.pos.add(&correction);

            let relative_velocity = other.vel.subract(&self.vel);
            let velocity_along_normal = relative_velocity.dot(&line_of_impact);

            if velocity_along_normal > 0.0 {
                return Option::None;
            }

            let restitution = 0.7; // Elastic collision
            let impulse_scalar = -(1.0 + restitution) * velocity_along_normal;

            let impulse = line_of_impact.multiply(impulse_scalar);

            self.vel = self.vel.subract(&impulse.divide(self.mass));
            other.vel = other.vel.add(&impulse.divide(other.mass));

            if !self.made_baby && !other.made_baby {
                self.made_baby = true;
                other.made_baby = true;
                let baby_delay = 5.0;
                self.made_baby_counter = baby_delay;
                other.made_baby_counter = baby_delay;
                return Some(self.create_baby(other));
            }
        }

        None
    }
}
