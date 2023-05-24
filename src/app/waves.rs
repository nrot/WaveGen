use bitvec::prelude::*;
use egui::{
    plot::{Line, PlotPoint, PlotPoints, Polygon},
    Pos2, Ui,
};
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Clock {
    period: usize,
    duty: usize,
    phase: usize,
}

#[derive(Serialize, Deserialize)]
enum WaveType {
    Clock(Clock),
    Wire,
    Reg(usize),
}

#[derive(Serialize, Deserialize)]
enum WaveDisplay {
    Binary,
    Hex,
    SignedDecimal,
    UnsignedDecimal,
    Analog(Box<WaveDisplay>), // ?
}

#[derive(Serialize, Deserialize)]
struct StateEdit {
    index: usize,
    value: BitVec,
    pos: Pos2,
}

#[derive(Serialize, Deserialize)]
enum WaveState {
    Show,
    Edit(StateEdit),
}

#[derive(Serialize, Deserialize)]
pub struct Wave {
    state: WaveState,
    tp: WaveType,
    display: WaveDisplay,
    name: String,
    data: Vec<BitVec>,
    max_value: f64,
    min_value: f64,
}

impl Wave {
    pub fn new<T: Into<String>>(name: T, size: usize) -> Self {
        let mut v = Vec::with_capacity(size);
        v.resize(size, bitvec![0; 1]);
        debug!("New data size: {}", v.len());
        Self {
            state: WaveState::Show,
            tp: WaveType::Wire,
            display: WaveDisplay::Binary,
            name: name.into(),
            data: v,
            max_value: 0.0,
            min_value: 0.0,
        }
    }

    pub fn display(&mut self, ui: &mut Ui, link_group_id: egui::Id, mouse_pos: Option<Pos2>) {
        ui.horizontal(|ui| {
            let name = self.name.clone();
            ui.text_edit_singleline(&mut self.name)
                .context_menu(|ui| self.name_menu(ui));
            egui::plot::Plot::new(name)
                .link_axis(link_group_id, true, false)
                .link_cursor(link_group_id, true, true)
                .show(ui, |plot_ui| {
                    let plot: PlotPoints = self
                        .data
                        .iter()
                        .enumerate()
                        .flat_map(|(i, v)| {
                            [
                                [i as f64, v[0] as u8 as f64],
                                [(i + 1) as f64, v[0] as u8 as f64],
                            ]
                        })
                        .collect();
                    // let sin: PlotPoints = (0..1000)
                    //     .map(|i| {
                    //         let x = i as f64 * 0.01;
                    //         [x, x.sin()]
                    //     })
                    //     .collect();
                    let line = Line::new(plot);
                    plot_ui.line(line);
                    if let Some(p) = plot_ui.pointer_coordinate() {
                        let polygon = Polygon::new(PlotPoints::Owned(vec![
                            //TODO: переделать в выделение максимума/минимума
                            PlotPoint::new(p.x.floor(), 100_000_000.0f64),
                            PlotPoint::new(p.x.ceil(), 100_000_000.0f64),
                            PlotPoint::new(p.x.ceil(), -100_000_000.0f64),
                            PlotPoint::new(p.x.floor(), -100_000_000.0f64),
                        ]));
                        plot_ui.polygon(polygon.name(""));
                        if plot_ui.plot_secondary_clicked() {
                            debug!(
                                "Clicked by plot: {}; Data size: {}",
                                p.x.floor() as usize,
                                self.data.len()
                            );
                            if let Some(v) = self.data.get(p.x.floor() as usize) {
                                self.state = WaveState::Edit(StateEdit {
                                    index: p.x.floor() as usize,
                                    value: v.clone(),
                                    pos: mouse_pos.unwrap_or(Pos2 { x: 0.0, y: 0.0 }),
                                });
                            }
                        }
                    }
                });
            if let WaveState::Edit(edit) = &mut self.state {
                let mut v = edit.value[0];
                let mut open = true;
                egui::Window::new(format!("Edit: {}", edit.index))
                    .title_bar(true)
                    .open(&mut open)
                    .default_pos(edit.pos)
                    .show(ui.ctx(), |ui| {
                        if ui.toggle_value(&mut v, "").changed() {
                            self.data[edit.index].insert(0, v);
                        };
                    });
                if !open{
                    self.state = WaveState::Show;
                }
            }
        });
    }

    fn name_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Change Type", |ui| {
            ui.button("Wire");
        });
    }
}
