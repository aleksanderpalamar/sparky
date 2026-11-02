use std::sync::Mutex;

use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_image_description_creator_params_v1::{
    Error, Request, WpImageDescriptionCreatorParamsV1,
};
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_image_description_v1::WpImageDescriptionV1;
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, New, Resource};

use super::image::{self, Information};
use super::names;
use crate::color_management::description::{Chromaticities, Luminances, Range, Xy};
use crate::color_management::params::{Params, ParamsError};
use crate::state::Sparky;

impl Dispatch<WpImageDescriptionCreatorParamsV1, Mutex<Params>> for Sparky {
    fn request(
        state: &mut Self,
        _: &Client,
        creator: &WpImageDescriptionCreatorParamsV1,
        request: Request,
        params: &Mutex<Params>,
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        let Ok(mut params) = params.lock() else {
            return;
        };
        let result = match request {
            Request::Create { image_description } => {
                let built = std::mem::take(&mut *params).build();
                return create(state, creator, init, image_description, built);
            }
            Request::SetTfNamed { tf } => names::supported_transfer(tf, creator.version())
                .ok_or(Error::InvalidTf)
                .and_then(|transfer| params.set_transfer(transfer).map_err(code)),
            Request::SetPrimariesNamed { primaries } => names::supported_primaries(primaries)
                .ok_or(Error::InvalidPrimariesNamed)
                .and_then(|primaries| params.set_primaries(primaries).map_err(code)),
            Request::SetLuminances {
                min_lum,
                max_lum,
                reference_lum,
            } => params
                .set_luminances(Luminances {
                    min: min_lum,
                    max: max_lum,
                    reference: reference_lum,
                })
                .map_err(code),
            Request::SetMasteringDisplayPrimaries {
                r_x,
                r_y,
                g_x,
                g_y,
                b_x,
                b_y,
                w_x,
                w_y,
            } => {
                let gamut = Chromaticities {
                    red: Xy { x: r_x, y: r_y },
                    green: Xy { x: g_x, y: g_y },
                    blue: Xy { x: b_x, y: b_y },
                    white: Xy { x: w_x, y: w_y },
                };
                params.set_mastering_primaries(gamut).map_err(code)
            }
            Request::SetMasteringLuminance { min_lum, max_lum } => params
                .set_mastering_luminance(Range {
                    min: min_lum,
                    max: max_lum,
                })
                .map_err(code),
            Request::SetMaxCll { max_cll } => params.set_max_cll(max_cll).map_err(code),
            Request::SetMaxFall { max_fall } => params.set_max_fall(max_fall).map_err(code),
            Request::SetTfPower { .. } | Request::SetPrimaries { .. } => {
                Err(Error::UnsupportedFeature)
            }
            _ => Ok(()),
        };
        if let Err(error) = result {
            creator.post_error(error, message(error));
        }
    }
}

fn create(
    state: &mut Sparky,
    creator: &WpImageDescriptionCreatorParamsV1,
    init: &mut DataInit<'_, Sparky>,
    id: New<WpImageDescriptionV1>,
    built: Result<crate::color_management::description::ImageDescription, ParamsError>,
) {
    match built {
        Ok(description) => {
            let record = state.color_management.records.record(description);
            image::ready(init, id, record, Information::Denied);
        }
        Err(error) => creator.post_error(code(error), message(code(error))),
    }
}

const fn code(error: ParamsError) -> Error {
    match error {
        ParamsError::IncompleteSet => Error::IncompleteSet,
        ParamsError::AlreadySet => Error::AlreadySet,
        ParamsError::InvalidLuminance => Error::InvalidLuminance,
    }
}

const fn message(error: Error) -> &'static str {
    match error {
        Error::IncompleteSet => "faltam a curva ou as primárias",
        Error::AlreadySet => "propriedade já definida",
        Error::UnsupportedFeature => "recurso não anunciado",
        Error::InvalidTf => "curva de transferência não anunciada",
        Error::InvalidPrimariesNamed => "primárias não anunciadas",
        Error::InvalidLuminance => "luminâncias inválidas",
        _ => "parâmetros inválidos",
    }
}
