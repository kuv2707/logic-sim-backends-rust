use bsim_engine::{
    components::Gate,
    types::{CompType, CLOCK_PIN, ID},
};
use egui::{vec2, Button, Color32, Rect, Response, Sense, Stroke, TextEdit, Ui, Vec2};

use crate::{
    app::DisplayData,
    consts::{GRID_UNIT_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH},
    update_ops::UpdateOps,
};

pub fn paint_component(
    disp_params: &mut DisplayData,
    ui: &mut Ui,
    gate: &mut Gate,
    from: &mut Option<ID>,
) -> Vec<UpdateOps> {
    let mut emit_evts = Vec::<UpdateOps>::new();
    let container = egui::Rect::from_min_size(
        disp_params.logical_loc * GRID_UNIT_SIZE,
        disp_params.size * GRID_UNIT_SIZE,
    );
    let al = ui.allocate_rect(container, Sense::click_and_drag());
    if ui.is_rect_visible(container) {
        let painter = ui.painter();
        painter.rect_filled(
            container,
            8.0,
            if gate.state {
                egui::Color32::from_rgb(144, 238, 144)
            } else {
                egui::Color32::from_rgb(255, 102, 102)
            },
        );
        painter.rect_stroke(container, 8.0, Stroke::new(2.0, Color32::BLACK));
    }

    for (i, pos) in disp_params.input_locs_rel.iter().enumerate() {
        if i == 0 && !disp_params.is_clocked {
            continue;
        }
        if add_pin_btn(container, *pos, ui, false).clicked() {
            let bks = is_ctrl_pressed(ui);
            match from {
                Some(id) => {
                    emit_evts.push(if bks {
                        UpdateOps::Disconnect(*id, (disp_params.id, i))
                    } else {
                        UpdateOps::Connect(*id, (disp_params.id, i))
                    });
                }
                None => {}
            }
        }
    }

    if add_pin_btn(
        container,
        disp_params.output_loc_rel,
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
        let mut newx = (k.x / GRID_UNIT_SIZE) - disp_params.size.x / 2.0;
        if newx + disp_params.size.x >= WINDOW_WIDTH {
            newx = WINDOW_WIDTH - disp_params.size.x;
        }
        if newx < 0.0 {
            newx = 0.0;
        }

        let mut newy = (k.y / GRID_UNIT_SIZE) - disp_params.size.y / 2.0;
        if newy + disp_params.size.y >= WINDOW_HEIGHT {
            newy = WINDOW_HEIGHT - disp_params.size.y;
        }
        if newy < 0.0 {
            newy = 0.0;
        }

        disp_params.logical_loc.x = newx;
        disp_params.logical_loc.y = newy;
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
