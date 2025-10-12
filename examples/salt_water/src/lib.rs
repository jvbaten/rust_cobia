//! Salt-Water stand-alone CAPE-OPEN 1.2 Property Package
//!
//! This is an example property package implementation, featuring
//! a single water phase and two components: water and NaCl. The 
//! calculations for the liquid phase are based on 
//! * Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
//! * Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
//! 
//! The package support calculation of the following liquid properties:
//! * Density
//! * Volume
//! * Enthalpy
//! * Entropy
//! * Viscosity
//! * Thermal conductivity
//!
//! See the [salt_water_calculator] documentation fo validity ranges.
//!
//! Phase equilibria routines are provided that calculate the conditions
//! of the liquid phase, at given temperaure and pressure, temperature and
//! enthalpy, and temperature and entropy, which makes the packge suitable
//! for steady state flowsheet calculations including heat exchangers, 
//! pumps and turbines.
//! 
//! To use the property package from source, compile the source, and 
//! register the property package using
//!
//! `cobiaRegister.exe salt_water.dll`
//!
//! An installer for Windows is made available through the AmsterCHEM web site.
//!
//! This software uses strum, a set of macros and traits for working with enums and strings easier in Rust, by Peter GlotFelty

use cobia;
mod salt_water_calculator;
mod salt_water_property_package;
mod property_tables;
mod phase_equilibrium_type;

/// Registering a PMC for all users requires administrative privileges; this package is registred for the current user only
fn register_pmcs_for_all_users() -> bool {
	false
}

///A list of all the PMCs that need to be registered
static PMCS: &[cobia::PMCInfo] = &[cobia::pmc_info::<salt_water_property_package::SaltWaterPropertyPackage>()];

//this defines the entry point for the cobia PMC shared library
cobia::pmc_entry_points!(PMCS, register_pmcs_for_all_users());
