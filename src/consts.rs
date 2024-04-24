/**
 * Everything is in SI units
 * Constants gotten from the following KSP wiki articles:
 * * [IX-6315 "Dawn" Electric Propulsion System](https://wiki.kerbalspaceprogram.com/wiki/IX-6315_%22Dawn%22_Electric_Propulsion_System)
 * * [Xenon gas](https://wiki.kerbalspaceprogram.com/wiki/Xenon_gas)
 * * [PB-X50R Xenon Container](https://wiki.kerbalspaceprogram.com/wiki/PB-X50R_Xenon_Container)
 * * [Z-100 Rechargeable Battery Pack](https://wiki.kerbalspaceprogram.com/wiki/Z-100_Rechargeable_Battery_Pack)
 **/

/// in kg
pub const DAWN_MASS: f64 = 250.;
/// in N
pub const DAWN_THRUST: f64 = 2000.;
/// in s
pub const DAWN_ISP: f64 = 4200.;
/// in units/sec
pub const DAWN_XENON_DRAIN: f64 = 0.486;
/// in ec/sec
pub const DAWN_EC_DRAIN: f64 = 8.74;

/// in kg/unit
pub const XENON_DENSITY: f64 = 0.1;
/// in kg
pub const MIN_XENON_TANK_DRY: f64 = 13.5;
/// in units
pub const MIN_XENON_TANK_CAPACITY: f64 = 405.;

/// in kg
pub const MIN_BATTERY_MASS: f64 = 5.;
/// in ec
pub const MIN_BATTERY_CAPACITY: f64 = 100.;

/// in m/s²
pub const G_0: f64 = 9.80665;
/// in m/s²
pub const G_KERBIN: f64 = 9.81;
