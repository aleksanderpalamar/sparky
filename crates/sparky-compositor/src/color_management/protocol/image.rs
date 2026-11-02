use std::sync::Arc;

use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_image_description_info_v1::{
    self, WpImageDescriptionInfoV1,
};
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_image_description_v1::{
    Cause, Error, Request, WpImageDescriptionV1,
};
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, New, Resource};

use super::{info, names};
use crate::color_management::records::Record;
use crate::state::Sparky;

const READY2_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Information {
    Allowed,
    Denied,
}

pub enum ImageData {
    Ready {
        record: Arc<Record>,
        information: Information,
    },
    Failed,
}

pub fn ready(
    init: &mut DataInit<'_, Sparky>,
    id: New<WpImageDescriptionV1>,
    record: Arc<Record>,
    information: Information,
) {
    let (high, low) = names::identity_parts(record.identity);
    let image = init.init(
        id,
        ImageData::Ready {
            record,
            information,
        },
    );
    match image.version() >= READY2_VERSION {
        true => image.ready2(high, low),
        false => image.ready(low),
    }
}

pub fn failed(
    init: &mut DataInit<'_, Sparky>,
    id: New<WpImageDescriptionV1>,
    cause: Cause,
    message: &str,
) {
    let image = init.init(id, ImageData::Failed);
    image.failed(cause, message.to_owned());
}

pub fn record_of(image: &WpImageDescriptionV1) -> Option<Arc<Record>> {
    match image.data::<ImageData>()? {
        ImageData::Ready { record, .. } => Some(record.clone()),
        ImageData::Failed => None,
    }
}

impl Dispatch<WpImageDescriptionV1, ImageData> for Sparky {
    fn request(
        state: &mut Self,
        _: &Client,
        image: &WpImageDescriptionV1,
        request: Request,
        data: &ImageData,
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        let Request::GetInformation { information } = request else {
            return;
        };
        match data {
            ImageData::Failed => {
                image.post_error(Error::NotReady, "a descrição de cor não ficou pronta");
            }
            ImageData::Ready {
                information: Information::Denied,
                ..
            } => image.post_error(
                Error::NoInformation,
                "esta descrição de cor não tem detalhes",
            ),
            ImageData::Ready {
                record,
                information: Information::Allowed,
            } => {
                let info = init.init(information, ());
                let description = record.description;
                state
                    .loop_handle
                    .insert_idle(move |_| info::send(&info, &description));
            }
        }
    }
}

impl Dispatch<WpImageDescriptionInfoV1, ()> for Sparky {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &WpImageDescriptionInfoV1,
        _: wp_image_description_info_v1::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
    }
}
