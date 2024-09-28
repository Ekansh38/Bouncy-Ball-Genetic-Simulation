use macroquad::prelude::*;

fn update_all_particles_as_balls(
    particles: &mut Vec<Particle>,
    delta_time: f32,
    mouse_tregectory: &mut Vec<Vector>,
) {
    for particle in particles.iter_mut() {
        particle.throwing_logic(mouse_tregectory);
        particle.update(delta_time);
    }
    for i in 0..particles.len() {
        for j in i + 1..particles.len() {
            let (left, right) = particles.split_at_mut(j);
            left[i].collide(&mut right[0]);
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

    request_new_screen_size(1800.0, 800.0);

    particles.push(Particle::new(600.0, -500.0, 35.0, RED, 0.9, 1.0, 5.0));
    particles.push(Particle::new(900.0, -300.0, 50.0, YELLOW, 0.9, 1.0, 6.0));
    particles.push(Particle::new(400.0, -300.0, 70.0, BLUE, 0.9, 1.0, 8.0));

    // Fps Logic
    let mut fps = 0;
    let mut update_fps_counter = 0.0;
    let mut can_update_fps = true;

    let mut previous_time = get_time();

    loop {
        let current_time = get_time();
        let delta_time = (current_time - previous_time) as f32;
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
    r: f32,
    c: Color,
    vel: Vector,
    grabing: bool,
    surface_friction: f32,
    retention: f32,
    mass: f32,
    force: Vector,
}

impl Particle {
    fn new(
        x: f32,
        y: f32,
        r: f32,
        c: Color,
        surface_friction: f32,
        retention: f32,
        mass: f32,
    ) -> Self {
        Self {
            pos: Vector::new(x, y),
            r,
            c,
            vel: Vector::new(0.0, 0.0),
            grabing: false,
            surface_friction,
            retention,
            mass: mass / 4.0,
            force: Vector::new(0.0, 0.0),
        }
    }

    fn friction_checks(&mut self) {
        if self.pos.y + self.r >= screen_height() {
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
        // Apply gravity
        let pixels_per_meter = 100.0;
        let universal_gravity_constant = 9.8; // 9.8 m/s^2
        let gravity = universal_gravity_constant * pixels_per_meter;
        self.vel.y += gravity * delta_time;
    }

    fn edges(&mut self) {
        if self.pos.y + self.r > screen_height() {
            self.pos.y = screen_height() - self.r;
            self.vel.y = self.vel.y * -1.0 * self.retention;
        }

        if self.pos.x + self.r > screen_width() {
            self.pos.x = screen_width() - self.r;
            self.vel.x = self.vel.x * -1.0 * self.retention;
        } else if self.pos.x - self.r < 0.0 {
            self.pos.x = self.r;
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

    fn update(&mut self, delta_time: f32) {
        if !self.grabing {
            self.apply_gravity(delta_time);
            self.friction_checks();
            self.euler_integration(delta_time);
            self.edges();
        } else {
            self.pos.x = mouse_position().0;
            self.pos.y = mouse_position().1;
            self.vel = Vector::new(0.0, 0.0);
        }
        // Update the force to 0
        self.force = Vector::new(0.0, 0.0);

        // Draw the particle
        self.draw();
    }

    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, self.r + 2.0, WHITE);
        draw_circle(self.pos.x, self.pos.y, self.r, self.c);
    }

    fn is_grabing(&mut self) -> i32 {
        let mouse_pos = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left)
            && mouse_pos.0 > self.pos.x - self.r
            && mouse_pos.0 < self.pos.x + self.r
            && mouse_pos.1 > self.pos.y - self.r
            && mouse_pos.1 < self.pos.y + self.r
        {
            self.grabing = true;
        } else if is_mouse_button_released(MouseButton::Left) && self.grabing {
            self.grabing = false;
            self.vel.y = 0.0;
            return -1;
        }

        if self.grabing {
            return 1;
        } else {
            return 0;
        }
    }

    fn apply_force(&mut self, force: Vector) {
        self.force = force;
    }

    fn collide(&mut self, other: &mut Particle) {
        let distance = self.pos.dist(&other.pos);
        let sum_radii = self.r + other.r;

        if distance < sum_radii {
            let line_of_impact = other.pos.subract(&self.pos).divide(distance);

            let relative_velocity = other.vel.subract(&self.vel);
            let velocity_along_normal = relative_velocity.dot(&line_of_impact);

            if velocity_along_normal > 0.0 {
                return;
            }

            let restitution = 0.7; // Elastic collision
            let impulse_scalar = -(1.0 + restitution) * velocity_along_normal;

            let impulse = line_of_impact.multiply(impulse_scalar);

            self.vel = self.vel.subract(&impulse.divide(self.mass));
            other.vel = other.vel.add(&impulse.divide(other.mass));
        }
    }
}
