use std::cmp::max;

use bsim_engine::{
    circuit::BCircuit,
    components::Gate,
    types::{CompType, ID},
};
use egui::{
    epaint::CubicBezierShape, vec2, Button, Color32, FontId, Id, Label, Painter, Pos2, Rect,
    Response, Rounding, Sense, Stroke, TextEdit, Ui, Vec2,
};

use crate::{
    app::SimulatorUI,
    consts::{DEFAULT_SCALE, GREEN_COL, GRID_UNIT_SIZE, RED_COL},
    display_elems::{CompDisplayData, DisplayState, Screen},
    true_false_color,
    update_ops::{self, CircuitUpdateOps, StateUpdateOps, UiUpdateOps},
    utils::{CompIO, EmitterReceiverPair},
};

pub const PIN_BTN_SIZE: f32 = 10.0;

pub fn paint_components(
    display_state: &mut DisplayState,
    ckt: &BCircuit,
    ui: &mut Ui,
) -> Vec<StateUpdateOps> {
    let mut ret = Vec::new();
    for v in &mut display_state.comp_display_data {
        paint_component(
            v.0,
            v.1,
            &ckt,
            ui,
            &mut display_state.connect_candidate,
            &mut ret,
            &display_state.screen,
        );
    }
    ret
}

pub fn paint_component(
    id: &egui::Id,
    disp_params: &mut CompDisplayData,
    ckt: &BCircuit,
    ui: &mut Ui,
    conn_cand: &mut Option<(egui::Id, CompIO)>,
    update_ops: &mut Vec<StateUpdateOps>,
    scr: &Screen,
) {
    let scroll_offset = ui.min_rect().min;
    let container = egui::Rect::from_min_size(
        scroll_offset + disp_params.logical_loc.to_vec2() * GRID_UNIT_SIZE,
        disp_params.size * GRID_UNIT_SIZE * disp_params.scale,
    );
    let is_clk = disp_params.name == "CLK";
    let al = ui.allocate_rect(container, Sense::click_and_drag());

    let gate = match &disp_params.state_indicator_ref {
        Some(id) => match ckt.components().get(id) {
            Some(g) => Some(g),
            None => None,
        },
        None => None,
    };

    if ui.is_rect_visible(container) {
        draw_component_shape(
            ui.painter(),
            disp_params,
            container,
            match gate {
                Some(g) => {
                    if g.borrow().active {
                        Some(g.borrow().state)
                    } else {
                        None // grey it out if inactive
                    }
                }
                None => None, // grey it out if no state indicator ref
            },
            disp_params.scale,
        );
    }

    for (i, port) in disp_params.inputs_rel.iter().enumerate() {
        if i == 0 && !disp_params.is_clocked {
            continue;
        }
        if add_pin_btn(
            container,
            // doesn't align with wires in AND etc
            port,
            ui,
            false,
            &ckt.components()
                .get(&port.id)
                .unwrap()
                .borrow()
                .input_pin_exprs[port.pin],
            disp_params.scale,
        )
        .clicked()
        {
            let bks = is_ctrl_pressed(ui);
            match conn_cand {
                Some(id) => {
                    update_ops.push(StateUpdateOps::CktOp(if bks {
                        CircuitUpdateOps::Disconnect(EmitterReceiverPair {
                            emitter: id.clone(),
                            receiver: (disp_params.id, port.clone()),
                        })
                    } else {
                        CircuitUpdateOps::Connect(EmitterReceiverPair {
                            emitter: id.clone(),
                            receiver: (disp_params.id, port.clone()),
                        })
                    }));
                }
                None => {}
            }
        }
    }

    for port in &disp_params.outputs_rel {
        if add_pin_btn(
            container,
            port,
            ui,
            match conn_cand {
                Some((_, pininfo)) => port.id == pininfo.id,
                None => false,
            },
            &ckt.components().get(&port.id).unwrap().borrow().state_expr,
            disp_params.scale,
        )
        .clicked()
        {
            *conn_cand = match conn_cand {
                Some((_, pininfo)) => {
                    if port.id == pininfo.id {
                        // clicking on the same pin btn twice deselects it
                        None
                    } else {
                        Some((disp_params.id, port.clone()))
                    }
                }
                None => Some((disp_params.id, port.clone())),
            };
        }
    }

    if disp_params.scale >= DEFAULT_SCALE {
        if !is_clk {
            let ted = TextEdit::singleline(&mut disp_params.label).hint_text("label");
            let min = container.left_top()
                + (
                    30.0,
                    disp_params.size.y * disp_params.scale * GRID_UNIT_SIZE / 2.0 - 8.0,
                )
                    .into();
            if ui
                .put(Rect::from_min_max(min, min + (4.0, 8.0).into()), ted)
                .lost_focus()
            {
                update_ops.push(StateUpdateOps::CktOp(CircuitUpdateOps::SetComponentLabel(
                    disp_params.outputs_rel[0].id, //todo: what would it mean for a module
                    // gate.label.clone(),
                    "".into(),
                    disp_params.label.clone(),
                )));
            }
        } else {
            ui.put(
                Rect::from_center_size(container.center() + vec2(0., 0.0), vec2(50.0, 10.0)),
                Button::new(&disp_params.label),
            );
        }
    }

    let r = ui.interact(container, al.id, Sense::click_and_drag());
    if r.dragged() {
        let k = ui.input(|i| i.pointer.interact_pos().unwrap()) - scroll_offset;
        let mut newx =
            (k.x / GRID_UNIT_SIZE).floor() - disp_params.size.x * disp_params.scale / 2.0;
        let mut newy =
            (k.y / GRID_UNIT_SIZE).floor() - disp_params.size.y * disp_params.scale / 2.0;

        newx = newx.max(2.0);
        newx = newx.min(scr.logical_width() as f32 - disp_params.size.x - 2.0);
        newy = newy.max(6.0);
        newy = newy.min(scr.logical_height() as f32 - disp_params.size.y - 2.0);
        disp_params.logical_loc.x = newx;
        disp_params.logical_loc.y = newy;
        update_ops.push(StateUpdateOps::UiOp(UiUpdateOps::Dragged));
    }
    if r.clicked() {
        if is_option_alt_pressed(ui) {
            if (disp_params.scale > 1.5) {
                disp_params.scale = 0.5;
            } else {
                disp_params.scale += 0.1;
            }
        }
        update_ops.push(StateUpdateOps::UiOp(UiUpdateOps::Dragged));
    }

    if r.clicked() && !is_clk {
        if is_ctrl_pressed(ui) {
            update_ops.push(StateUpdateOps::UiOp(UiUpdateOps::RemoveComponent(
                disp_params.id,
            )));
        } else {
            update_ops.push(StateUpdateOps::UiOp(UiUpdateOps::Select(disp_params.id)));

            match gate {
                Some(gate) => {
                    if gate.borrow().comp_type == CompType::Input {
                        update_ops.push(StateUpdateOps::CktOp(CircuitUpdateOps::SetState(
                            disp_params.outputs_rel[0].id,
                            !gate.borrow().state,
                        )));
                    }
                }
                None => {}
            }
        }
    }
}

pub fn add_pin_btn(
    container: Rect,
    pin: &CompIO,
    ui: &mut Ui,
    selected: bool,
    expr: &str,
    scale: f32,
) -> Response {
    let brect = Rect::from_center_size(
        container.min + pin.loc_rel * GRID_UNIT_SIZE * scale,
        vec2(PIN_BTN_SIZE, PIN_BTN_SIZE),
    );
    ui.painter().rect_filled(
        brect,
        Rounding::same(12.0),
        if selected {
            Color32::YELLOW
        } else {
            Color32::GRAY
        },
    );
    let mut al = ui.allocate_rect(brect, Sense::click_and_drag());
    if expr.len() > 0 {
        ui.put(
            Rect::from_min_size(brect.min + vec2(PIN_BTN_SIZE, 0.), vec2(10.0, 10.0)),
            Label::new(&pin.label),
        );
        al = al.on_hover_text(expr);
    }
    let res = ui.interact(brect, al.id, Sense::click_and_drag());
    res
}

fn is_ctrl_pressed(ui: &Ui) -> bool {
    ui.input(|rd| rd.modifiers.mac_cmd || rd.modifiers.ctrl)
}

fn is_option_alt_pressed(ui: &Ui) -> bool {
    ui.input(|rd| rd.modifiers.alt)
}

fn draw_component_shape(
    painter: &Painter,
    disp_params: &CompDisplayData,
    container: Rect,
    state: Option<bool>,
    scale: f32,
) {
    let color = match state {
        Some(state) => {
            true_false_color!(state)
        }
        None => egui::Color32::from_rgb(19, 19, 19),
    };
    let state = match state {
        Some(s) => s,
        None => false, // when drawing a module
    };
    let stroke = Stroke::new(2.0, color);
    let name = disp_params.name.as_str();
    match name {
        "Input" => {
            let mut pts: Vec<Pos2> = vec![
                (0., 2.0).into(),
                (5.0, 2.0).into(),
                (7., 4.0).into(),
                (8., 4.0).into(),
                (7., 4.0).into(),
                (5., 6.0).into(),
                (0., 6.0).into(),
            ];
            pts.push(pts[0]);
            draw_path(painter, pts, stroke, scale, container, state);
        }
        "NOT" | "BFR" => {
            let pts: Vec<Pos2> = vec![
                (0.0, 4.0).into(),
                (0., 0.0).into(),
                (6., 4.0).into(),
                (8., 4.0).into(),
                (6., 4.0).into(),
                (0., 8.0).into(),
                (0., 4.0).into(),
            ];
            draw_path(painter, pts, stroke, scale, container, state);
            if name == "NOT" {
                painter.circle(
                    container.left_top()
                        + vec2(
                            6.0 * GRID_UNIT_SIZE * scale + 4.0,
                            4.0 * GRID_UNIT_SIZE * scale,
                        ),
                    2.5,
                    color,
                    stroke,
                );
            }
        }
        "AND" | "NAND" => {
            let tl = container.left_top();
            painter.add(CubicBezierShape::from_points_stroke(
                [
                    tl + vec2(0., 1.0) * GRID_UNIT_SIZE,
                    tl + vec2(8.0, 1.0) * GRID_UNIT_SIZE,
                    tl + vec2(8.0, 7.0) * GRID_UNIT_SIZE,
                    tl + vec2(0., 7.0) * GRID_UNIT_SIZE,
                ],
                true,
                Color32::TRANSPARENT,
                stroke,
            ));
            draw_path(
                painter,
                vec![(6., 4.0).into(), (8.0, 4.0).into()],
                stroke,
                scale,
                container,
                state,
            );

            if name == "NAND" {
                painter.circle(
                    container.left_top()
                        + vec2(
                            6.0 * GRID_UNIT_SIZE * scale + 4.0,
                            4.0 * GRID_UNIT_SIZE * scale,
                        ),
                    2.5,
                    color,
                    stroke,
                );
            }
        }
        _ => {
            painter.rect_filled(container, 8.0, true_false_color!(state));
        }
    };
    // painter.rect_stroke(container, 8.0, Stroke::new(2.0, Color32::BLACK));
}

fn draw_path(
    painter: &Painter,
    pts: Vec<Pos2>,
    stroke: Stroke,
    scale: f32,
    container: Rect,
    state: bool,
) {
    // todo: optionally fill with a color.
    painter.line(
        pts.iter()
            .map(|p| container.left_top() + p.to_vec2() * GRID_UNIT_SIZE * scale)
            .collect(),
        stroke,
    );
}
