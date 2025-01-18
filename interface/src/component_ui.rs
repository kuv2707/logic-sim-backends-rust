use bsim_engine::{
    components::Gate,
    types::{CompType, CLOCK_PIN, ID},
};
use egui::{vec2, Button, Color32, Rect, Response, Sense, TextEdit, Ui, Vec2};

use crate::{app::DisplayData, consts::GRID_UNIT_SIZE, update_ops::UpdateOps};

pub fn paint_component(
    disp_params: &mut DisplayData,
    ui: &mut Ui,
    gate: &mut Gate,
    from: &mut Option<ID>,
) -> Vec<UpdateOps> {
    let mut emit_evts = Vec::<UpdateOps>::new();
    let container = egui::Rect::from_min_size(
        disp_params.loc,
        egui::vec2(
            disp_params.scale * GRID_UNIT_SIZE,
            disp_params.scale * GRID_UNIT_SIZE,
        ),
    );
    let al = ui.allocate_rect(container, Sense::click_and_drag());
    if ui.is_rect_visible(container) {
        let painter = ui.painter();
        painter.rect_filled(
            container,
            5.0,
            if gate.state {
                egui::Color32::from_rgb(144, 238, 144)
            } else {
                egui::Color32::from_rgb(255, 102, 102)
            },
        );
    }

    for (i, pos) in disp_params.input_locs.iter().enumerate() {
        if add_pin_btn(container, *pos, ui, false).clicked() {
            let bks = is_ctrl_pressed(ui);
            match from {
                Some(id) => {
                    emit_evts.push(if bks {
                        UpdateOps::Disconnect(*id, (disp_params.id, i + 1))
                    } else {
                        UpdateOps::Connect(*id, (disp_params.id, i + 1))
                    });
                }
                None => {}
            }
        }
    }

    if add_pin_btn(
        container,
        disp_params.output_loc,
        ui,
        match from {
            Some(id) => disp_params.id == *id,
            None => false,
        },
    )
    .clicked()
    {
        *from = match from {
            Some(id) => {
                if *id == disp_params.id {
                    // clicking on the same gate twice deselects it
                    None
                } else {
                    Some(disp_params.id)
                }
            }
            None => Some(disp_params.id),
        }
    }
    if disp_params.is_clocked {
        if add_pin_btn(container, vec2(50.0, 90.0), ui, false).clicked() {
            let bks = is_ctrl_pressed(ui);
            match from {
                Some(id) => {
                    emit_evts.push(if bks {
                        UpdateOps::Disconnect(*id, (disp_params.id, CLOCK_PIN))
                    } else {
                        UpdateOps::Connect(*id, (disp_params.id, CLOCK_PIN))
                    });
                }
                None => {}
            }
        }
    }

    let ted = TextEdit::singleline(&mut gate.label).hint_text("label");
    ui.put(
        Rect::from_center_size(container.center(), vec2(50.0, 10.0)),
        ted,
    );
    ui.put(
        Rect::from_center_size(container.center() + vec2(0., 20.0), vec2(50.0, 10.0)),
        Button::new(&gate.name),
    );

    let r = ui.interact(container, al.id, Sense::click_and_drag());
    if r.dragged() {
        let k = ui.input(|i| i.pointer.interact_pos().unwrap());
        disp_params.loc.x = ((k.x / GRID_UNIT_SIZE) as usize) as f32 * GRID_UNIT_SIZE;
        disp_params.loc.y = ((k.y / GRID_UNIT_SIZE) as usize) as f32 * GRID_UNIT_SIZE;
    }

    if r.clicked() {
        if is_ctrl_pressed(ui) {
            emit_evts.push(UpdateOps::Remove(gate.id));
        } else if gate.comp_type == CompType::Input {
            emit_evts.push(UpdateOps::SetState(disp_params.id, !gate.state));
        }
    }
    return emit_evts;
}

fn add_pin_btn(container: Rect, pos: Vec2, ui: &mut Ui, selected: bool) -> Response {
    let brect = Rect::from_center_size(container.min + pos, vec2(20.0, 20.0));
    let mut btn = Button::new("");
    if selected {
        btn = btn.fill(Color32::YELLOW)
    }
    ui.put(brect, btn)
}

fn is_ctrl_pressed(ui: &Ui) -> bool {
    ui.input(|rd| rd.modifiers.mac_cmd || rd.modifiers.ctrl)
}
