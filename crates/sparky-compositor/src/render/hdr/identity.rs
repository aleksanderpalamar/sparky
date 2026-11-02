use std::cell::Cell;

use smithay::backend::renderer::element::Id;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::with_states;

use crate::color_management::conversion::Conversion;

#[derive(Default)]
struct Tag(Cell<Option<(Conversion, Id)>>);

impl Tag {
    fn id(&self, conversion: Conversion) -> Id {
        let id = match self.0.take() {
            Some((known, id)) if known == conversion => id,
            _ => Id::new(),
        };
        self.0.set(Some((conversion, id.clone())));
        id
    }
}

pub fn element_id(surface: &WlSurface, conversion: Conversion) -> Id {
    with_states(surface, |states| {
        states.data_map.insert_if_missing(Tag::default);
        states
            .data_map
            .get::<Tag>()
            .map_or_else(Id::new, |tag| tag.id(conversion))
    })
}

#[cfg(test)]
mod tests {
    use super::Tag;
    use crate::color_management::conversion::{Conversion, conversion};
    use crate::color_management::description::{
        ImageDescription, NamedPrimaries, TransferFunction,
    };
    use crate::color_management::presets::windows_scrgb;

    fn hdr10() -> Option<Conversion> {
        conversion(&ImageDescription::standard(
            TransferFunction::St2084Pq,
            NamedPrimaries::Bt2020,
        ))
    }

    #[test]
    fn mesma_conversao_mantem_o_elemento() -> Result<(), &'static str> {
        let tag = Tag::default();
        let pq = hdr10().ok_or("PQ precisa de conversão")?;
        assert_eq!(tag.id(pq), tag.id(pq));
        Ok(())
    }

    #[test]
    fn trocar_a_conversao_cria_outro_elemento_e_redesenha_a_janela() -> Result<(), &'static str> {
        let tag = Tag::default();
        let pq = hdr10().ok_or("PQ precisa de conversão")?;
        let scrgb = conversion(&windows_scrgb()).ok_or("scRGB precisa de conversão")?;
        let primeira = tag.id(pq);
        let linear = tag.id(scrgb);
        assert_ne!(linear, primeira);
        let de_volta = tag.id(pq);
        assert_ne!(de_volta, linear);
        assert_eq!(tag.id(pq), de_volta);
        Ok(())
    }
}
