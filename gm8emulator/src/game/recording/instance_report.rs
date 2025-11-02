use crate::{
    imgui_utils::*,
    instance::Field,
    game::{
        Game,
        recording::window::{Window, EmulatorContext},
    },
    render::atlas::AtlasRef,
};
use std::ops::Index;

pub struct InstanceReport {
    object_name: String,
    id: String,
    general_vars: [String; 7],
    physics_vars: [String; 13],
    image_vars: [String; 11],
    timeline_vars: [String; 5],
    alarms: Vec<String>,
    fields: Vec<ReportField>,
}

enum ReportField {
    Single(String),
    Array(String, Vec<String>),
}

pub struct InstanceReportWindow {
    instance_images: Vec<AtlasRef>,
    report_count: usize,
}

impl InstanceReport {
    pub fn new(game: &Game, id: i32) -> Option<Self> {
        use crate::game::GetAsset;
        if let Some((handle, instance)) = game.room.instance_list.get_by_instid(id).map(|x| (x, game.room.instance_list.get(x))) {
            instance.update_bbox(game.get_instance_mask_sprite(handle));
            let object_name = game.assets.objects.get_asset(instance.object_index.get())
                .map(|x| x.name.decode(game.encoding))
                .unwrap_or("<deleted object>".into());

            Some(Self {
                object_name: object_name.clone().into(),
                id: id.to_string(),
                general_vars: [
                    format!("object_index: {} ({})", instance.object_index.get(), object_name),
                    format!("x: {:.4}", instance.x.get()),
                    format!("y: {:.4}", instance.y.get()),
                    format!("xprevious: {:.4}", instance.xprevious.get()),
                    format!("yprevious: {:.4}", instance.yprevious.get()),
                    format!("xstart: {:.4}", instance.xstart.get()),
                    format!("ystart: {:.4}", instance.ystart.get()),
                ],
                physics_vars: [
                    format!("speed: {:.4}", instance.speed.get()),
                    format!("direction: {:.4}", instance.direction.get()),
                    format!("hspeed: {:.4}", instance.hspeed.get()),
                    format!("vspeed: {:.4}", instance.vspeed.get()),
                    format!("gravity: {:.4}", instance.gravity.get()),
                    format!("gravity_direction: {:.4}", instance.gravity_direction.get()),
                    format!("friction: {:.4}", instance.friction.get()),
                    format!("solid: {}", instance.solid.get()),
                    format!("persistent: {}", instance.persistent.get()),
                    format!("bbox_left: {}", instance.bbox_left.get()),
                    format!("bbox_right: {}", instance.bbox_right.get()),
                    format!("bbox_top: {}", instance.bbox_top.get()),
                    format!("bbox_bottom: {}", instance.bbox_bottom.get()),
                ],
                image_vars: [
                    format!(
                        "sprite_index: {} ({})",
                        instance.sprite_index.get(),
                        game.assets.sprites.get_asset(instance.sprite_index.get())
                            .map(|x| x.name.decode(game.encoding))
                            .unwrap_or("<deleted sprite>".into()),
                    ),
                    format!(
                        "mask_index: {} ({})",
                        instance.mask_index.get(),
                        game.assets.sprites.get_asset(instance.mask_index.get())
                            .map(|x| x.name.decode(game.encoding))
                            .unwrap_or("<same as sprite>".into()),
                    ),
                    format!("image_index: {:.4}", instance.image_index.get()),
                    format!("image_speed: {:.4}", instance.image_speed.get()),
                    format!("visible: {}", instance.visible.get()),
                    format!("depth: {:.4}", instance.depth.get()),
                    format!("image_xscale: {:.4}", instance.image_xscale.get()),
                    format!("image_yscale: {:.4}", instance.image_yscale.get()),
                    format!("image_angle: {:.4}", instance.image_angle.get()),
                    format!("image_blend: {}", instance.image_blend.get()),
                    format!("image_alpha: {:.4}", instance.image_alpha.get()),
                ],
                timeline_vars: [
                    format!(
                        "timeline_index: {} ({})",
                        instance.timeline_index.get(),
                        game.assets.timelines.get_asset(instance.timeline_index.get())
                            .map(|x| x.name.decode(game.encoding))
                            .unwrap_or("<deleted timeline>".into()),
                    ),
                    format!("timeline_running: {}", instance.timeline_running.get()),
                    format!("timeline_speed: {:.4}", instance.timeline_speed.get()),
                    format!("timeline_position: {:.4}", instance.timeline_position.get()),
                    format!("timeline_loop: {}", instance.timeline_loop.get()),
                ],
                alarms: instance.alarms.borrow().iter().map(|(id, time)| format!("alarm[{}]: {}", id, time)).collect(),
                fields: instance.fields.borrow().iter().map(|(id, field)| {
                    let field_name = game.compiler.get_field_name(*id).unwrap_or("<???>".into());
                    match field {
                        Field::Single(value) => ReportField::Single(format!("{}: {}", field_name, value)),
                        Field::Array(map) => ReportField::Array(
                            field_name,
                            map.iter().map(|(index, value)| format!("[{}]: {}", index, value)).collect()
                        ),
                    }
                }).collect(),
            })
        } else {
            None
        }
    }
}

// Instance-watcher windows
impl Window for InstanceReportWindow {
    fn name(&self) -> String {
        format!("Instance Reports ({})", self.report_count)
    }

    fn show_window(&mut self, info: &mut EmulatorContext) {
        let previous_len = info.config.watched_ids.len();
        {
            let EmulatorContext {
                game,
                frame,
                config,
                instance_reports,
                ..
            } = info;

            self.instance_images.clear();
            self.instance_images.reserve(config.watched_ids.len());

            config.watched_ids.retain(|id| {
                let report = instance_reports.iter().find(|(i, _)| i == id);
                self.instance_window(*frame, *game, *id, report)
            });
        }

        if info.config.watched_ids.len() != previous_len {
            info.update_instance_reports();
            info.config.save();
        }
        self.report_count = info.config.watched_ids.len();
    }

    fn is_open(&self) -> bool { true }
}

impl InstanceReportWindow {
    pub fn new() -> Self {
        Self {
            instance_images: Vec::new(),
            report_count: 0,
        }
    }

    fn show_text(frame: &imgui::Ui, text: &String) {
        frame.text(text);
        if frame.is_mouse_clicked(imgui::MouseButton::Middle) && frame.is_item_hovered() {
            let value_str = text.index(text.find(':').unwrap_or(usize::MAX).wrapping_add(1)..);
            frame.set_clipboard_text(value_str);
        }
    }

    /// Creates the window for the instance.
    /// Returns whether or not the window is open
    fn instance_window(&mut self, frame: &imgui::Ui, game: &mut Game, id: i32, instance_report: Option<&(i32, Option<InstanceReport>)>) -> bool {
        let mut open = true;
        frame.window(format!("Instance {}", id))
            .opened(&mut open)
            .build(|| {
                if let Some((_, Some(report))) = instance_report {
                    Self::show_text(frame, &report.object_name);
                    Self::show_text(frame, &report.id);
                    frame.text("");
                    if let Some(node) = frame.tree_node("General Variables") {
                        report.general_vars.iter().for_each(|s| Self::show_text(frame, s));
                        node.end();
                    }
                    if let Some(node) = frame.tree_node("Physics Variables") {
                        report.physics_vars.iter().for_each(|s| Self::show_text(frame, s));
                        node.end();
                    }
                    if let Some(node) = frame.tree_node("Image Variables") {
                        report.image_vars.iter().for_each(|s| Self::show_text(frame, s));
                        node.end();
                    }
                    if let Some(node) = frame.tree_node("Timeline Variables") {
                        report.timeline_vars.iter().for_each(|s| Self::show_text(frame, s));
                        node.end();
                    }
                    if let Some(node) = frame.tree_node("Alarms") {
                        report.alarms.iter().for_each(|s| Self::show_text(frame, s));
                        node.end();
                    }
                    if let Some(node) = frame.tree_node("Fields") {
                        report.fields.iter().for_each(|f| match f {
                            ReportField::Single(s) => Self::show_text(frame, s),
                            ReportField::Array(label, array) => {
                                if let Some(node2) = frame.tree_node(label) {
                                    array.iter().for_each(|s| Self::show_text(frame, s));
                                    node2.end();
                                }
                            },
                        });
                        node.end();
                    }
                    self.add_sprite_image(frame, game, id);
                } else {
                    frame.text_centered("<deleted instance>", Vec2(160.0, 35.0));
                }
            }
        );
        open
    }

    fn add_sprite_image(&mut self, frame: &imgui::Ui, game: &mut Game, id: i32) {
        if let Some(handle) = game.room.instance_list.get_by_instid(id) {
            use crate::game::GetAsset;
            let instance = game.room.instance_list.get(handle);
            if let Some((sprite, atlas_ref)) =
                game.assets.sprites.get_asset(instance.sprite_index.get()).and_then(|x| {
                    x.get_atlas_ref(instance.image_index.get().floor().to_i32()).map(|y| (x, y))
                })
            {
                if sprite.width <= 48 && sprite.height <= 48 {
                    let i = self.instance_images.len();
                    self.instance_images.push(atlas_ref);
                    let Vec2(win_x, win_y) = frame.window_pos().into();
                    let win_w = frame.window_size()[0];
                    let center_x = win_x + win_w - 28.0;
                    let center_y = win_y + 46.0;
                    let min_x = center_x - (sprite.width / 2) as f32;
                    let min_y = center_y - (sprite.height / 2) as f32;
                    unsafe {
                        imgui::sys::ImDrawList_AddImage(
                            imgui::sys::igGetWindowDrawList(),
                            self.instance_images.as_mut_ptr().add(i) as _,
                            imgui::sys::ImVec2 { x: min_x, y: min_y },
                            imgui::sys::ImVec2 { 
                                x: min_x + sprite.width as f32,
                                y: min_y + sprite.height as f32
                            },
                            imgui::sys::ImVec2 { x: 0.0, y: 0.0 },
                            imgui::sys::ImVec2 { x: 1.0, y: 1.0 },
                            instance.image_blend.get() as u32 | 0xFF000000,
                        );
                    }
                }
            }
        }
    }
}
