use bsim_engine::{
    circuit::BCircuit,
    components::Gate,
    types::{CompType, ID},
};
use egui::{
    epaint::CubicBezierShape, vec2, Button, Color32, FontId, Label, Painter, Pos2, Rect, Response,
    Rounding, Sense, Stroke, TextEdit, Ui, Vec2,
};

use crate::{
    consts::{GREEN_COL, GRID_UNIT_SIZE, RED_COL, WINDOW_HEIGHT, WINDOW_WIDTH},
    display_elems::CompDisplayData,
    update_ops::{CircuitUpdateOps, UiUpdateOps},
    utils::{CompIO, EmitterReceiverPair},
};

pub const PIN_BTN_SIZE: f32 = 10.0;

pub fn paint_component(
    disp_params: &mut CompDisplayData,
    ui: &mut Ui,
    ckt: &BCircuit,
    from: &mut Option<(egui::Id, CompIO)>,
    ckt_evts: &mut Vec<CircuitUpdateOps>,
    ui_evts: &mut Vec<UiUpdateOps>,
) {
    let container = egui::Rect::from_min_size(
        disp_params.logical_loc * GRID_UNIT_SIZE,
        disp_params.size * GRID_UNIT_SIZE,
    );
    let is_clk = disp_params.name == "CLK";
    let al = ui.allocate_rect(container, Sense::click_and_drag());

    // todo: this should only be Some() when the component isn't a module
    let gate = match ckt.components().get(&disp_params.outputs_rel[0].id) {
        Some(g) => g,
        None => return,
    }
    .borrow();

    if ui.is_rect_visible(container) {
        draw_component_shape(
            ui.painter(),
            disp_params,
            container,
            if disp_params.is_module {
                None
            } else {
                Some(gate.state)
            },
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
        )
        .clicked()
        {
            let bks = is_ctrl_pressed(ui);
            match from {
                Some(id) => {
                    ckt_evts.push(if bks {
                        CircuitUpdateOps::Disconnect(EmitterReceiverPair {
                            emitter: id.clone(),
                            receiver: (disp_params.id, port.clone()),
                        })
                    } else {
                        CircuitUpdateOps::Connect(EmitterReceiverPair {
                            emitter: id.clone(),
                            receiver: (disp_params.id, port.clone()),
                        })
                    });
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
            match from {
                Some((_, pininfo)) => port.id == pininfo.id,
                None => false,
            },
            &ckt.components().get(&port.id).unwrap().borrow().state_expr,
        )
        .clicked()
        {
            *from = match from {
                Some((id, pininfo)) => {
                    if port.id == pininfo.id {
                        // clicking on the same pin btn twice deselects it
                        None
                    } else {
                        Some((disp_params.id, port.clone()))
                    }
                }
                None => Some((disp_params.id, port.clone())),
            }
        }
    }

    if !is_clk {
        let ted = TextEdit::singleline(&mut disp_params.label).hint_text("label");
        let min =
            container.left_top() + (30.0, disp_params.size.y * GRID_UNIT_SIZE / 2.0 - 8.0).into();
        if ui
            .put(Rect::from_min_max(min, min + (4.0, 8.0).into()), ted)
            .lost_focus()
        {
            ckt_evts.push(CircuitUpdateOps::SetComponentLabel(
                disp_params.outputs_rel[0].id, //todo: what would it mean for a module
                // gate.label.clone(),
                "".into(),
                disp_params.label.clone(),
            ));
        }
    } else {
        ui.put(
            Rect::from_center_size(container.center() + vec2(0., 0.0), vec2(50.0, 10.0)),
            Button::new(&disp_params.label),
        );
    }

    let r = ui.interact(container, al.id, Sense::click_and_drag());
    if r.dragged() {
        let k = ui.input(|i| i.pointer.interact_pos().unwrap());
        let mut newx = (k.x / GRID_UNIT_SIZE).floor() - disp_params.size.x / 2.0;
        if newx + disp_params.size.x >= WINDOW_WIDTH - 2.0 {
            // for safety
            newx = WINDOW_WIDTH - disp_params.size.x - 2.0;
        }
        if newx < 2.0 {
            newx = 2.0;
        }

        let mut newy = (k.y / GRID_UNIT_SIZE).floor() - disp_params.size.y / 2.0;
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

    if r.clicked() && !is_clk {
        if is_ctrl_pressed(ui) {
            ui_evts.push(UiUpdateOps::RemoveComponent(disp_params.id));
        } else {
            ui_evts.push(UiUpdateOps::Select(disp_params.id));

            if gate.comp_type == CompType::Input {
                ckt_evts.push(CircuitUpdateOps::SetState(
                    disp_params.outputs_rel[0].id,
                    !gate.state,
                ));
            }
        }
    }
}

pub fn add_pin_btn(
    container: Rect,
    pin: &CompIO,
    ui: &mut Ui,
    selected: bool,
    input_expr: &str,
) -> Response {
    let brect = Rect::from_center_size(
        container.min + pin.loc_rel * GRID_UNIT_SIZE,
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
    ui.put(
        Rect::from_min_size(brect.min + vec2(PIN_BTN_SIZE, 0.), vec2(10.0, 10.0)),
        Label::new(&pin.label),
    );
    let mut al = ui.allocate_rect(brect, Sense::click_and_drag());
    if input_expr.len() > 0 {
        al = al.on_hover_text(input_expr);
    }
    let res = ui.interact(brect, al.id, Sense::click_and_drag());
    res
}

fn is_ctrl_pressed(ui: &Ui) -> bool {
    ui.input(|rd| rd.modifiers.mac_cmd || rd.modifiers.ctrl)
}

fn draw_component_shape(
    painter: &Painter,
    disp_params: &CompDisplayData,
    container: Rect,
    state: Option<bool>,
) {
    let color = match state {
        Some(state) => {
            if state {
                GREEN_COL
            } else {
                RED_COL
            }
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
            draw_path(painter, pts, stroke, container, state);
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
            draw_path(painter, pts, stroke, container, state);
            if name == "NOT" {
                painter.circle(
                    container.left_top() + vec2(6.0 * GRID_UNIT_SIZE + 4.0, 4.0 * GRID_UNIT_SIZE),
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
                container,
                state,
            );

            if name == "NAND" {
                painter.circle(
                    container.left_top() + vec2(6.0 * GRID_UNIT_SIZE + 4.0, 4.0 * GRID_UNIT_SIZE),
                    2.5,
                    color,
                    stroke,
                );
            }
        }
        _ => {
            painter.rect_filled(container, 8.0, if state { GREEN_COL } else { RED_COL });
        }
    };
    // painter.rect_stroke(container, 8.0, Stroke::new(2.0, Color32::BLACK));
}

fn draw_path(painter: &Painter, pts: Vec<Pos2>, stroke: Stroke, container: Rect, state: bool) {
    // todo: optionally fill with a color.
    painter.line(
        pts.iter()
            .map(|p| container.left_top() + p.to_vec2() * GRID_UNIT_SIZE)
            .collect(),
        stroke,
    );
}
