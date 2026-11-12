//! CAD that runs on a potato: a `no_std` core for tiny parametric CAD on microcontrollers.
//!
//! This first release is the STL writer ([`stl`]); the model, sketches, features and renderer follow.
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod stl;
