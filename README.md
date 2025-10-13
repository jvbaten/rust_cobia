# Rust binding for COBIA

(C) Jasper, AmsterCHEM 2025

This workspace implements rust bindings for COBIA.

CAPE-OPEN consists of a series of specifications to expand the range of application of process simulation technologies. 
The CAPE-OPEN specifications specify a set of software interfaces that allow plug and play inter-operability between
a given process modelling environment (PME) and a third-party process modelling component (PMC).

The CAPE-OPEN specifications are supported by the non-profit organization CO-LaN: http://www.colan.org

The COBIA middle-ware is a platform independent object model and request broker implementation to
facilitate the inter-operation of CAPE-OPEN compliant PMEs and PMCs.

This workspace provides:
* the `cobia` crate, a rust language binding for CO-LaN’s COBIA middleware
* the `salt_water` crate, a CAPE-OPEN Property Package PMC example, for water with NaCl
* the `distillation_shortcut_unit` crate, a CAPE-OPEN Unit Operation PMC

# License

This project is provided under the MIT license. See LICENSE for more details.

# Prerequisites

To build the COBIA crate, the following must be installed on your computer:

* The COBIA SDK (available for download from [https://colan.repositoryhosting.com/trac/colan_cobia/downloads](https://colan.repositoryhosting.com/trac/colan_cobia/downloads) 
* The required ingredients for bindgen, including a valid CLANG installation, see [https://rust-lang.github.io/rust-bindgen/requirements.html](https://rust-lang.github.io/rust-bindgen/requirements.html)
* The rust compiler, edition 2024 or up



