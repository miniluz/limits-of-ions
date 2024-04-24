mod consts;
use consts::*;
use conv::{prelude::*, RoundToPosInf};
use tracing::debug;
use tracing_subscriber::layer::SubscriberExt;

fn main() {
    let subscriber =
        tracing_subscriber::Registry::default().with(tracing_tree::HierarchicalLayer::new(2));

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // safety assumption: should always be convertible to f64. really no reason for it not to be.
    const MAX_TANKS: usize = 10;
    const ITERATION_LIMIT: usize = 100;

    const TARGET_DVS: [f64; 24] = [
        10., 20., 30., 50., 80., 100., 150., 200., 250., 300., 350., 400., 450., 500., 600., 700.,
        800., 900., 1000., 1100., 1200., 1300., 1400., 1500.,
    ];
    const TARGET_DVS_LEN: usize = TARGET_DVS.len();

    struct Result {
        dv: f64,
        twr: f64,
        battery_num: usize,
    }

    impl Result {
        fn to_string(self) -> String {
            format!(
                "{:.2} m/s ΔV\n{:.3} TWR\n{} batteries",
                self.dv, self.twr, self.battery_num
            )
        }
    }

    let mut results: [[Option<Result>; TARGET_DVS_LEN]; MAX_TANKS] = Default::default();

    for (target_dv_index, target_dv) in TARGET_DVS.iter().enumerate() {
        let _span = tracing::debug_span!("Target ΔV: {}", target_dv).entered();

        for tank_num in 1..=MAX_TANKS {
            let _span = tracing::debug_span!("Number of tanks: {}", tank_num).entered();

            let tank_num_float =
                f64::value_from(tank_num).expect("Tank number should be representable in f64");

            let max_xenon = tank_num_float * MIN_XENON_TANK_CAPACITY;
            debug!(max_xenon);

            let mut battery_num = 0;
            let battery_num: Option<usize> = (|| {
                for _ in 0..ITERATION_LIMIT {
                    let _span =
                        tracing::debug_span!("Number of batteries: {}", battery_num).entered();

                    let (_dry_mass, wet_mass) = dry_and_wet_mass(tank_num, battery_num);
                    debug!(wet_mass);

                    let required_xenon =
                        xenon_required_for_dv_at_initial_burn(*target_dv, wet_mass, max_xenon);
                    let required_xenon = match required_xenon {
                        Some(required_xenon) => required_xenon,
                        None => None?,
                    };
                    debug!(required_xenon);

                    let required_batteries = required_batteries_for_xenon(required_xenon);
                    debug!(required_batteries);

                    if battery_num >= required_batteries {
                        return Some(required_batteries);
                    } else {
                        battery_num = required_batteries;
                    }
                }
                return None;
            })();

            results[tank_num - 1][target_dv_index] = battery_num.map(|battery_num| {
                let (dry_mass, wet_mass) = dry_and_wet_mass(tank_num, battery_num);
                let twr = DAWN_THRUST / wet_mass / G_KERBIN;
                let dv = DAWN_ISP * G_0 * f64::ln(wet_mass / dry_mass);
                Result {
                    dv,
                    twr,
                    battery_num,
                }
            });
        }
    }

    let results = results.map(|results| {
        results.map(|option| option.map_or("Unachievable".to_owned(), Result::to_string))
    });

    let mut table = comfy_table::Table::new();
    table.set_header({
        let r: [String; TARGET_DVS_LEN + 1] = std::array::from_fn(|i| {
            if i == 0 {
                "Max ΔV from single burn →\nNumber of tanks ↓".to_owned()
            } else {
                format!("{:.0} m/s", TARGET_DVS[i - 1])
            }
        });
        r
    });

    for (tank_number_minus_one, results) in results.into_iter().enumerate() {
        let r: [String; TARGET_DVS_LEN + 1] = std::array::from_fn(|i| {
            if i == 0 {
                (tank_number_minus_one + 1).to_string()
            } else {
                results[i - 1].clone()
            }
        });
        table.add_row(r);
    }

    table
        .column_iter_mut()
        .skip(1)
        .for_each(|column| column.set_cell_alignment(comfy_table::CellAlignment::Right));
    table
        .column_mut(0)
        .expect("Always should be header column")
        .set_cell_alignment(comfy_table::CellAlignment::Center);

    println!("{table}");
}

/// returns (dry_mass, wet_mass)
fn dry_and_wet_mass(tank_num: usize, battery_num: usize) -> (f64, f64) {
    let tank_num: f64 = tank_num
        .value_into()
        .expect("Tank number should be representable as f64");
    let battery_num: f64 = battery_num
        .value_into()
        .expect("Battery number should be representable as f64");

    let dry_mass = DAWN_MASS + MIN_XENON_TANK_DRY * tank_num + MIN_BATTERY_MASS * battery_num;

    let wet_mass = dry_mass + MIN_XENON_TANK_CAPACITY * tank_num * XENON_DENSITY;

    (dry_mass, wet_mass)
}

fn xenon_required_for_dv_at_initial_burn(dv: f64, wet_mass: f64, max_xenon: f64) -> Option<f64> {
    /*
        dV = Isp * g0 * ln (wet_mass / dry_mass)
        wet_mass / dry_mass = e^(dV/Isp/g0)
        wet_mass / (wet_mass - d_xe * xe) = e^(dV/Isp/g0)
    */

    let exponential_term = f64::exp(dv / DAWN_ISP / G_0);

    /*
        wet_mass = et * wet_mass - et * (d_xe * xe)
        et * d_xe * xe = (et - 1) wet_mass
        xe = (et - 1) * wet_mass / et / d_xe
    */

    let required_xenon = (exponential_term - 1.) * wet_mass / exponential_term / XENON_DENSITY;

    if required_xenon <= max_xenon {
        Some(required_xenon)
    } else {
        None
    }
}

fn required_batteries_for_xenon(xenon: f64) -> usize {
    let required_energy = xenon / DAWN_XENON_DRAIN * DAWN_EC_DRAIN;
    let required_batteries = <usize as ApproxFrom<f64, RoundToPosInf>>::approx_from(
        required_energy / MIN_BATTERY_CAPACITY,
    )
    .expect("Required batteries should be representable in usize");

    required_batteries
}
