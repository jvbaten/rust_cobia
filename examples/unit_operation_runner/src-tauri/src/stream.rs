use cobia::*;

use crate::{
    information_stream::InformationStream,
    material_stream::MaterialStream,
    stream_specification::{StreamSpecification, StreamSpecificationType},
};

/// Enumeration for the kind of a stream.
///
/// Each variant wraps the underlying [`CapeObject`] handle and determines
/// which CAPE-OPEN implementation is used when borrowing stream state.
pub(crate) enum StreamType {
    /// CAPE-OPEN Material Stream
    Material(CapeObject),
    /// CAPE-OPEN Energy Stream
    Energy(CapeObject),
    /// CAPE-OPEN Information Stream
    Information(CapeObject),
}

/// A stream connected to one of the unit operation's ports.
///
/// Wraps a [`CapeObject`] of the appropriate type and provides a uniform
/// interface for reading/writing specifications regardless of whether the
/// underlying object is a material, energy, or information stream.
pub(crate) struct Stream {
    /// The kind and underlying object for this stream.
    pub(crate) stream_type: StreamType,
    /// Port name this stream is attached to, used as a human-readable label.
    pub(crate) name: String,
    /// `true` if this stream enters the unit operation (feed), `false` if it leaves (product).
    is_feed: bool,
}

impl Stream {
    /// Creates a [`Stream`] wrapping a material stream object.
    ///
    /// Initialises the embedded [`MaterialStream`] name, description, and
    /// feed/product flag to match the port.
    ///
    /// # Parameters
    /// - `is_feed`: `true` when the port is a feed port.
    /// - `port_name`: CAPE-OPEN string name of the port.
    /// - `obj`: Owned [`CapeObject`] implementing `ICapeThermoMaterial`.
    ///
    /// # Returns
    /// A new [`Stream`] of kind [`StreamType::Material`].
    pub(crate) fn material(is_feed: bool, port_name: &CapeStringImpl, obj: CapeObject) -> Self {
        let material_stream = unsafe { MaterialStream::borrow_mut(&obj) };
        material_stream.name = format!(
            "{} stream connected to {} port {}",
            if material_stream.is_feed {
                "Feed"
            } else {
                "Product"
            },
            if is_feed { "feed" } else { "product" },
            port_name.as_string()
        );
        material_stream.description = format!(
            "Material stream connected to {} port {}",
            if is_feed { "feed" } else { "product" },
            port_name.as_string()
        );
        material_stream.is_feed = is_feed;
        Self {
            stream_type: StreamType::Material(obj),
            name: port_name.to_string(),
            is_feed,
        }
    }
    /// Creates a [`Stream`] wrapping an energy stream object.
    ///
    /// Initialises the embedded [`InformationStream`] name, description, feed
    /// flag, and marks it as an energy stream.
    ///
    /// # Parameters
    /// - `is_feed`: `true` when the port is a feed port.
    /// - `port_name`: CAPE-OPEN string name of the port.
    /// - `obj`: Owned [`CapeObject`] implementing the energy-stream interfaces.
    ///
    /// # Returns
    /// A new [`Stream`] of kind [`StreamType::Energy`].
    pub(crate) fn energy(is_feed: bool, port_name: &CapeStringImpl, obj: CapeObject) -> Self {
        let energy_stream = unsafe { InformationStream::borrow_mut(&obj) };
        energy_stream.name = format!(
            "Energy stream connected to {} port {}",
            if is_feed { "feed" } else { "product" },
            port_name.as_string()
        );
        energy_stream.description = format!(
            "Energy stream connected to {} port {}",
            if is_feed { "feed" } else { "product" },
            port_name.as_string()
        );
        energy_stream.is_feed = is_feed;
        energy_stream.is_energy = true;
        Self {
            stream_type: StreamType::Energy(obj),
            name: port_name.to_string(),
            is_feed,
        }
    }

    /// Creates a [`Stream`] wrapping an information/signal stream object.
    ///
    /// Initialises the embedded [`InformationStream`] name, description, and
    /// feed flag; marks it as a non-energy information stream.
    ///
    /// # Parameters
    /// - `is_feed`: `true` when the port is a feed port.
    /// - `port_name`: CAPE-OPEN string name of the port.
    /// - `obj`: Owned [`CapeObject`] implementing the information-stream interfaces.
    ///
    /// # Returns
    /// A new [`Stream`] of kind [`StreamType::Information`].
    pub(crate) fn information(is_feed: bool, port_name: &CapeStringImpl, obj: CapeObject) -> Self {
        let information_stream = unsafe { InformationStream::borrow_mut(&obj) };
        information_stream.name = format!(
            "Information stream connected to {} port {}",
            if is_feed { "feed" } else { "product" },
            port_name.as_string()
        );
        information_stream.description = format!(
            "Information stream connected to {} port {}",
            if is_feed { "feed" } else { "product" },
            port_name.as_string()
        );
        information_stream.is_feed = is_feed;
        information_stream.is_energy = false;
        Self {
            stream_type: StreamType::Information(obj),
            name: port_name.to_string(),
            is_feed,
        }
    }

    /// Returns `true` if this stream is a feed to the unit operation.
    ///
    /// # Returns
    /// `true` for feed streams, `false` for product streams.
    pub fn feed(&self) -> bool {
        self.is_feed
    }
    /// Returns the last specification validation error for this stream, if any.
    ///
    /// Delegates to the underlying [`MaterialStream`] or [`InformationStream`].
    ///
    /// # Returns
    /// - `Some(message)` when the current set of specifications is invalid or inconsistent.
    /// - `None` when the specifications are valid
    pub fn specification_error(&self) -> Option<String> {
        match &self.stream_type {
            StreamType::Material(obj) => unsafe { MaterialStream::borrow(obj) }
                .specification_error
                .clone(),
            StreamType::Energy(obj) | StreamType::Information(obj) => {
                unsafe { InformationStream::borrow(obj) }
                    .specification_error
                    .clone()
            }
        }
    }
    /// Returns `true` when the stream has a complete, consistent set of specifications.
    ///
    /// # Returns
    /// `true` if the stream is fully specified and ready for calculation.
    pub fn is_specified(&self) -> bool {
        match &self.stream_type {
            StreamType::Material(obj) => unsafe { MaterialStream::borrow(obj) }.is_specified,
            StreamType::Energy(obj) | StreamType::Information(obj) => {
                unsafe { InformationStream::borrow(obj) }.is_specified
            }
        }
    }
    /// Adds or updates a single stream specification.
    ///
    /// Parses `value` and stores it against `spec_type` in the underlying
    /// stream implementation.
    ///
    /// # Parameters
    /// - `spec_type`: The [`StreamSpecificationType`] discriminant identifying what quantity is being specified.
    /// - `value`: Numeric value as a string (in SI units).
    ///
    /// # Returns
    /// `true` if the specification was accepted; `false` if the value could not
    /// be parsed or the specification is incompatible with the current set.
    pub fn specify(&mut self, spec_type: StreamSpecificationType, value: &str) -> bool {
        match &mut self.stream_type {
            StreamType::Material(obj) => {
                unsafe { MaterialStream::borrow_mut(obj) }.specify(spec_type, value)
            }
            StreamType::Energy(obj) | StreamType::Information(obj) => {
                unsafe { InformationStream::borrow_mut(obj) }.specify(spec_type, value)
            }
        }
    }
    /// Returns the current list of specifications for this stream.
    ///
    /// # Returns
    /// A reference to the vector of [`StreamSpecification`] entries stored in
    /// the underlying stream implementation.
    pub fn get_specifications(&self) -> &Vec<StreamSpecification> {
        match &self.stream_type {
            StreamType::Material(obj) => {
                unsafe { MaterialStream::borrow(obj) }.get_specifications()
            }
            StreamType::Energy(obj) | StreamType::Information(obj) => {
                unsafe { InformationStream::borrow(obj) }.get_specifications()
            }
        }
    }
    /// Replaces the current specification list with the supplied vector.
    ///
    /// Used during persistence restore to push a previously-saved set of
    /// specifications back into the stream.
    ///
    /// # Parameters
    /// - `specs`: The complete replacement list of [`StreamSpecification`] entries.
    pub fn set_specifications(&mut self, specs: Vec<StreamSpecification>) {
        match &mut self.stream_type {
            StreamType::Material(obj) => {
                unsafe { MaterialStream::borrow_mut(obj) }.set_specifications(specs);
            }
            StreamType::Energy(obj) | StreamType::Information(obj) => {
                unsafe { InformationStream::borrow_mut(obj) }.set_specifications(specs);
            }
        }
    }
    /// Applies the current specification list to the stream, triggering
    /// validation and (for material streams) flash calculation setup.
    pub fn apply_specifications(&mut self) {
        match &mut self.stream_type {
            StreamType::Material(obj) => {
                unsafe { MaterialStream::borrow_mut(obj) }.apply_specifications();
            }
            StreamType::Energy(obj) | StreamType::Information(obj) => {
                unsafe { InformationStream::borrow_mut(obj) }.apply_specifications();
            }
        }
    }

    /// Clears all calculated values from the stream, resetting it to the
    /// unresolved state after a topology or specification change.
    ///
    /// Does not discard the specifications themselves — only the results
    /// derived from them.
    pub fn clear_values(&mut self) {
        match &mut self.stream_type {
            StreamType::Material(obj) => {
                unsafe { MaterialStream::borrow_mut(obj) }.clear();
            }
            StreamType::Energy(obj) | StreamType::Information(obj) => {
                unsafe { InformationStream::borrow_mut(obj) }.clear_values();
            }
        }
    }
}
