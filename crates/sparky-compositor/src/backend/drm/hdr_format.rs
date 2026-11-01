use smithay::backend::allocator::Fourcc;

use crate::backend::drm::gpu::Surface;
use crate::backend::drm::hdr::Video;

pub const HDR_FORMATS: [Fourcc; 2] = [Fourcc::Xrgb2101010, Fourcc::Xbgr2101010];
pub const SDR_FORMATS: [Fourcc; 2] = [Fourcc::Argb8888, Fourcc::Abgr8888];

pub fn switch_format(
    video: &Video,
    surface: &Surface,
    formats: &[Fourcc],
) -> Result<Fourcc, String> {
    let chosen = surface.drm_output.with_compositor(|compositor| {
        first_accepted(formats, |code| {
            let modifiers = video
                .formats
                .iter()
                .filter(|format| format.code == code)
                .map(|format| format.modifier);
            compositor
                .set_format(video.allocator.clone(), code, modifiers)
                .is_ok()
        })
    })?;
    tracing::info!(output = %surface.output.name(), "formato de saída {chosen:?}");
    Ok(chosen)
}

fn first_accepted(
    formats: &[Fourcc],
    mut accept: impl FnMut(Fourcc) -> bool,
) -> Result<Fourcc, String> {
    formats
        .iter()
        .copied()
        .find(|code| accept(*code))
        .ok_or_else(|| format!("nenhum formato aceito entre {formats:?}"))
}

#[cfg(test)]
mod tests {
    use smithay::backend::allocator::Fourcc;

    use super::{HDR_FORMATS, first_accepted};

    #[test]
    fn nenhum_formato_de_10_bits_aceito_vira_erro() {
        assert!(first_accepted(&HDR_FORMATS, |_| false).is_err());
    }

    #[test]
    fn usa_o_primeiro_formato_aceito_na_ordem() {
        let segundo = first_accepted(&HDR_FORMATS, |code| code == Fourcc::Xbgr2101010);
        assert_eq!(segundo, Ok(Fourcc::Xbgr2101010));
        assert_eq!(
            first_accepted(&HDR_FORMATS, |_| true),
            Ok(Fourcc::Xrgb2101010)
        );
    }
}
