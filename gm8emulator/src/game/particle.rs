use crate::{
    game::{Assets, GetAsset},
    gml::rand::Random,
    math::Real,
    render::{
        atlas::{AtlasBuilder, AtlasRef},
        BlendType, Renderer,
    },
};
use serde::{Deserialize, Serialize};

pub struct PSIterDrawOrder(usize);
impl PSIterDrawOrder {
    pub fn next(&mut self, manager: &Manager) -> Option<i32> {
        manager.draw_order.get(self.0).map(|val| {
            self.0 += 1 + manager
                .draw_order
                .iter()
                .skip(self.0 + 1)
                .position(|i| manager.systems.get_asset(*i).filter(|s| s.auto_draw).is_some())
                .unwrap_or(manager.draw_order.len());
            *val
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EffectManager {
    system_above: i32,
    system_below: i32,

    explosion_types: [i32; 6],
    ring_types: [i32; 3],
    ellipse_types: [i32; 3],
    firework_types: [i32; 3],
    smoke_types: [i32; 3],
    smokeup_types: [i32; 3],
    star_types: [i32; 3],
    spark_types: [i32; 3],
    flare_types: [i32; 3],
    cloud_types: [i32; 3],
    rain_type: i32,
    snow_type: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manager {
    systems: Vec<Option<Box<System>>>,
    types: Vec<Option<Box<ParticleType>>>,
    draw_order: Vec<i32>,
    shapes: Vec<AtlasRef>,

    dnd_system: i32,
    dnd_types: [i32; 16],
    dnd_emitters: [i32; 8],

    effects: Option<EffectManager>,
}

impl Manager {
    pub fn new(shapes: Vec<AtlasRef>) -> Self {
        Self {
            systems: Vec::new(),
            types: Vec::new(),
            draw_order: Vec::new(),
            shapes,
            dnd_system: -1,
            dnd_types: [-1; 16],
            dnd_emitters: [-1; 8],
            effects: None,
        }
    }

    pub fn create_system(&mut self) -> i32 {
        let ps = Box::new(System::new());
        let id = if let Some(id) = self.systems.iter().position(|x| x.is_none()) {
            self.systems[id] = Some(ps);
            id as i32
        } else {
            self.systems.push(Some(ps));
            self.systems.len() as i32 - 1
        };
        self.draw_order.push(id);
        id
    }

    pub fn get_system(&self, id: i32) -> Option<&Box<System>> {
        self.systems.get_asset(id)
    }

    pub fn get_system_mut(&mut self, id: i32) -> Option<&mut Box<System>> {
        self.systems.get_asset_mut(id)
    }

    pub fn auto_update_systems(&mut self, rand: &mut Random) {
        for ps in self.systems.iter_mut().filter_map(|x| x.as_mut().filter(|s| s.auto_update)) {
            ps.update(rand, &self.types);
        }
    }

    pub fn update_system(&mut self, id: i32, rand: &mut Random) {
        if let Some(ps) = self.systems.get_asset_mut(id) {
            ps.update(rand, &self.types);
        }
    }

    pub fn draw_system(&mut self, id: i32, renderer: &mut Renderer, assets: &Assets, set_depth: bool) {
        if let Some(ps) = self.systems.get_asset_mut(id) {
            if set_depth {
                renderer.set_depth(ps.depth.into_inner() as f32);
            }
            ps.draw(renderer, assets, &self.types, &self.shapes);
        }
    }

    pub fn system_create_particles(
        &mut self,
        id: i32,
        x: Real,
        y: Real,
        ptype: i32,
        colour: Option<i32>,
        number: i32,
        rand: &mut Random,
    ) {
        if let Some(ps) = self.systems.get_asset_mut(id) {
            ps.create_particles(x, y, ptype, colour, number, rand, &self.types);
        }
    }

    pub fn emitter_burst(&mut self, psid: i32, id: i32, parttype: i32, number: i32, rand: &mut Random) {
        if let Some(ps) = self.systems.get_asset_mut(psid) {
            if let Some(em) = ps.emitters.get_asset(id) {
                em.burst(parttype, number, &mut ps.particles, rand, &self.types);
            }
        }
    }

    pub fn destroy_system(&mut self, id: i32) {
        if self.systems.get_asset(id).is_some() {
            self.systems[id as usize] = None;
            self.draw_order.retain(|e| *e != id);
        }
    }

    pub fn draw_sort(&mut self) {
        self.draw_order.sort_by(|id1, id2| {
            let left = self.systems.get_asset(*id1).unwrap();
            let right = self.systems.get_asset(*id2).unwrap();

            right.depth.cmp_nan_first(&left.depth)
        });
    }

    pub fn iter_by_drawing(&self) -> PSIterDrawOrder {
        PSIterDrawOrder(0)
    }

    pub fn create_type(&mut self) -> i32 {
        let pt = Box::new(ParticleType::new());
        if let Some(id) = self.types.iter().position(|x| x.is_none()) {
            self.types[id] = Some(pt);
            id as i32
        } else {
            self.types.push(Some(pt));
            self.types.len() as i32 - 1
        }
    }

    pub fn get_type(&self, id: i32) -> Option<&Box<ParticleType>> {
        self.types.get_asset(id)
    }

    pub fn get_type_mut(&mut self, id: i32) -> Option<&mut Box<ParticleType>> {
        self.types.get_asset_mut(id)
    }

    pub fn destroy_type(&mut self, id: i32) {
        if self.types.get_asset(id).is_some() {
            self.types[id as usize] = None;
        }
    }

    pub fn get_dnd_system_mut(&mut self) -> &mut Box<System> {
        if self.get_system(self.dnd_system).is_none() {
            self.dnd_system = self.create_system();
        }
        self.get_system_mut(self.dnd_system).unwrap()
    }

    pub fn destroy_dnd_system(&mut self) {
        self.destroy_system(self.dnd_system);
        self.dnd_system = -1;
    }

    pub fn clear_dnd_system(&mut self) {
        if let Some(ps) = self.get_system_mut(self.dnd_system) {
            ps.particles.clear();
        }
    }

    pub fn get_dnd_type_mut(&mut self, id: usize) -> &mut Box<ParticleType> {
        if self.get_type(self.dnd_types[id]).is_none() {
            self.dnd_types[id] = self.create_type();
        }
        self.get_type_mut(self.dnd_types[id]).unwrap()
    }

    pub fn dnd_type_secondary(
        &mut self,
        id: usize,
        step_type: usize,
        step_number: i32,
        death_type: usize,
        death_number: i32,
    ) {
        let step_type = self.dnd_types[step_type];
        let death_type = self.dnd_types[death_type];
        let pt = self.get_dnd_type_mut(id);
        pt.step_type = step_type;
        pt.step_number = step_number;
        pt.death_type = death_type;
        pt.death_number = death_number;
    }

    pub fn get_dnd_emitter_mut(&mut self, id: usize) -> &mut Emitter {
        self.get_dnd_system_mut(); // make sure the system is there so we can smartly borrow it
        let id = &mut self.dnd_emitters[id];
        let ps = self.systems.get_asset_mut(self.dnd_system).unwrap();
        if ps.emitters.get_asset(*id).is_none() {
            *id = {
                let em = Emitter::new();
                if let Some(id) = ps.emitters.iter().position(|x| x.is_none()) {
                    ps.emitters[id] = Some(em);
                    id as i32
                } else {
                    ps.emitters.push(Some(em));
                    ps.emitters.len() as i32 - 1
                }
            };
        }
        ps.emitters.get_asset_mut(*id).unwrap()
    }

    pub fn destroy_dnd_emitter(&mut self, id: usize) {
        let em_id = self.dnd_emitters[id];
        if let Some(ps) = self.systems.get_asset_mut(self.dnd_system) {
            let em = &mut ps.emitters[em_id as usize];
            if em.is_some() {
                *em = None;
            }
        }
        self.dnd_emitters[id] = -1;
    }

    pub fn dnd_emitter_stream(&mut self, id: usize, parttype: usize, number: i32) {
        let ptype = self.dnd_types[parttype];
        let em = self.get_dnd_emitter_mut(id);
        em.ptype = ptype;
        em.number = number;
    }

    pub fn dnd_emitter_burst(&mut self, id: usize, parttype: usize, number: i32, rand: &mut Random) {
        self.emitter_burst(self.dnd_system, self.dnd_emitters[id], self.dnd_types[parttype], number, rand);
    }

    pub fn create_effect(
        &mut self,
        kind: EffectType,
        x: Real,
        y: Real,
        size: EffectSize,
        col: i32,
        below: bool,
        fps_mod: Real,
        room_width: i32,
        room_height: i32,
        rand: &mut Random,
    ) {
        if self.effects.is_none() {
            let system_below = self.create_system();
            let system_above = self.create_system();
            self.effects = Some(EffectManager {
                system_below,
                system_above,
                explosion_types: [
                    self.create_type(),
                    self.create_type(),
                    self.create_type(),
                    self.create_type(),
                    self.create_type(),
                    self.create_type(),
                ],
                ring_types: [self.create_type(), self.create_type(), self.create_type()],
                ellipse_types: [self.create_type(), self.create_type(), self.create_type()],
                firework_types: [self.create_type(), self.create_type(), self.create_type()],
                smoke_types: [self.create_type(), self.create_type(), self.create_type()],
                smokeup_types: [self.create_type(), self.create_type(), self.create_type()],
                star_types: [self.create_type(), self.create_type(), self.create_type()],
                spark_types: [self.create_type(), self.create_type(), self.create_type()],
                flare_types: [self.create_type(), self.create_type(), self.create_type()],
                cloud_types: [self.create_type(), self.create_type(), self.create_type()],
                rain_type: self.create_type(),
                snow_type: self.create_type(),
            });
            self.get_system_mut(system_below).unwrap().depth = 100000.into();
            self.get_system_mut(system_above).unwrap().depth = (-100000).into();
        }
        let effects = self.effects.as_ref().unwrap();
        let system = if below { effects.system_below } else { effects.system_above };
        let size_num = match size {
            EffectSize::Small => 0,
            EffectSize::Medium => 1,
            EffectSize::Large => 2,
        };
        match kind {
            EffectType::Explosion => {
                let (id1, id2) = (effects.explosion_types[size_num * 2], effects.explosion_types[size_num * 2 + 1]);
                let pt = self.types.get_asset_mut(id1).unwrap();
                pt.graphic = ParticleGraphic::Shape(10);
                pt.size_wiggle = 0.into();
                pt.ang_min = 0.into();
                pt.ang_max = 360.into();
                pt.ang_incr = 0.into();
                pt.ang_wiggle = 0.into();
                pt.ang_relative = false;
                pt.dir_min = 0.into();
                pt.dir_max = 360.into();
                pt.dir_incr = 0.into();
                pt.dir_wiggle = 0.into();
                pt.speed_wiggle = 0.into();
                pt.alpha1 = 0.6.into();
                pt.alpha2 = 0.3.into();
                pt.alpha3 = 0.into();

                match size {
                    EffectSize::Small => {
                        pt.size_min = 0.1.into();
                        pt.size_incr = fps_mod * 0.05.into();
                        pt.speed_min = fps_mod * 2.into();
                        pt.speed_incr = fps_mod * (-0.1).into();
                        pt.life_min = (Real::from(10) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(15) / fps_mod).round().to_i32();
                    },
                    EffectSize::Medium => {
                        pt.size_min = 0.3.into();
                        pt.size_incr = fps_mod * 0.1.into();
                        pt.speed_min = fps_mod * 4.into();
                        pt.speed_incr = fps_mod * (-0.18).into();
                        pt.life_min = (Real::from(12) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(17) / fps_mod).round().to_i32();
                    },
                    EffectSize::Large => {
                        pt.size_min = 0.4.into();
                        pt.size_incr = fps_mod * 0.2.into();
                        pt.speed_min = fps_mod * 7.into();
                        pt.speed_incr = fps_mod * (-0.2).into();
                        pt.life_min = (Real::from(15) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(20) / fps_mod).round().to_i32();
                    },
                }
                pt.size_max = pt.size_min;
                pt.speed_max = pt.speed_min;

                let pt = self.types.get_asset_mut(id2).unwrap();

                pt.graphic = ParticleGraphic::Shape(10);
                pt.ang_min = 0.into();
                pt.ang_max = 360.into();
                pt.ang_incr = 0.into();
                pt.ang_wiggle = 0.into();
                pt.ang_relative = false;
                pt.dir_min = 0.into();
                pt.dir_max = 360.into();
                pt.dir_incr = 0.into();
                pt.dir_wiggle = 0.into();
                pt.speed_wiggle = 0.into();
                pt.alpha1 = 0.8.into();
                pt.alpha2 = 0.4.into();
                pt.alpha3 = 0.into();

                match size {
                    EffectSize::Small => {
                        pt.size_min = 0.1.into();
                        pt.size_incr = fps_mod * 0.1.into();
                        pt.life_min = (Real::from(15) / fps_mod).round().to_i32();
                    },
                    EffectSize::Medium => {
                        pt.size_min = 0.3.into();
                        pt.size_incr = fps_mod * 0.2.into();
                        pt.life_min = (Real::from(17) / fps_mod).round().to_i32();
                    },
                    EffectSize::Large => {
                        pt.size_min = 0.4.into();
                        pt.size_incr = fps_mod * 0.4.into();
                        pt.life_min = (Real::from(20) / fps_mod).round().to_i32();
                    },
                }

                pt.size_max = pt.size_min;
                pt.life_max = pt.life_min;
                self.system_create_particles(system, x, y, id1, Some(col), 20, rand);
                self.system_create_particles(system, x, y, id2, Some(0), 1, rand);
            },
            EffectType::Ring => {
                let id = effects.ring_types[size_num];
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic = ParticleGraphic::Shape(6);
                pt.alpha1 = 1.into();
                pt.alpha2 = 0.5.into();
                pt.alpha3 = 0.into();
                pt.size_min = 0.into();
                pt.size_max = 0.into();
                pt.size_wiggle = 0.into();
                match size {
                    EffectSize::Small => {
                        pt.size_incr = fps_mod * 0.15.into();
                        pt.life_min = (Real::from(10) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(12) / fps_mod).round().to_i32();
                    },
                    EffectSize::Medium => {
                        pt.size_incr = fps_mod * 0.25.into();
                        pt.life_min = (Real::from(13) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(15) / fps_mod).round().to_i32();
                    },
                    EffectSize::Large => {
                        pt.size_incr = fps_mod * 0.4.into();
                        pt.life_min = (Real::from(18) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(20) / fps_mod).round().to_i32();
                    },
                }
                self.system_create_particles(system, x, y, id, Some(col), 1, rand);
            },
            EffectType::Ellipse => {
                let id = effects.ellipse_types[size_num];
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic = ParticleGraphic::Shape(6);
                pt.alpha1 = 1.into();
                pt.alpha2 = 0.5.into();
                pt.alpha3 = 0.into();
                pt.size_min = 0.into();
                pt.size_max = 0.into();
                pt.size_wiggle = 0.into();
                pt.xscale = 1.into();
                pt.yscale = 0.5.into();
                match size {
                    EffectSize::Small => {
                        pt.size_incr = fps_mod * 0.2.into();
                        pt.life_min = (Real::from(10) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(12) / fps_mod).round().to_i32();
                    },
                    EffectSize::Medium => {
                        pt.size_incr = fps_mod * 0.35.into();
                        pt.life_min = (Real::from(13) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(15) / fps_mod).round().to_i32();
                    },
                    EffectSize::Large => {
                        pt.size_incr = fps_mod * 0.6.into();
                        pt.life_min = (Real::from(18) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(20) / fps_mod).round().to_i32();
                    },
                }
                self.system_create_particles(system, x, y, id, Some(col), 1, rand);
            },
            EffectType::Firework => {
                let id = effects.firework_types[size_num];
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic = ParticleGraphic::Shape(8);
                pt.size_min = 0.1.into();
                pt.size_max = 0.2.into();
                pt.size_incr = 0.into();
                pt.size_wiggle = 0.into();
                pt.speed_min = fps_mod * 0.5.into();
                pt.speed_incr = 0.into();
                pt.speed_wiggle = 0.into();
                pt.dir_min = 0.into();
                pt.dir_max = 360.into();
                pt.dir_incr = 0.into();
                pt.dir_wiggle = 0.into();
                pt.alpha1 = 1.into();
                pt.alpha2 = 0.7.into();
                pt.alpha3 = 0.4.into();
                pt.grav_dir = 270.into();
                let number = match size {
                    EffectSize::Small => {
                        pt.speed_max = fps_mod * 3.into();
                        pt.life_min = (Real::from(15) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(25) / fps_mod).round().to_i32();
                        pt.grav_amount = 0.1.into();
                        75
                    },
                    EffectSize::Medium => {
                        pt.speed_max = fps_mod * 6.into();
                        pt.life_min = (Real::from(20) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(30) / fps_mod).round().to_i32();
                        pt.grav_amount = 0.15.into();
                        150
                    },
                    EffectSize::Large => {
                        pt.speed_max = fps_mod * 8.into();
                        pt.life_min = (Real::from(30) / fps_mod).round().to_i32();
                        pt.life_max = (Real::from(40) / fps_mod).round().to_i32();
                        pt.grav_amount = 0.17.into();
                        250
                    },
                };
                self.system_create_particles(system, x, y, id, Some(col), number, rand);
            },
            EffectType::Smoke => {
                let id = effects.smoke_types[size_num];
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic = ParticleGraphic::Shape(10);
                pt.size_incr = fps_mod * (-0.01).into();
                pt.size_wiggle = 0.into();
                pt.alpha1 = 0.4.into();
                pt.alpha2 = 0.2.into();
                pt.alpha3 = 0.into();
                match size {
                    EffectSize::Small => {
                        pt.size_min = 0.2.into();
                        pt.size_max = 0.4.into();
                        pt.life_min = (Real::from(25) / fps_mod).round().to_i32();
                        pt.life_max = pt.life_min;
                        for _ in 0..6 {
                            let dx = rand.next_int(9) - 5;
                            let dy = rand.next_int(9) - 5;
                            self.system_create_particles(system, x + dx.into(), y + dy.into(), id, Some(col), 1, rand);
                        }
                    },
                    EffectSize::Medium => {
                        pt.size_min = 0.4.into();
                        pt.size_max = 0.7.into();
                        pt.life_min = (Real::from(30) / fps_mod).round().to_i32();
                        pt.life_max = pt.life_min;
                        for _ in 0..11 {
                            let dx = rand.next_int(29) - 15;
                            let dy = rand.next_int(29) - 15;
                            self.system_create_particles(system, x + dx.into(), y + dy.into(), id, Some(col), 1, rand);
                        }
                    },
                    EffectSize::Large => {
                        pt.size_min = 0.4.into();
                        pt.size_max = 1.into();
                        pt.life_min = (Real::from(50) / fps_mod).round().to_i32();
                        pt.life_max = pt.life_min;
                        for _ in 0..16 {
                            let dx = rand.next_int(59) - 30;
                            let dy = rand.next_int(59) - 30;
                            self.system_create_particles(system, x + dx.into(), y + dy.into(), id, Some(col), 1, rand);
                        }
                    },
                }
            },
            EffectType::SmokeUp => {
                let id = effects.smokeup_types[size_num];
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic = ParticleGraphic::Shape(10);
                pt.size_incr = fps_mod * (-0.01).into();
                pt.size_wiggle = 0.into();
                pt.alpha1 = 0.4.into();
                pt.alpha2 = 0.2.into();
                pt.alpha3 = 0.into();
                pt.speed_incr = 0.into();
                pt.speed_wiggle = 0.into();
                pt.dir_min = 90.into();
                pt.dir_max = 90.into();
                pt.dir_incr = 0.into();
                pt.dir_wiggle = 0.into();
                match size {
                    EffectSize::Small => {
                        pt.size_min = 0.2.into();
                        pt.size_max = 0.4.into();
                        pt.speed_min = fps_mod * 3.into();
                        pt.speed_max = fps_mod * 4.into();
                        pt.life_min = (Real::from(25) / fps_mod).round().to_i32();
                        pt.life_max = pt.life_min;
                        for _ in 0..6 {
                            let dx = rand.next_int(9) - 5;
                            let dy = rand.next_int(9) - 5;
                            self.system_create_particles(system, x + dx.into(), y + dy.into(), id, Some(col), 1, rand);
                        }
                    },
                    EffectSize::Medium => {
                        pt.size_min = 0.4.into();
                        pt.size_max = 0.7.into();
                        pt.speed_min = fps_mod * 5.into();
                        pt.speed_max = fps_mod * 6.into();
                        pt.life_min = (Real::from(30) / fps_mod).round().to_i32();
                        pt.life_max = pt.life_min;
                        for _ in 0..11 {
                            let dx = rand.next_int(29) - 15;
                            let dy = rand.next_int(29) - 15;
                            self.system_create_particles(system, x + dx.into(), y + dy.into(), id, Some(col), 1, rand);
                        }
                    },
                    EffectSize::Large => {
                        pt.size_min = 0.4.into();
                        pt.size_max = 1.into();
                        pt.speed_min = fps_mod * 6.into();
                        pt.speed_max = fps_mod * 7.into();
                        pt.life_min = (Real::from(50) / fps_mod).round().to_i32();
                        pt.life_max = pt.life_min;
                        for _ in 0..16 {
                            let dx = rand.next_int(59) - 30;
                            let dy = rand.next_int(59) - 30;
                            self.system_create_particles(system, x + dx.into(), y + dy.into(), id, Some(col), 1, rand);
                        }
                    },
                }
            },
            EffectType::Star => {
                let id = effects.star_types[size_num];
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic = ParticleGraphic::Shape(4);
                pt.size_wiggle = 0.into();
                pt.ang_min = 0.into();
                pt.ang_max = 360.into();
                pt.ang_incr = 0.into();
                pt.ang_wiggle = 0.into();
                pt.ang_relative = false;
                match size {
                    EffectSize::Small => {
                        pt.size_min = 0.4.into();
                        pt.size_max = 0.3.into(); // yes this way round
                        pt.size_incr = fps_mod * (-0.02).into();
                        pt.life_min = (Real::from(20) / fps_mod).round().to_i32();
                        pt.life_max = pt.life_min;
                    },
                    EffectSize::Medium => {
                        pt.size_min = 0.75.into();
                        pt.size_max = 0.75.into();
                        pt.size_incr = fps_mod * (-0.03).into();
                        pt.life_min = (Real::from(25) / fps_mod).round().to_i32();
                        pt.life_max = pt.life_min;
                    },
                    EffectSize::Large => {
                        pt.size_min = 1.2.into();
                        pt.size_max = 1.2.into();
                        pt.size_incr = fps_mod * (-0.04).into();
                        pt.life_min = (Real::from(30) / fps_mod).round().to_i32();
                        pt.life_max = pt.life_min;
                    },
                }
                self.system_create_particles(system, x, y, id, Some(col), 1, rand);
            },
            EffectType::Spark | EffectType::Flare => {
                let id = if kind == EffectType::Spark {
                    effects.spark_types[size_num]
                } else {
                    effects.flare_types[size_num]
                };
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic =
                    if kind == EffectType::Spark { ParticleGraphic::Shape(9) } else { ParticleGraphic::Shape(8) };
                pt.size_wiggle = 0.into();
                pt.ang_min = 0.into();
                pt.ang_max = 360.into();
                pt.ang_incr = 0.into();
                pt.ang_wiggle = 0.into();
                pt.ang_relative = false;
                match size {
                    EffectSize::Small => {
                        pt.size_min = 0.4.into();
                        pt.size_incr = Real::from(-0.02) / fps_mod;
                        pt.life_min = (Real::from(20) / fps_mod).round().to_i32();
                    },
                    EffectSize::Medium => {
                        pt.size_min = 0.75.into();
                        pt.size_incr = Real::from(-0.03) / fps_mod;
                        pt.life_min = (Real::from(25) / fps_mod).round().to_i32();
                    },
                    EffectSize::Large => {
                        pt.size_min = 1.2.into();
                        pt.size_incr = Real::from(-0.04) / fps_mod;
                        pt.life_min = (Real::from(30) / fps_mod).round().to_i32();
                    },
                }
                pt.size_max = pt.size_min;
                pt.life_max = pt.life_min;
                self.system_create_particles(system, x, y, id, Some(col), 1, rand);
            },
            EffectType::Cloud => {
                let id = effects.cloud_types[size_num];
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic = ParticleGraphic::Shape(10);
                pt.size_incr = 0.into();
                pt.size_wiggle = 0.into();
                pt.xscale = 1.into();
                pt.yscale = 0.5.into();
                pt.alpha1 = 0.into();
                pt.alpha2 = 0.3.into();
                pt.alpha3 = 0.into();
                pt.life_min = (Real::from(100) / fps_mod).round().to_i32();
                pt.life_max = pt.life_min;
                pt.size_min = match size {
                    EffectSize::Small => 2.into(),
                    EffectSize::Medium => 4.into(),
                    EffectSize::Large => 8.into(),
                };
                pt.size_max = pt.size_min;
                self.system_create_particles(system, x, y, id, Some(col), 1, rand);
            },
            EffectType::Rain => {
                let id = effects.rain_type;
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic = ParticleGraphic::Shape(3);
                pt.size_min = 0.2.into();
                pt.size_max = 0.3.into();
                pt.size_incr = 0.into();
                pt.size_wiggle = 0.into();
                pt.ang_min = 0.into();
                pt.ang_max = 0.into();
                pt.ang_incr = 0.into();
                pt.ang_wiggle = 0.into();
                pt.ang_relative = true;
                pt.speed_min = fps_mod * 7.into();
                pt.speed_max = pt.speed_min;
                pt.speed_incr = 0.into();
                pt.speed_wiggle = 0.into();
                pt.dir_min = 260.into();
                pt.dir_max = 260.into();
                pt.dir_incr = 0.into();
                pt.dir_wiggle = 0.into();
                pt.alpha1 = 0.4.into();
                pt.alpha2 = 0.4.into();
                pt.alpha3 = 0.4.into();
                pt.life_min = (Real::from(0.2) / fps_mod * room_height.into()).round().to_i32();
                pt.life_max = pt.life_min;
                let number = match size {
                    EffectSize::Small => 2,
                    EffectSize::Medium => 5,
                    EffectSize::Large => 9,
                };
                for _ in 0..number {
                    self.system_create_particles(
                        system,
                        Real::from(rand.next(1.2)) * room_width.into(),
                        Real::from(rand.next_int(19) - 30),
                        id,
                        Some(col),
                        1,
                        rand,
                    );
                }
            },
            EffectType::Snow => {
                let id = effects.snow_type;
                let pt = self.get_type_mut(id).unwrap();
                pt.graphic = ParticleGraphic::Shape(13);
                pt.size_min = 0.1.into();
                pt.size_max = 0.25.into();
                pt.size_incr = 0.into();
                pt.size_wiggle = 0.into();
                pt.alpha1 = 0.6.into();
                pt.alpha2 = 0.6.into();
                pt.alpha3 = 0.6.into();
                pt.ang_min = 0.into();
                pt.ang_max = 360.into();
                pt.ang_incr = 0.into();
                pt.ang_wiggle = 0.into();
                pt.ang_relative = false;
                pt.speed_min = fps_mod * 2.5.into();
                pt.speed_max = fps_mod * 3.into();
                pt.speed_incr = 0.into();
                pt.speed_wiggle = 0.into();
                pt.dir_min = 240.into();
                pt.dir_max = 300.into();
                pt.dir_incr = 0.into();
                pt.dir_wiggle = 20.into();
                pt.life_min = (Real::from(0.5) / fps_mod * room_height.into()).round().to_i32();
                pt.life_max = pt.life_min;
                let number = match size {
                    EffectSize::Small => 1,
                    EffectSize::Medium => 3,
                    EffectSize::Large => 7,
                };
                for _ in 0..number {
                    self.system_create_particles(
                        system,
                        Real::from(rand.next(1.2)) * room_width.into() - 60.into(),
                        Real::from(rand.next_int(19) - 30),
                        id,
                        Some(col),
                        1,
                        rand,
                    );
                }
            },
        }
    }

    pub fn effect_clear(&mut self) {
        if let Some((system_below, system_above)) = self.effects.as_ref().map(|e| (e.system_below, e.system_above)) {
            self.get_system_mut(system_below).map(|ps| ps.particles.clear());
            self.get_system_mut(system_above).map(|ps| ps.particles.clear());
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    Explosion,
    Ring,
    Ellipse,
    Firework,
    Smoke,
    SmokeUp,
    Star,
    Spark,
    Flare,
    Cloud,
    Rain,
    Snow,
}

pub enum EffectSize {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub colour: ParticleColour,
    pub alpha1: Real,
    pub alpha2: Real,
    pub alpha3: Real,
    pub additive_blending: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParticleGraphic {
    Sprite { sprite: i32, animat: bool, stretch: bool, random: bool },
    Shape(i32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParticleColour {
    One(i32),
    Two(i32, i32),
    Three(i32, i32, i32),
    RGB { rmin: i32, rmax: i32, gmin: i32, gmax: i32, bmin: i32, bmax: i32 },
    HSV { hmin: i32, hmax: i32, smin: i32, smax: i32, vmin: i32, vmax: i32 },
    Mix(i32, i32),
}

fn colour_lerp(c1: i32, c2: i32, lerp: Real) -> i32 {
    let c1_part = Real::from(1.0) - lerp;
    let c2_part = lerp;
    let (r1, r2) = (c1 & 0xff, c2 & 0xff);
    let (g1, g2) = ((c1 >> 8) & 0xff, (c2 >> 8) & 0xff);
    let (b1, b2) = ((c1 >> 16) & 0xff, (c2 >> 16) & 0xff);
    let r = (Real::from(r1) * c1_part + Real::from(r2) * c2_part).round().to_i32();
    let g = (Real::from(g1) * c1_part + Real::from(g2) * c2_part).round().to_i32();
    let b = (Real::from(b1) * c1_part + Real::from(b2) * c2_part).round().to_i32();
    r | (g << 8) | (b << 16)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    colour: i32,
    alpha: Real,
    size: Real,
    subimage: i32,
    random_start: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attractor {
    pub x: Real,
    pub y: Real,
    pub force: Real,
    pub dist: Real,
    pub kind: ForceKind,
    pub additive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForceKind {
    Constant,
    Linear,
    Quadratic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChangerKind {
    Motion,
    Shape,
    All,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deflector {
    pub xmin: Real,
    pub xmax: Real,
    pub ymin: Real,
    pub ymax: Real,
    pub kind: DeflectorKind,
    pub friction: Real,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeflectorKind {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Destroyer {
    pub xmin: Real,
    pub xmax: Real,
    pub ymin: Real,
    pub ymax: Real,
    pub shape: Shape,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
            colour: ParticleColour::One(0xffffff),
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

    pub fn draw(
        &self,
        renderer: &mut Renderer,
        assets: &Assets,
        types: &dyn GetAsset<Box<ParticleType>>,
        shapes: &Vec<AtlasRef>,
    ) {
        // TODO set texture lerp on just for this function
        let mut last_was_additive = false;
        let mut fix_blend = |new, renderer: &mut Renderer| {
            if new != last_was_additive {
                last_was_additive = new;
                match new {
                    false => renderer.set_blend_mode(BlendType::SrcAlpha, BlendType::InvSrcAlpha), // bm_normal
                    true => renderer.set_blend_mode(BlendType::SrcAlpha, BlendType::One),          // bm_add
                }
            }
        };
        if self.draw_old_to_new {
            for particle in &self.particles {
                types.get_asset(particle.ptype).map(|pt| fix_blend(pt.additive_blending, renderer));
                particle.draw(self.x, self.y, renderer, assets, types, shapes);
            }
        } else {
            for particle in self.particles.iter().rev() {
                types.get_asset(particle.ptype).map(|pt| fix_blend(pt.additive_blending, renderer));
                particle.draw(self.x, self.y, renderer, assets, types, shapes);
            }
        }
        fix_blend(false, renderer);
    }

    pub fn create_particles(
        &mut self,
        x: Real,
        y: Real,
        ptype: i32,
        colour: Option<i32>,
        number: i32,
        rand: &mut Random,
        types: &dyn GetAsset<Box<ParticleType>>,
    ) {
        for _ in 0..number {
            if let Some(particle) = Particle::new(ptype, x, y, rand, types, colour) {
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
                    self.create_particles(x, y, ptype.death_type, None, number, rand, types);
                } else {
                    // particle is alive
                    let mut number = ptype.step_number;
                    if number < 0 && rand.next_int((-number) as u32) == 0 {
                        number = 1;
                    }
                    self.create_particles(x, y, ptype.step_type, None, number, rand, types);
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
        colour_arg: Option<i32>,
    ) -> Option<Self> {
        if let Some(ptype) = types.get_asset(ptype_id) {
            let speed = Distribution::Linear.range(rand, ptype.speed_min.into(), ptype.speed_max.into()).into();
            let direction = Distribution::Linear.range(rand, ptype.dir_min.into(), ptype.dir_max.into()).into();
            let image_angle = Distribution::Linear.range(rand, ptype.ang_min.into(), ptype.ang_max.into()).into();
            let lifetime =
                Distribution::Linear.range(rand, ptype.life_min.into(), ptype.life_max.into()).round().to_i32();
            let colour = Self::init_colour(rand, &ptype.colour); // do this despite colour_arg for rng parity
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
                colour: colour_arg.unwrap_or(colour),
                alpha,
                size,
                subimage,
                random_start,
            })
        } else {
            None
        }
    }

    fn init_colour(rand: &mut Random, colour_type: &ParticleColour) -> i32 {
        match colour_type {
            ParticleColour::One(c) => *c,
            ParticleColour::Two(c, _) => *c,
            ParticleColour::Three(c, _, _) => *c,
            ParticleColour::RGB { rmin, rmax, gmin, gmax, bmin, bmax } => {
                let r = Distribution::Linear.range(rand, (*rmin).into(), (*rmax).into()).round().to_i32();
                let g = Distribution::Linear.range(rand, (*gmin).into(), (*gmax).into()).round().to_i32();
                let b = Distribution::Linear.range(rand, (*bmin).into(), (*bmax).into()).round().to_i32();
                r | (g << 8) | (b << 16)
            },
            ParticleColour::HSV { hmin, hmax, smin, smax, vmin, vmax } => {
                let h = Distribution::Linear.range(rand, (*hmin).into(), (*hmax).into()).round().to_i32();
                let s = Distribution::Linear.range(rand, (*smin).into(), (*smax).into()).round().to_i32();
                let v = Distribution::Linear.range(rand, (*vmin).into(), (*vmax).into()).round().to_i32();

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

                let out_r = ((r + m) * Real::from(255.0)).round().to_i32();
                let out_g = ((g + m) * Real::from(255.0)).round().to_i32();
                let out_b = ((b + m) * Real::from(255.0)).round().to_i32();
                out_r | (out_g << 8) | (out_b << 16)
            },
            ParticleColour::Mix(c1, c2) => colour_lerp(*c1, *c2, rand.next(1.0).into()),
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

            let mut dir_wiggle_factor = Real::from((self.timer + self.random_start * 3) % 24) / Real::from(6.0);
            if Real::from(2.0) < dir_wiggle_factor {
                dir_wiggle_factor = Real::from(4.0) - dir_wiggle_factor;
            }
            dir_wiggle_factor -= Real::from(1.0);
            let direction = self.direction + ptype.dir_wiggle * dir_wiggle_factor;

            let mut speed_wiggle_factor = Real::from((self.timer + self.random_start * 4) % 20) / Real::from(5.0);
            if Real::from(2.0) < speed_wiggle_factor {
                speed_wiggle_factor = Real::from(4.0) - speed_wiggle_factor;
            }
            speed_wiggle_factor -= Real::from(1.0);
            let speed = self.speed + ptype.speed_wiggle * speed_wiggle_factor;

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
            self.colour = match ptype.colour {
                ParticleColour::Two(c1, c2) => colour_lerp(c1, c2, halflife_elapsed / Real::from(2.0)),
                ParticleColour::Three(c1, c2, c3) => {
                    if halflife_elapsed < Real::from(1.0) {
                        colour_lerp(c1, c2, halflife_elapsed)
                    } else {
                        colour_lerp(c2, c3, halflife_elapsed - Real::from(1.0))
                    }
                },
                _ => self.colour,
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
        shapes: &Vec<AtlasRef>,
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
                    sprite.get_atlas_ref(subimage % sprite.frames.len() as i32)
                } else {
                    None
                }
            },
            ParticleGraphic::Shape(s) => usize::try_from(s).ok().and_then(|s| shapes.get(s).copied()),
        };
        let mut angle_wiggle_factor = ((self.timer + self.random_start * 2) % 16) as f64 / 4.0;
        if 2.0 < angle_wiggle_factor {
            angle_wiggle_factor = 4.0 - angle_wiggle_factor;
        }
        angle_wiggle_factor -= 1.0;
        let mut angle = self.image_angle + ptype.ang_wiggle * angle_wiggle_factor.into();
        if ptype.ang_relative {
            angle += self.direction;
        }
        let mut size_wiggle_factor = ((self.timer + self.random_start) % 16) as f64 / 4.0;
        if 2.0 < size_wiggle_factor {
            size_wiggle_factor = 4.0 - size_wiggle_factor;
        }
        size_wiggle_factor -= 1.0;
        let size = self.size + ptype.size_wiggle * size_wiggle_factor.into();
        if let Some(atlas_ref) = atlas_ref {
            renderer.draw_sprite(
                atlas_ref,
                (system_x + self.x).into(),
                (system_y + self.y).into(),
                (ptype.xscale * size).into(),
                (ptype.yscale * size).into(),
                angle.into(),
                self.colour.into(),
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
                    match self.kind {
                        DeflectorKind::Horizontal => {
                            particle.direction = (Real::from(180) - particle.direction).rem_euclid(360.into());
                            particle.x = particle.xprevious - (particle.x - particle.xprevious);
                        },
                        DeflectorKind::Vertical => {
                            particle.direction = (Real::from(360) - particle.direction).rem_euclid(360.into());
                            particle.y = particle.yprevious - (particle.y - particle.yprevious);
                        },
                    }
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
                    if let Some(new_part) = Particle::new(self.parttype2, particle.x, particle.y, rand, types, None) {
                        particle.ptype = new_part.ptype;
                        match self.kind {
                            ChangerKind::Motion => {
                                particle.speed = new_part.speed;
                                particle.direction = new_part.direction;
                            },
                            ChangerKind::Shape => {
                                particle.colour = new_part.colour;
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
                None,
            ) {
                particles.push(particle);
            }
        }
    }
}

pub fn load_shapes(atlases: &mut AtlasBuilder) -> Vec<AtlasRef> {
    let raw = include_bytes!("../../data/particles.dat");
    let mut atlas_refs = Vec::with_capacity(14);
    for i in 0..14 {
        let mut rgba = Box::new([255u8; 64 * 64 * 4]);
        for j in 0..64 * 64 {
            rgba[4 * j + 3] = raw[64 * 64 * i + j];
        }
        atlas_refs.push(atlases.texture(64, 64, 32, 32, rgba).expect("failed to pack particle shapes"));
    }
    atlas_refs
}
