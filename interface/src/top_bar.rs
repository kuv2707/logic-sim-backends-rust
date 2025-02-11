use std::{borrow::Cow, cmp::max, collections::HashSet};

use bsim_engine::{
    circuit::BCircuit,
    components::Gate,
    types::{ID, PIN},
};
use egui::{vec2, Button, ComboBox, Id, TextEdit, Ui, Vec2, Widget};

use crate::{
    app::SimulatorUI,
    consts::{DEFAULT_SCALE, GRID_UNIT_SIZE},
    display_elems::CompDisplayData,
    logic_units::{get_logic_unit, ModuleCreationData},
    update_ops::{self, StateUpdateOps, UiUpdateOps},
    utils::CompIO,
};

pub enum TopBarOption {
    AddComponent {
        name: String,
        scale: f32,
    },
    AddModuleFromText {
        typed_text: String,
        modulator: Modulator,
        enter_text: String,
    },
    AddModuleFromOptions {
        label_text: String,
        options: Vec<String>,
        modulator: Modulator,
    },
}

pub enum Modulator {
    Expressions,
    Decoder,
    Encoder,
    SevenSegment,
}

impl Modulator {
    pub fn pre_modulate(&self, s: &str) -> String {
        match self {
            Self::Expressions => s.to_string(),
            Self::Decoder => {
                // todo: use regex
                let parts: Vec<&str> = s.split('x').collect();
                if parts.len() == 2 {
                    if let (Ok(ins), Ok(outs)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>())
                    {
                        // assert outs = 2**ins

                        let mut result: Vec<String> = Vec::new();
                        let labs: Vec<char> =
                            (0..ins).map(|i| ((b'a' + i as u8) as char)).collect();

                        let mut temp: Vec<String> = Vec::new();
                        for i in 0..outs {
                            temp.clear();
                            for j in 0..ins as usize {
                                let bit = i >> j & 1 == 1;

                                if bit {
                                    temp.push(labs[j].to_string());
                                } else {
                                    temp.push(format!("!{}", labs[j]));
                                }
                            }
                            result.push(format!("{} = {}", i, temp.join(".")));
                        }
                        return result.join(";");
                    }
                }
                "".to_string()
            }
            Self::Encoder {} => {
                todo!()
            }
            Self::SevenSegment {} => "a=a;b=b;c=c;d=d;e=e;f=f;g=g;".into(),
        }
    }
    pub fn post_modulate(&self, data: &mut CompDisplayData) {
        match self {
            Self::SevenSegment {} => {
                data.size.x = 12.0;
                data.size.y = 20.0;
                for k in data.outputs_rel.iter_mut() {
                    k.loc_rel.x = data.size.x;
                }
                data.name = "7Segment".into();
            }
            Modulator::Expressions => {}
            Modulator::Decoder => {}
            Modulator::Encoder => {}
        }
    }
}

impl TopBarOption {
    pub fn render(&mut self, ckt: &mut BCircuit, ui: &mut Ui) -> Vec<StateUpdateOps> {
        let mut update_ops = Vec::new();
        match self {
            Self::AddComponent { name, scale } => {
                let button = egui::Button::new(name.as_str()).min_size(Vec2::new(80.0, 40.0));
                let response = button.ui(ui);
                if response.clicked() {
                    let id = match name.as_str() {
                        "Input" => ckt.add_input("", false),
                        _ => ckt.add_component(name, "").unwrap(),
                    };
                    let data = compose_comp_data(&ckt.get_component(&id).unwrap().borrow(), *scale);

                    update_ops.push(StateUpdateOps::UiOp(UiUpdateOps::AddComponent(data)));
                }
            }
            Self::AddModuleFromText {
                typed_text,
                modulator,
                enter_text,
            } => {
                ui.vertical(|ui| {
                    ui.add(TextEdit::singleline(typed_text).desired_width(80.0));
                    if ui.add(Button::new(enter_text.as_str())).clicked() {
                        match get_disp_data_from_modctx(get_logic_unit(
                            ckt,
                            &modulator.pre_modulate(typed_text),
                        )) {
                            Ok(mut data) => {
                                modulator.post_modulate(&mut data);
                                update_ops
                                    .push(StateUpdateOps::UiOp(UiUpdateOps::AddComponent(data)));
                            }
                            Err(e) => {
                                // todo: show msg that expr was bad
                            }
                        }
                        typed_text.clear();
                    }
                });
            }
            Self::AddModuleFromOptions {
                label_text,
                options,
                modulator,
            } => {
                ComboBox::from_id_salt(Id::new(label_text.to_string()))
                    .selected_text(label_text.to_string())
                    .show_ui(ui, |ui| {
                        for option in options {
                            if ui.selectable_label(false, option.to_string()).clicked() {
                                match get_disp_data_from_modctx(get_logic_unit(
                                    ckt,
                                    &modulator.pre_modulate(option),
                                )) {
                                    Ok(mut data) => {
                                        modulator.post_modulate(&mut data);
                                        update_ops.push(StateUpdateOps::UiOp(
                                            UiUpdateOps::AddComponent(data),
                                        ));
                                    }
                                    Err(e) => {
                                        // todo: show msg that expr was bad
                                    }
                                }
                            }
                        }
                    });
            }
        }
        update_ops
    }
}

pub fn compose_comp_data(gate: &Gate, scale: f32) -> CompDisplayData {
    let loc = egui::pos2(40.0 + 80.0, 100.0) / GRID_UNIT_SIZE;
    let size: Vec2 = (8.0, 8.0).into();
    let spc = size.y / (gate.num_inputs()) as f32;
    let inputs_rel = (0..gate.num_inputs())
        .map(|i| {
            CompIO {
                id: gate.id,
                pin: i,
                loc_rel: if i == 0 {
                    // clock
                    vec2(size.x / 2.0, size.y)
                } else {
                    vec2(0.0, spc * i as f32)
                },
                label: String::new(),
            }
        })
        .collect();
    let mut contents = HashSet::new();
    contents.insert(gate.id);
    CompDisplayData {
        id: egui::Id::new(gate.id),
        logical_loc: loc,
        name: gate.name.clone(),
        label: gate.label.clone(),
        outputs_rel: vec![CompIO {
            id: gate.id,
            pin: 1,
            loc_rel: vec2(size.x, size.y / 2.0),
            label: String::new(),
        }],
        inputs_rel,
        is_clocked: gate.clock_manager.is_some(),
        scale,
        size,
        state_indicator_ref: vec![gate.id],
        contents,
    }
}

fn get_disp_data_from_modctx(
    res: Result<ModuleCreationData, String>,
) -> Result<CompDisplayData, String> {
    match res {
        Ok(ctx) => {
            let mut ipins: Vec<(&String, &ID)> = ctx.inputs.iter().collect();
            ipins.sort();
            let mut opins: Vec<(&String, &ID)> = ctx.outputs.iter().collect();
            opins.sort();
            let contents = ctx.contents;
            let size: Vec2 = (8.0, 2.0 * (max(ipins.len(), opins.len())) as f32).into();

            let i_gap = size.y / (ipins.len() + 1) as f32;
            let o_gap = size.y / (opins.len() + 1) as f32;
            let data = CompDisplayData {
                id: egui::Id::new(contents.iter().next().unwrap()),
                logical_loc: (7.0, 7.0).into(),
                name: "module".into(),
                label: "".into(),
                outputs_rel: opins
                    .iter()
                    .enumerate()
                    .map(|(i, id)| CompIO {
                        id: *id.1,
                        pin: 1,
                        loc_rel: vec2(size.x, o_gap * (i + 1) as f32),
                        label: id.0.to_string(),
                    })
                    .collect(),
                inputs_rel: ipins
                    .iter()
                    .enumerate()
                    .map(|(i, id)| CompIO {
                        id: *id.1,
                        pin: 1,
                        loc_rel: vec2(0.0, i_gap * (i + 1) as f32),
                        label: id.0.to_string(),
                    })
                    .collect(),
                is_clocked: true, // todo
                scale: DEFAULT_SCALE,
                size,
                state_indicator_ref: vec![],
                contents,
            };
            Ok(data)
        }
        Err(e) => Err(e),
    }
}
