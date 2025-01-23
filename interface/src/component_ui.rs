use bsim_engine::{
    components::Gate,
    types::{CompType, ID},
};
use egui::{vec2, Button, Color32, Rect, Response, Sense, Stroke, TextEdit, Ui, Vec2};

use crate::{
    consts::{GRID_UNIT_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH},
    display_elems::DisplayData,
    update_ops::{CircuitUpdateOps, UiUpdateOps},
};

pub const PIN_BTN_SIZE: f32 = 20.0;

pub fn paint_component(
    disp_params: &mut DisplayData,
    ui: &mut Ui,
    gate: &mut Gate,
    from: &mut Option<ID>,
) -> (Vec<CircuitUpdateOps>, Vec<UiUpdateOps>) {
    let mut ckt_evts = Vec::<CircuitUpdateOps>::new();
    let mut ui_evts = Vec::<UiUpdateOps>::new();
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
        if add_pin_btn(container, *pos * GRID_UNIT_SIZE, ui, false).clicked() {
            let bks = is_ctrl_pressed(ui);
            match from {
                Some(id) => {
                    ckt_evts.push(if bks {
                        CircuitUpdateOps::Disconnect(*id, (disp_params.id, i))
                    } else {
                        CircuitUpdateOps::Connect(*id, (disp_params.id, i))
                    });
                }
                None => {}
            }
        }
    }

    if add_pin_btn(
        container,
        disp_params.output_loc_rel * GRID_UNIT_SIZE,
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
        if newx + disp_params.size.x >= WINDOW_WIDTH - 2.0 {
            // for safety
            newx = WINDOW_WIDTH - disp_params.size.x - 2.0;
        }
        if newx < 2.0 {
            newx = 2.0;
        }

        let mut newy = (k.y / GRID_UNIT_SIZE) - disp_params.size.y / 2.0;
        if newy + disp_params.size.y >= WINDOW_HEIGHT - 2.0 {
            // for safety
            newy = WINDOW_HEIGHT - disp_params.size.y - 2.0;
        }
        if newy < 6.0 {
            // to avoid menu buttons etc
            newy = 6.0;
        }

        disp_params.logical_loc.x = newx;
        disp_params.logical_loc.y = newy;
        ui_evts.push(UiUpdateOps::Dragged);
    }

    if r.clicked() {
        if is_ctrl_pressed(ui) {
            ckt_evts.push(CircuitUpdateOps::Remove(gate.id));
            ui_evts.push(UiUpdateOps::RemoveComponent(gate.id));
        } else {
            ui_evts.push(UiUpdateOps::Select(disp_params.id));
            if gate.comp_type == CompType::Input {
                ckt_evts.push(CircuitUpdateOps::SetState(disp_params.id, !gate.state));
            }
        }
    }
    return (ckt_evts, ui_evts);
}

fn add_pin_btn(container: Rect, pos: Vec2, ui: &mut Ui, selected: bool) -> Response {
    let brect = Rect::from_center_size(container.min + pos, vec2(PIN_BTN_SIZE, PIN_BTN_SIZE));
    let mut btn = Button::new("");
    if selected {
        btn = btn.fill(Color32::YELLOW)
    }
    ui.put(brect, btn)
}

fn is_ctrl_pressed(ui: &Ui) -> bool {
    ui.input(|rd| rd.modifiers.mac_cmd || rd.modifiers.ctrl)
}
