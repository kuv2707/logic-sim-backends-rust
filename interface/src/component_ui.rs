use bsim_engine::{
    components::Gate,
    types::{CompType, ID},
};
use egui::{
    epaint::CubicBezierShape, pos2, vec2, Button, Color32, FontId, Painter, Pos2, Rect, Response,
    Rounding, Sense, Stroke, TextEdit, Ui, Vec2,
};

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
    let is_clk = disp_params.name == "CLK";
    let al = ui.allocate_rect(container, Sense::click_and_drag());
    if ui.is_rect_visible(container) {
        draw_component_shape(ui.painter(), disp_params, container, gate.state);
    }

    for (i, pos) in disp_params.input_locs_rel.iter().enumerate() {
        if i == 0 && !disp_params.is_clocked {
            continue;
        }
        if add_pin_btn(
            container,
            // doesn't align with wires in AND etc
            *pos * GRID_UNIT_SIZE,
            ui,
            false,
            &gate.input_pin_exprs[i],
        )
        .clicked()
        {
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
        &gate.state_expr,
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

    if !is_clk {
        let ted = TextEdit::singleline(&mut disp_params.label).hint_text("label");
        let min =
            container.left_top() + (30.0, disp_params.size.y * GRID_UNIT_SIZE / 2.0 - 8.0).into();
        if ui
            .put(Rect::from_min_max(min, min + (4.0, 8.0).into()), ted)
            .lost_focus()
        {
            ckt_evts.push(CircuitUpdateOps::SetComponentLabel(
                disp_params.id,
                gate.label.clone(),
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

pub fn add_pin_btn(
    container: Rect,
    pos: Vec2,
    ui: &mut Ui,
    selected: bool,
    input_expr: &str,
) -> Response {
    let brect = Rect::from_center_size(container.min + pos, vec2(PIN_BTN_SIZE, PIN_BTN_SIZE));
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
    disp_params: &DisplayData,
    container: Rect,
    state: bool,
) {
    let color = if state {
        egui::Color32::from_rgb(144, 238, 144)
    } else {
        egui::Color32::from_rgb(255, 102, 102)
    };
    let stroke = Stroke::new(2.0, color);
    let name = disp_params.name.as_str();
    match name {
        "Input" => {
            let mut pts: Vec<Pos2> = vec![
                (0., 2.0).into(),
                (5.0, 2.0).into(),
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
            painter.rect_filled(
                container,
                8.0,
                if state {
                    egui::Color32::from_rgb(144, 238, 144)
                } else {
                    egui::Color32::from_rgb(255, 102, 102)
                },
            );
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
