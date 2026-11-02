use std::sync::Arc;

use smithay::output::{Output, WeakOutput};
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_management_output_v1::WpColorManagementOutputV1;
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_management_surface_feedback_v1::WpColorManagementSurfaceFeedbackV1;
use smithay::reexports::wayland_server::Weak;

use super::presets::{self, OutputColor};
use super::records::{Record, Records};
use super::surfaces::Surfaces;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Same,
    Changed,
}

pub struct ColorManagementState {
    pub records: Records,
    pub output_objects: Vec<Weak<WpColorManagementOutputV1>>,
    pub feedbacks: Vec<Weak<WpColorManagementSurfaceFeedbackV1>>,
    pub surfaces: Surfaces,
    sdr: Arc<Record>,
    outputs: Vec<OutputEntry>,
}

struct OutputEntry {
    output: WeakOutput,
    color: OutputColor,
    record: Arc<Record>,
}

impl Default for ColorManagementState {
    fn default() -> Self {
        let mut records = Records::default();
        let sdr = records.record(presets::output(OutputColor::Sdr));
        Self {
            records,
            output_objects: Vec::new(),
            feedbacks: Vec::new(),
            surfaces: Surfaces::default(),
            sdr,
            outputs: Vec::new(),
        }
    }
}

impl ColorManagementState {
    pub fn sdr(&self) -> Arc<Record> {
        self.sdr.clone()
    }

    pub fn record_for(&self, output: &Output) -> Arc<Record> {
        self.outputs
            .iter()
            .find(|entry| entry.output == *output)
            .map_or_else(|| self.sdr(), |entry| entry.record.clone())
    }

    pub fn set(&mut self, output: &Output, color: OutputColor) -> Change {
        self.outputs
            .retain(|entry| entry.output.upgrade().is_some());
        let index = self
            .outputs
            .iter()
            .position(|entry| entry.output == *output);
        let previous = index
            .and_then(|index| self.outputs.get(index))
            .map_or(OutputColor::Sdr, |entry| entry.color);
        if previous == color {
            return Change::Same;
        }
        let entry = OutputEntry {
            output: output.downgrade(),
            color,
            record: self.records.record(presets::output(color)),
        };
        match index.and_then(|index| self.outputs.get_mut(index)) {
            Some(slot) => *slot = entry,
            None => self.outputs.push(entry),
        }
        Change::Changed
    }
}

#[cfg(test)]
mod tests {
    use smithay::output::{Output, PhysicalProperties, Subpixel};

    use super::{Change, ColorManagementState};
    use crate::color_management::description::{Range, TransferFunction};
    use crate::color_management::presets::OutputColor;

    fn tela(nome: &str) -> Output {
        Output::new(
            nome.into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "teste".into(),
                model: "teste".into(),
            },
        )
    }

    const HDR: OutputColor = OutputColor::Hdr(Range { min: 50, max: 600 });

    #[test]
    fn tela_nova_comeca_em_sdr_com_identidade_estavel() {
        let state = ColorManagementState::default();
        let monitor = tela("HDMI-A-1");
        let sdr = state.record_for(&monitor);
        assert_eq!(sdr.description.transfer, TransferFunction::Srgb);
        assert_ne!(sdr.identity, 0);
        assert_eq!(state.record_for(&monitor).identity, sdr.identity);
        assert_eq!(state.sdr().identity, sdr.identity);
    }

    #[test]
    fn so_avisa_quando_a_cor_da_tela_muda() {
        let mut state = ColorManagementState::default();
        let monitor = tela("HDMI-A-1");
        let sdr = state.record_for(&monitor).identity;
        assert_eq!(state.set(&monitor, OutputColor::Sdr), Change::Same);
        assert_eq!(state.set(&monitor, HDR), Change::Changed);
        assert_eq!(state.set(&monitor, HDR), Change::Same);
        let hdr = state.record_for(&monitor);
        assert_eq!(hdr.description.transfer, TransferFunction::St2084Pq);
        assert_ne!(hdr.identity, sdr);
        assert_eq!(state.set(&monitor, OutputColor::Sdr), Change::Changed);
        assert_eq!(state.record_for(&monitor).identity, sdr);
    }

    #[test]
    fn cada_tela_tem_a_sua_cor() {
        let mut state = ColorManagementState::default();
        let tv = tela("HDMI-A-1");
        let monitor = tela("DP-1");
        state.set(&tv, HDR);
        assert_eq!(
            state.record_for(&tv).description.transfer,
            TransferFunction::St2084Pq
        );
        assert_eq!(
            state.record_for(&monitor).description.transfer,
            TransferFunction::Srgb
        );
    }

    #[test]
    fn telas_que_sumiram_saem_da_lista() {
        let mut state = ColorManagementState::default();
        state.set(&tela("HDMI-A-1"), HDR);
        state.set(&tela("DP-1"), HDR);
        assert_eq!(state.outputs.len(), 1);
    }
}
