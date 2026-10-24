//! One face for the two data-control protocols (R1, R2).
//!
//! `ext-data-control-v1` and `wlr-data-control-unstable-v1` have the same shape, so the
//! wrappers here hide which one a connection speaks. The generated bindings come from
//! `wayland-protocols` (feature `staging`) and `wayland-protocols-wlr`: no XML is vendored, and
//! the pure-Rust `wayland-client` backend means no libwayland (Constitution I).
//!
//! A state type becomes a data-control client by implementing [`Handler`] and invoking
//! [`impl_data_control_dispatch!`].

use std::os::fd::{BorrowedFd, OwnedFd};
use std::sync::Mutex;

use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Dispatch, Proxy, QueueHandle};
use wayland_protocols::ext::data_control::v1::client as ext;
use wayland_protocols_wlr::data_control::v1::client as wlr;

use ext::{
    ext_data_control_device_v1::ExtDataControlDeviceV1 as ExtDevice,
    ext_data_control_manager_v1::ExtDataControlManagerV1 as ExtManager,
    ext_data_control_offer_v1::ExtDataControlOfferV1 as ExtOffer,
    ext_data_control_source_v1::ExtDataControlSourceV1 as ExtSource,
};
use wlr::{
    zwlr_data_control_device_v1::ZwlrDataControlDeviceV1 as WlrDevice,
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1 as WlrManager,
    zwlr_data_control_offer_v1::ZwlrDataControlOfferV1 as WlrOffer,
    zwlr_data_control_source_v1::ZwlrDataControlSourceV1 as WlrSource,
};

/// Registry interface name of the standard manager.
pub const EXT_MANAGER: &str = "ext_data_control_manager_v1";
/// Registry interface name of the wlroots manager.
pub const WLR_MANAGER: &str = "zwlr_data_control_manager_v1";

/// Which data-control protocol a connection speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// `ext_data_control_manager_v1`.
    Ext,
    /// `zwlr_data_control_manager_v1`.
    Zwlr,
}

impl Protocol {
    /// Registry interface name of this protocol's manager.
    #[must_use]
    pub fn interface(self) -> &'static str {
        match self {
            Self::Ext => EXT_MANAGER,
            Self::Zwlr => WLR_MANAGER,
        }
    }

    /// The highest manager version this build understands.
    #[must_use]
    pub fn known_version(self) -> u32 {
        match self {
            Self::Ext => 1,
            Self::Zwlr => 2,
        }
    }
}

/// Everything a dispatching state must handle. Called from the event queue.
pub trait Handler {
    /// The selection changed; `None` means no owner.
    fn selection(&mut self, offer: Option<Offer>);
    /// The device is finished and must not be used again.
    fn finished(&mut self);
    /// A reader asked source `source` for `mime`; write to `fd` and close it.
    fn send(&mut self, source: u64, mime: String, fd: OwnedFd);
    /// Source `source` lost the selection.
    fn cancelled(&mut self, source: u64);
}

/// User data of an offer: the MIME types it announced.
#[derive(Debug, Default)]
pub struct OfferData {
    mimes: Mutex<Vec<String>>,
}

/// User data of a source: the id the owner gave it.
#[derive(Debug, Clone, Copy)]
pub struct SourceData(pub u64);

/// Marker for states that dispatch every data-control object.
pub trait DcState:
    Handler
    + Dispatch<ExtManager, ()>
    + Dispatch<ExtDevice, ()>
    + Dispatch<ExtSource, SourceData>
    + Dispatch<ExtOffer, OfferData>
    + Dispatch<WlrManager, ()>
    + Dispatch<WlrDevice, ()>
    + Dispatch<WlrSource, SourceData>
    + Dispatch<WlrOffer, OfferData>
    + 'static
{
}

impl<T> DcState for T where
    T: Handler
        + Dispatch<ExtManager, ()>
        + Dispatch<ExtDevice, ()>
        + Dispatch<ExtSource, SourceData>
        + Dispatch<ExtOffer, OfferData>
        + Dispatch<WlrManager, ()>
        + Dispatch<WlrDevice, ()>
        + Dispatch<WlrSource, SourceData>
        + Dispatch<WlrOffer, OfferData>
        + 'static
{
}

/// The bound manager global.
#[derive(Debug, Clone)]
pub enum Manager {
    /// Standard protocol.
    Ext(ExtManager),
    /// wlroots protocol.
    Zwlr(WlrManager),
}

impl Manager {
    /// Bind global `name` at the highest version both sides know.
    pub fn bind<D: DcState>(
        registry: &WlRegistry,
        protocol: Protocol,
        name: u32,
        advertised: u32,
        qh: &QueueHandle<D>,
    ) -> Self {
        let version = advertised.clamp(1, protocol.known_version());
        match protocol {
            Protocol::Ext => Self::Ext(registry.bind(name, version, qh, ())),
            Protocol::Zwlr => Self::Zwlr(registry.bind(name, version, qh, ())),
        }
    }

    /// The protocol this manager speaks.
    #[must_use]
    pub fn protocol(&self) -> Protocol {
        match self {
            Self::Ext(_) => Protocol::Ext,
            Self::Zwlr(_) => Protocol::Zwlr,
        }
    }

    /// The data device of `seat`.
    pub fn get_data_device<D: DcState>(&self, seat: &WlSeat, qh: &QueueHandle<D>) -> Device {
        match self {
            Self::Ext(m) => Device::Ext(m.get_data_device(seat, qh, ())),
            Self::Zwlr(m) => Device::Zwlr(m.get_data_device(seat, qh, ())),
        }
    }

    /// A new source tagged `id`.
    pub fn create_data_source<D: DcState>(&self, id: u64, qh: &QueueHandle<D>) -> Source {
        match self {
            Self::Ext(m) => Source::Ext(m.create_data_source(qh, SourceData(id))),
            Self::Zwlr(m) => Source::Zwlr(m.create_data_source(qh, SourceData(id))),
        }
    }
}

/// A seat's data device.
#[derive(Debug, Clone)]
pub enum Device {
    /// Standard protocol.
    Ext(ExtDevice),
    /// wlroots protocol.
    Zwlr(WlrDevice),
}

impl Device {
    /// Become the selection owner with `source`, or clear the selection with `None`.
    pub fn set_selection(&self, source: Option<&Source>) {
        match (self, source) {
            (Self::Ext(d), Some(Source::Ext(s))) => d.set_selection(Some(s)),
            (Self::Ext(d), None) => d.set_selection(None),
            (Self::Zwlr(d), Some(Source::Zwlr(s))) => d.set_selection(Some(s)),
            (Self::Zwlr(d), None) => d.set_selection(None),
            // A source of the other protocol cannot exist on this connection.
            _ => {}
        }
    }
}

/// Data we offer.
#[derive(Debug, Clone)]
pub enum Source {
    /// Standard protocol.
    Ext(ExtSource),
    /// wlroots protocol.
    Zwlr(WlrSource),
}

impl Source {
    /// Advertise one MIME type.
    pub fn offer(&self, mime: &str) {
        match self {
            Self::Ext(s) => s.offer(mime.to_string()),
            Self::Zwlr(s) => s.offer(mime.to_string()),
        }
    }

    /// Release the source.
    pub fn destroy(&self) {
        match self {
            Self::Ext(s) => s.destroy(),
            Self::Zwlr(s) => s.destroy(),
        }
    }
}

/// Data someone else offers.
#[derive(Debug, Clone)]
pub enum Offer {
    /// Standard protocol.
    Ext(ExtOffer),
    /// wlroots protocol.
    Zwlr(WlrOffer),
}

impl Offer {
    /// The MIME types announced so far, verbatim and in order.
    #[must_use]
    pub fn mimes(&self) -> Vec<String> {
        let data = match self {
            Self::Ext(o) => o.data::<OfferData>(),
            Self::Zwlr(o) => o.data::<OfferData>(),
        };
        data.and_then(|d| d.mimes.lock().ok().map(|m| m.clone()))
            .unwrap_or_default()
    }

    /// Ask the owner to write `mime` to `fd`.
    pub fn receive(&self, mime: &str, fd: BorrowedFd<'_>) {
        match self {
            Self::Ext(o) => o.receive(mime.to_string(), fd),
            Self::Zwlr(o) => o.receive(mime.to_string(), fd),
        }
    }

    /// Release the offer.
    pub fn destroy(&self) {
        match self {
            Self::Ext(o) => o.destroy(),
            Self::Zwlr(o) => o.destroy(),
        }
    }
}

/// Record an announced MIME type on an offer's user data.
#[doc(hidden)]
pub fn push_mime(data: &OfferData, mime: String) {
    if let Ok(mut m) = data.mimes.lock() {
        m.push(mime);
    }
}

/// Implement every `Dispatch` a data-control client needs for `$state`, forwarding to
/// [`Handler`]. The state still implements `Dispatch` for `wl_registry`, `wl_seat` and
/// `wl_callback` itself.
#[macro_export]
#[doc(hidden)]
macro_rules! impl_data_control_dispatch {
    ($state:ty) => {
        $crate::impl_data_control_dispatch!(@one $state,
            ::wayland_protocols::ext::data_control::v1::client,
            [ext_data_control_manager_v1, ext_data_control_device_v1,
             ext_data_control_source_v1, ext_data_control_offer_v1],
            ExtDataControlManagerV1, ExtDataControlDeviceV1, ExtDataControlSourceV1,
            ExtDataControlOfferV1, Ext);
        $crate::impl_data_control_dispatch!(@one $state,
            ::wayland_protocols_wlr::data_control::v1::client,
            [zwlr_data_control_manager_v1, zwlr_data_control_device_v1,
             zwlr_data_control_source_v1, zwlr_data_control_offer_v1],
            ZwlrDataControlManagerV1, ZwlrDataControlDeviceV1, ZwlrDataControlSourceV1,
            ZwlrDataControlOfferV1, Zwlr);
    };
    (@one $state:ty, $base:path, [$mgr_m:ident, $dev_m:ident, $src_m:ident, $off_m:ident], $mgr:ident, $dev:ident, $src:ident, $off:ident, $variant:ident) => {
        const _: () = {
            use $base as p;
            use ::wayland_client::{Connection, Dispatch, QueueHandle};
            use $crate::backend::wayland::protocols::{Handler, Offer, OfferData, SourceData};

            impl Dispatch<p::$mgr_m::$mgr, ()> for $state {
                fn event(
                    _: &mut Self,
                    _: &p::$mgr_m::$mgr,
                    _: p::$mgr_m::Event,
                    _: &(),
                    _: &Connection,
                    _: &QueueHandle<Self>,
                ) {
                }
            }

            impl Dispatch<p::$dev_m::$dev, ()> for $state {
                fn event(
                    state: &mut Self,
                    _: &p::$dev_m::$dev,
                    event: p::$dev_m::Event,
                    _: &(),
                    _: &Connection,
                    _: &QueueHandle<Self>,
                ) {
                    match event {
                        p::$dev_m::Event::Selection { id } => {
                            Handler::selection(state, id.map(Offer::$variant));
                        }
                        p::$dev_m::Event::PrimarySelection { id: Some(o) } => o.destroy(),
                        p::$dev_m::Event::Finished => Handler::finished(state),
                        _ => {}
                    }
                }

                ::wayland_client::event_created_child!($state, p::$dev_m::$dev, [
                    p::$dev_m::EVT_DATA_OFFER_OPCODE => (p::$off_m::$off, OfferData::default()),
                ]);
            }

            impl Dispatch<p::$off_m::$off, OfferData> for $state {
                fn event(
                    _: &mut Self,
                    _: &p::$off_m::$off,
                    event: p::$off_m::Event,
                    data: &OfferData,
                    _: &Connection,
                    _: &QueueHandle<Self>,
                ) {
                    if let p::$off_m::Event::Offer { mime_type } = event {
                        $crate::backend::wayland::protocols::push_mime(data, mime_type);
                    }
                }
            }

            impl Dispatch<p::$src_m::$src, SourceData> for $state {
                fn event(
                    state: &mut Self,
                    _: &p::$src_m::$src,
                    event: p::$src_m::Event,
                    data: &SourceData,
                    _: &Connection,
                    _: &QueueHandle<Self>,
                ) {
                    match event {
                        p::$src_m::Event::Send { mime_type, fd } => {
                            Handler::send(state, data.0, mime_type, fd);
                        }
                        p::$src_m::Event::Cancelled => Handler::cancelled(state, data.0),
                        _ => {}
                    }
                }
            }
        };
    };
}
