use crate::{
    game::{Assets, GetAsset},
    gml::rand::Random,
    math::Real,
    render::Renderer,
};

pub struct ParticleType {
    pub graphic: ParticleGraphic,
    pub size_min: Real,
    pub size_max: Real,
    pub size_incr: Real,
    pub size_wiggle: Real,
    pub xscale: Real,
    pub yscale: Real,
    pub life_min: i32,
    pub life_max: i32,
    pub death_type: i32,
    pub death_number: i32,
    pub step_type: i32,
    pub step_number: i32,
    pub speed_min: Real,
    pub speed_max: Real,
    pub speed_incr: Real,
    pub speed_wiggle: Real,
    pub dir_min: Real,
    pub dir_max: Real,
    pub dir_incr: Real,
    pub dir_wiggle: Real,
    pub grav_amount: Real,
    pub grav_dir: Real,
    pub ang_min: Real,
    pub ang_max: Real,
    pub ang_incr: Real,
    pub ang_wiggle: Real,
    pub ang_relative: bool,
    pub color: ParticleColor,
    pub alpha1: Real,
    pub alpha2: Real,
    pub alpha3: Real,
    pub additive_blending: bool,
}

pub enum ParticleGraphic {
    Sprite { sprite: i32, animat: bool, stretch: bool, random: bool },
    Shape(i32),
}

pub enum ParticleColor {
    One(i32),
    Two(i32, i32),
    Three(i32, i32, i32),
    RGB { rmin: i32, rmax: i32, gmin: i32, gmax: i32, bmin: i32, bmax: i32 },
    HSV { hmin: i32, hmax: i32, smin: i32, smax: i32, vmin: i32, vmax: i32 },
    Mix(i32, i32),
}

fn color_lerp(c1: i32, c2: i32, lerp: Real) -> i32 {
    let c1_part = Real::from(1.0) - lerp;
    let c2_part = lerp;
    let (r1, r2) = (c1 & 0xff, c2 & 0xff);
    let (g1, g2) = ((c1 >> 8) & 0xff, (c2 >> 8) & 0xff);
    let (b1, b2) = ((c1 >> 16) & 0xff, (c2 >> 16) & 0xff);
    let r = (Real::from(r1) * c1_part + Real::from(r2) * c2_part).round();
    let g = (Real::from(g1) * c1_part + Real::from(g2) * c2_part).round();
    let b = (Real::from(b1) * c1_part + Real::from(b2) * c2_part).round();
    r | (g << 8) | (b << 16)
}

pub struct Particle {
    ptype: i32,
    timer: i32,
    lifetime: i32,
    x: Real,
    y: Real,
    xprevious: Real,
    yprevious: Real,
    speed: Real,
    direction: Real,
    image_angle: Real,
    color: i32,
    alpha: Real,
    size: Real,
    subimage: i32,
    random_start: i32,
}

pub struct System {
    pub particles: Vec<Particle>,
    pub emitters: Vec<Option<Emitter>>,
    pub attractors: Vec<Option<Attractor>>,
    pub destroyers: Vec<Option<Destroyer>>,
    pub deflectors: Vec<Option<Deflector>>,
    pub changers: Vec<Option<Changer>>,
    pub draw_old_to_new: bool,
    pub depth: Real,
    pub x: Real,
    pub y: Real,
    pub auto_update: bool,
    pub auto_draw: bool,
}

pub struct Attractor {
    pub x: Real,
    pub y: Real,
    pub force: Real,
    pub dist: Real,
    pub kind: ForceKind,
    pub additive: bool,
}

#[derive(PartialEq, Eq)]
pub enum ForceKind {
    Constant,
    Linear,
    Quadratic,
}

pub struct Changer {
    pub xmin: Real,
    pub xmax: Real,
    pub ymin: Real,
    pub ymax: Real,
    pub shape: Shape,
    pub parttype1: i32,
    pub parttype2: i32,
    pub kind: ChangerKind,
}

pub enum ChangerKind {
    Motion,
    Shape,
    All,
}

pub struct Deflector {
    pub xmin: Real,
    pub xmax: Real,
    pub ymin: Real,
    pub ymax: Real,
    pub kind: DeflectorKind,
    pub friction: Real,
}

pub enum DeflectorKind {
    Horizontal,
    Vertical,
}

pub struct Destroyer {
    pub xmin: Real,
    pub xmax: Real,
    pub ymin: Real,
    pub ymax: Real,
    pub shape: Shape,
}

pub struct Emitter {
    pub xmin: Real,
    pub xmax: Real,
    pub ymin: Real,
    pub ymax: Real,
    pub shape: Shape,
    pub distribution: Distribution,
    pub ptype: i32,
    pub number: i32,
}

#[derive(PartialEq, Eq)]
pub enum Distribution {
    Linear,
    Gaussian,
    InvGaussian,
}

impl Distribution {
    fn range(&self, rand: &mut Random, min: Real, max: Real) -> Real {
        if min < max {
            match self {
                Distribution::Linear => Real::from(rand.next((max - min).into())) + min,
                Distribution::Gaussian => {
                    let x = loop {
                        let x = Real::from(rand.next(6.0) - 3.0);
                        if (-x * x * Real::from(0.5)).exp() > Real::from(rand.next(1.0)) {
                            break x
                        }
                    };
                    min + (max - min) * (x + Real::from(3.0)) / Real::from(6.0)
                },
                Distribution::InvGaussian => {
                    let mut x = loop {
                        let x = Real::from(rand.next(6.0) - 3.0);
                        if (-x * x * Real::from(0.5)).exp() > Real::from(rand.next(1.0)) {
                            break x
                        }
                    };
                    if x < Real::from(0.0) {
                        x += Real::from(6.0);
                    }
                    min + (max - min) * x / Real::from(6.0)
                },
            }
        } else {
            min
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Shape {
    Rectangle,
    Ellipse,
    Diamond,
    Line,
}

impl Shape {
    fn contains(&self, x: Real, y: Real, x1: Real, y1: Real, x2: Real, y2: Real) -> bool {
        if x1 > x || x > x2 || y1 > y || y > y2 {
            return false
        }
        let xcenter = (x1 + x2) / Real::from(2.0);
        let ycenter = (y1 + y2) / Real::from(2.0);
        let xsize = x2 - x1;
        let ysize = y2 - y1;
        // Scaled to range (-1.0, 1.0)
        let xdiff = (x - xcenter) * Real::from(2.0) / xsize;
        let ydiff = (y - ycenter) * Real::from(2.0) / ysize;
        match self {
            Self::Rectangle => true,
            Self::Ellipse => xdiff.into_inner().hypot(ydiff.into_inner()) <= 1.0,
            Self::Diamond => xdiff + ydiff <= Real::from(1.0),
            Self::Line => true, // This will be dealt with differently
        }
    }
}

impl ParticleType {
    pub fn new() -> Self {
        Self {
            graphic: ParticleGraphic::Shape(0),
            size_min: Real::from(1.0),
            size_max: Real::from(1.0),
            size_incr: Real::from(0.0),
            size_wiggle: Real::from(0.0),
            xscale: Real::from(1.0),
            yscale: Real::from(1.0),
            life_min: 100,
            life_max: 100,
            step_type: 0,
            step_number: 0,
            death_type: 0,
            death_number: 0,
            speed_min: Real::from(0.0),
            speed_max: Real::from(0.0),
            speed_incr: Real::from(0.0),
            speed_wiggle: Real::from(0.0),
            dir_min: Real::from(0.0),
            dir_max: Real::from(0.0),
            dir_incr: Real::from(0.0),
            dir_wiggle: Real::from(0.0),
            ang_min: Real::from(0.0),
            ang_max: Real::from(0.0),
            ang_incr: Real::from(0.0),
            ang_wiggle: Real::from(0.0),
            ang_relative: false,
            grav_amount: Real::from(0.0),
            grav_dir: Real::from(270.0),
            color: ParticleColor::One(0xffffff),
            alpha1: Real::from(1.0),
            alpha2: Real::from(1.0),
            alpha3: Real::from(1.0),
            additive_blending: false,
        }
    }
}

impl System {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            emitters: Vec::new(),
            attractors: Vec::new(),
            destroyers: Vec::new(),
            deflectors: Vec::new(),
            changers: Vec::new(),
            draw_old_to_new: true,
            depth: Real::from(0.0),
            x: Real::from(0.0),
            y: Real::from(0.0),
            auto_update: true,
            auto_draw: true,
        }
    }

    pub fn update(&mut self, rand: &mut Random, types: &dyn GetAsset<Box<ParticleType>>) {
        self.update_lifetimes(rand, types);
        for particle in self.particles.iter_mut() {
            particle.update_physics(&self.attractors, types);
            particle.update_graphics(types);
        }
        for deflector in self.deflectors.iter().filter_map(|x| x.as_ref()) {
            deflector.update(&mut self.particles);
        }
        for changer in self.changers.iter().filter_map(|x| x.as_ref()) {
            changer.update(&mut self.particles, rand, types);
        }
        for destroyer in self.destroyers.iter().filter_map(|x| x.as_ref()) {
            destroyer.update(&mut self.particles);
        }
        for emitter in self.emitters.iter().filter_map(|x| x.as_ref()) {
            if emitter.number != 0 {
                emitter.burst(emitter.ptype, emitter.number, &mut self.particles, rand, types);
            }
        }
    }

    pub fn draw(&self, renderer: &mut Renderer, assets: &Assets, types: &dyn GetAsset<Box<ParticleType>>) {
        // TODO set texture lerp on just for this function
        if self.draw_old_to_new {
            for particle in &self.particles {
                particle.draw(self.x, self.y, renderer, assets, types);
            }
        } else {
            for particle in self.particles.iter().rev() {
                particle.draw(self.x, self.y, renderer, assets, types);
            }
        }
    }

    pub fn create_particles(
        &mut self,
        x: Real,
        y: Real,
        ptype: i32,
        number: i32,
        rand: &mut Random,
        types: &dyn GetAsset<Box<ParticleType>>,
    ) {
        for _ in 0..number {
            if let Some(particle) = Particle::new(ptype, x, y, rand, types) {
                self.particles.push(particle);
            }
        }
    }

    pub fn create_particles_color(
        &mut self,
        x: Real,
        y: Real,
        ptype: i32,
        color: i32,
        number: i32,
        rand: &mut Random,
        types: &dyn GetAsset<Box<ParticleType>>,
    ) {
        for _ in 0..number {
            if let Some(mut particle) = Particle::new(ptype, x, y, rand, types) {
                particle.color = color;
                self.particles.push(particle);
            }
        }
    }

    fn update_lifetimes(&mut self, rand: &mut Random, types: &dyn GetAsset<Box<ParticleType>>) {
        for i in 0..self.particles.len() {
            let particle = &mut self.particles[i];
            particle.timer += 1;
            // copy x and y to avoid borrowing conflicts
            let x = particle.x;
            let y = particle.y;
            if let Some(ptype) = types.get_asset(particle.ptype) {
                if particle.timer >= particle.lifetime {
                    let mut number = ptype.death_number;
                    if number < 0 && rand.next_int((-number) as u32) == 0 {
                        number = 1;
                    }
                    self.create_particles(x, y, ptype.death_type, number, rand, types);
                } else {
                    // particle is alive
                    let mut number = ptype.step_number;
                    if number < 0 && rand.next_int((-number) as u32) == 0 {
                        number = 1;
                    }
                    self.create_particles(x, y, ptype.step_type, number, rand, types);
                }
            }
        }
        self.particles.retain(|p| p.timer < p.lifetime);
    }
}

impl Particle {
    fn new(
        ptype_id: i32,
        x: Real,
        y: Real,
        rand: &mut Random,
        types: &dyn GetAsset<Box<ParticleType>>,
    ) -> Option<Self> {
        if let Some(ptype) = types.get_asset(ptype_id) {
            let speed = Distribution::Linear.range(rand, ptype.speed_min.into(), ptype.speed_max.into()).into();
            let direction = Distribution::Linear.range(rand, ptype.dir_min.into(), ptype.dir_max.into()).into();
            let image_angle = Distribution::Linear.range(rand, ptype.ang_min.into(), ptype.ang_max.into()).into();
            let lifetime = Distribution::Linear.range(rand, ptype.life_min.into(), ptype.life_max.into()).round();
            let color = Self::init_color(rand, &ptype.color);
            let alpha = ptype.alpha1;
            let size = Distribution::Linear.range(rand, ptype.size_min.into(), ptype.size_max.into()).into();
            let mut subimage = 0;
            if let ParticleGraphic::Sprite { sprite: _, animat: _, stretch: _, random } = ptype.graphic {
                if random {
                    subimage = rand.next_int(10000);
                }
            }
            let random_start = rand.next_int(100000);
            Some(Self {
                ptype: ptype_id,
                timer: 0,
                lifetime,
                x,
                y,
                xprevious: x,
                yprevious: y,
                speed,
                direction,
                image_angle,
                color,
                alpha,
                size,
                subimage,
                random_start,
            })
        } else {
            None
        }
    }

    fn init_color(rand: &mut Random, color_type: &ParticleColor) -> i32 {
        match color_type {
            ParticleColor::One(c) => *c,
            ParticleColor::Two(c, _) => *c,
            ParticleColor::Three(c, _, _) => *c,
            ParticleColor::RGB { rmin, rmax, gmin, gmax, bmin, bmax } => {
                let r = Distribution::Linear.range(rand, (*rmin).into(), (*rmax).into()).round();
                let g = Distribution::Linear.range(rand, (*gmin).into(), (*gmax).into()).round();
                let b = Distribution::Linear.range(rand, (*bmin).into(), (*bmax).into()).round();
                r | (g << 8) | (b << 16)
            },
            ParticleColor::HSV { hmin, hmax, smin, smax, vmin, vmax } => {
                let h = Distribution::Linear.range(rand, (*hmin).into(), (*hmax).into()).round();
                let s = Distribution::Linear.range(rand, (*smin).into(), (*smax).into()).round();
                let v = Distribution::Linear.range(rand, (*vmin).into(), (*vmax).into()).round();

                let h = Real::from(h) * Real::from(360.0) / Real::from(255.0);
                let s = Real::from(s) / Real::from(255.0);
                let v = Real::from(v) / Real::from(255.0);
                let chroma = v * s;
                let hprime = (h / Real::from(60.0)) % Real::from(6.0);
                let x = chroma * (Real::from(1.0) - ((hprime % Real::from(2.0)) - Real::from(1.0)).abs());
                let m = v - chroma;

                let (r, g, b) = match hprime.floor().into_inner() as i32 {
                    0 => (chroma, x, Real::from(0.0)),
                    1 => (x, chroma, Real::from(0.0)),
                    2 => (Real::from(0.0), chroma, x),
                    3 => (Real::from(0.0), x, chroma),
                    4 => (x, Real::from(0.0), chroma),
                    5 => (chroma, Real::from(0.0), x),
                    _ => (Real::from(0.0), Real::from(0.0), Real::from(0.0)),
                };

                let out_r = ((r + m) * Real::from(255.0)).round();
                let out_g = ((g + m) * Real::from(255.0)).round();
                let out_b = ((b + m) * Real::from(255.0)).round();
                out_r | (out_g << 8) | (out_b << 16)
            },
            ParticleColor::Mix(c1, c2) => color_lerp(*c1, *c2, rand.next(1.0).into()),
        }
    }

    fn update_physics(&mut self, attractors: &Vec<Option<Attractor>>, types: &dyn GetAsset<Box<ParticleType>>) {
        if let Some(ptype) = types.get_asset(self.ptype) {
            self.xprevious = self.x;
            self.yprevious = self.y;
            self.speed += ptype.speed_incr;
            self.speed = self.speed.max(Real::from(0.0));
            self.direction += ptype.dir_incr;
            self.image_angle += ptype.ang_incr;
            // gravity and attractors
            if ptype.grav_amount != Real::from(0.0) || !attractors.is_empty() {
                // make the speed cartesian for a bit
                let mut hspeed = self.direction.to_radians().cos() * self.speed;
                let mut vspeed = -self.direction.to_radians().sin() * self.speed;
                // gravity
                hspeed += ptype.grav_dir.to_radians().cos() * ptype.grav_amount;
                vspeed -= ptype.grav_dir.to_radians().sin() * ptype.grav_amount;
                // attractors
                for attractor in attractors.iter().filter_map(|x| x.as_ref()) {
                    if attractor.force != Real::from(0.0) && attractor.dist != Real::from(0.0) {
                        let xdiff = attractor.x - self.x;
                        let ydiff = attractor.y - self.y;
                        let dist = (xdiff * xdiff + ydiff * ydiff).sqrt();
                        if dist <= attractor.dist && dist > Real::from(0.0) {
                            let mut haccel = attractor.force * xdiff / dist;
                            let mut vaccel = attractor.force * ydiff / dist;
                            if attractor.kind != ForceKind::Constant {
                                let extra = (attractor.dist - dist) / attractor.dist;
                                haccel *= extra;
                                vaccel *= extra;
                                if attractor.kind == ForceKind::Quadratic {
                                    haccel *= extra;
                                    vaccel *= extra;
                                }
                            }
                            if attractor.additive {
                                hspeed += haccel;
                                vspeed += vaccel;
                            } else {
                                self.x += haccel;
                                self.y += vaccel;
                            }
                        }
                    }
                }
                // revert to polar speed
                self.speed = (hspeed * hspeed + vspeed * vspeed).sqrt();
                self.direction = -vspeed.arctan2(hspeed).to_degrees();
            }

            let mut dir_wiggle = Real::from((self.timer + self.random_start * 3) % 24) / Real::from(6.0);
            if Real::from(2.0) < dir_wiggle {
                dir_wiggle = Real::from(4.0) - dir_wiggle;
            }
            dir_wiggle -= Real::from(1.0);
            let direction = self.direction + dir_wiggle;

            let mut speed_wiggle = Real::from((self.timer + self.random_start * 4) % 20) / Real::from(5.0);
            if Real::from(2.0) < speed_wiggle {
                speed_wiggle = Real::from(4.0) - speed_wiggle;
            }
            speed_wiggle -= Real::from(1.0);
            let speed = self.speed + speed_wiggle;

            self.x += direction.to_radians().cos() * speed;
            self.y -= direction.to_radians().sin() * speed;
        }
    }

    fn update_graphics(&mut self, types: &dyn GetAsset<Box<ParticleType>>) {
        if let Some(ptype) = types.get_asset(self.ptype) {
            self.size += ptype.size_incr;
            self.size = self.size.max(Real::from(0.0));
            let mut halflife_elapsed = Real::from(1.0);
            if self.lifetime >= 1 {
                halflife_elapsed = Real::from(self.timer * 2) / Real::from(self.lifetime);
            }
            self.color = match ptype.color {
                ParticleColor::Two(c1, c2) => color_lerp(c1, c2, halflife_elapsed / Real::from(2.0)),
                ParticleColor::Three(c1, c2, c3) => {
                    if halflife_elapsed < Real::from(1.0) {
                        color_lerp(c1, c2, halflife_elapsed)
                    } else {
                        color_lerp(c2, c3, halflife_elapsed - Real::from(1.0))
                    }
                },
                _ => self.color,
            };
            if halflife_elapsed >= Real::from(1.0) {
                self.alpha = (halflife_elapsed - Real::from(1.0)) * ptype.alpha3
                    + (Real::from(2.0) - halflife_elapsed) * ptype.alpha2;
            } else {
                self.alpha = halflife_elapsed * ptype.alpha2 + (Real::from(1.0) - halflife_elapsed) * ptype.alpha1;
            }
        }
    }

    fn draw(
        &self,
        system_x: Real,
        system_y: Real,
        renderer: &mut Renderer,
        assets: &Assets,
        types: &dyn GetAsset<Box<ParticleType>>,
    ) {
        let ptype = types.get_asset(self.ptype);
        if ptype.is_none() {
            return
        }
        let ptype = ptype.unwrap();
        let image_count = 4;
        let atlas_ref = match ptype.graphic {
            ParticleGraphic::Sprite { sprite, animat, stretch, random: _ } => {
                let mut subimage = self.subimage;
                if animat {
                    if stretch && self.lifetime > 0 {
                        subimage += image_count * self.timer / self.lifetime;
                    } else {
                        subimage += self.timer;
                    }
                }
                if let Some(sprite) = assets.sprites.get_asset(sprite) {
                    sprite.frames.get((subimage % sprite.frames.len() as i32) as usize).map(|x| &x.atlas_ref)
                } else {
                    None
                }
            },
            ParticleGraphic::Shape(_) => None, // TODO
        };
        let mut angle_wiggle = ((self.timer + self.random_start * 2) % 16) as f64 / 4.0;
        if 2.0 < angle_wiggle {
            angle_wiggle = 4.0 - angle_wiggle;
        }
        angle_wiggle -= 1.0;
        let mut angle = self.image_angle + ptype.ang_wiggle * angle_wiggle.into();
        if ptype.ang_relative {
            angle += self.direction;
        }
        let mut size_wiggle = ((self.timer + self.random_start) % 16) as f64 / 4.0;
        if 2.0 < size_wiggle {
            size_wiggle = 4.0 - size_wiggle;
        }
        size_wiggle -= 1.0;
        let size = self.size + ptype.size_wiggle * size_wiggle.into();
        // TODO set blend mode to bm_add if additive
        if let Some(atlas_ref) = atlas_ref {
            renderer.draw_sprite(
                atlas_ref,
                (system_x + self.x).into(),
                (system_y + self.y).into(),
                (ptype.xscale * size).into(),
                (ptype.yscale * size).into(),
                angle.into(),
                self.color.into(),
                self.alpha.into(),
            );
        }
    }
}

impl Attractor {
    pub fn new() -> Self {
        Self {
            x: Real::from(0.0),
            y: Real::from(0.0),
            force: Real::from(0.0),
            dist: Real::from(0.0),
            kind: ForceKind::Constant,
            additive: false,
        }
    }
}

impl Deflector {
    pub fn new() -> Self {
        Self {
            xmin: Real::from(0.0),
            xmax: Real::from(0.0),
            ymin: Real::from(0.0),
            ymax: Real::from(0.0),
            kind: DeflectorKind::Horizontal,
            friction: Real::from(0.0),
        }
    }

    fn update(&self, particles: &mut Vec<Particle>) {
        if self.xmin < self.xmax && self.ymin < self.ymax {
            for particle in particles.iter_mut() {
                if self.xmin <= particle.x
                    && particle.x <= self.xmax
                    && self.ymin <= particle.y
                    && particle.y <= self.ymax
                {
                    particle.direction = match self.kind {
                        DeflectorKind::Horizontal => Real::from(180.0) - particle.direction,
                        DeflectorKind::Vertical => Real::from(360.0) - particle.direction,
                    }
                    .rem_euclid(Real::from(360.0));
                    particle.x = particle.xprevious - (particle.x - particle.xprevious);
                    particle.y = particle.yprevious - (particle.y - particle.yprevious);
                    particle.speed = (particle.speed - self.friction).max(Real::from(0.0));
                }
            }
        }
    }
}

impl Changer {
    pub fn new() -> Self {
        Self {
            xmin: Real::from(0.0),
            xmax: Real::from(0.0),
            ymin: Real::from(0.0),
            ymax: Real::from(0.0),
            shape: Shape::Rectangle,
            parttype1: -1,
            parttype2: -1,
            kind: ChangerKind::Motion,
        }
    }

    fn update(&self, particles: &mut Vec<Particle>, rand: &mut Random, types: &dyn GetAsset<Box<ParticleType>>) {
        if self.xmin < self.xmax && self.ymin < self.ymax {
            for particle in particles.iter_mut() {
                if self.shape.contains(particle.x, particle.y, self.xmin, self.ymin, self.xmax, self.ymax)
                    && self.parttype1 == particle.ptype
                {
                    if let Some(new_part) = Particle::new(self.parttype2, particle.x, particle.y, rand, types) {
                        particle.ptype = new_part.ptype;
                        match self.kind {
                            ChangerKind::Motion => {
                                particle.speed = new_part.speed;
                                particle.direction = new_part.direction;
                            },
                            ChangerKind::Shape => {
                                particle.color = new_part.color;
                                particle.alpha = new_part.alpha;
                                particle.size = new_part.size;
                                particle.subimage = new_part.subimage;
                                particle.image_angle = new_part.image_angle;
                            },
                            ChangerKind::All => *particle = new_part,
                        }
                    }
                }
            }
        }
    }
}

impl Destroyer {
    pub fn new() -> Self {
        Self {
            xmin: Real::from(0.0),
            xmax: Real::from(0.0),
            ymin: Real::from(0.0),
            ymax: Real::from(0.0),
            shape: Shape::Rectangle,
        }
    }

    fn update(&self, particles: &mut Vec<Particle>) {
        if self.xmin < self.xmax && self.ymin < self.ymax {
            particles.retain(|p| !self.shape.contains(p.x, p.y, self.xmin, self.ymin, self.xmax, self.ymax));
        }
    }
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            xmin: Real::from(0.0),
            xmax: Real::from(0.0),
            ymin: Real::from(0.0),
            ymax: Real::from(0.0),
            shape: Shape::Rectangle,
            distribution: Distribution::Linear,
            ptype: -1,
            number: 0,
        }
    }

    pub fn burst(
        &self,
        ptype: i32,
        number: i32,
        particles: &mut Vec<Particle>,
        rand: &mut Random,
        types: &dyn GetAsset<Box<ParticleType>>,
    ) {
        let number = if number < 0 {
            if rand.next_int(-number as u32) == 0 { 1 } else { return }
        } else {
            number
        };
        for _ in 0..number {
            let (xspawn, yspawn) = loop {
                let mut xspawn = self.distribution.range(rand, Real::from(0.0), Real::from(1.0));
                let mut yspawn = self.distribution.range(rand, Real::from(0.0), Real::from(1.0));
                if self.distribution == Distribution::InvGaussian && self.shape != Shape::Line {
                    if rand.next(1.0) >= 0.5 {
                        yspawn = rand.next(1.0).into();
                    } else {
                        xspawn = rand.next(1.0).into();
                    }
                }
                if self.shape.contains(
                    xspawn,
                    yspawn,
                    Real::from(0.0),
                    Real::from(0.0),
                    Real::from(1.0),
                    Real::from(1.0),
                ) {
                    if self.shape == Shape::Line {
                        yspawn = xspawn;
                    }
                    break (xspawn, yspawn)
                }
            };
            if let Some(particle) = Particle::new(
                ptype,
                (self.xmax - self.xmin) * xspawn + self.xmin,
                (self.ymax - self.ymin) * yspawn + self.ymin,
                rand,
                types,
            ) {
                particles.push(particle);
            }
        }
    }
}
