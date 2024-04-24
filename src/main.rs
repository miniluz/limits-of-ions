mod consts;

use consts::*;
use conv::{prelude::*, RoundToPosInf};
use std::io::Write;
use tracing::trace;
use tracing_subscriber::layer::SubscriberExt;

const TARGET_DVS: [f64; 24] = [
    10., 20., 30., 50., 80., 100., 150., 200., 250., 300., 350., 400., 450., 500., 600., 700.,
    800., 900., 1000., 1100., 1200., 1300., 1400., 1500.,
];
const TARGET_DVS_LEN: usize = TARGET_DVS.len();

const TARGET_TANKS: [i32; 10] = [1, 2, 4, 5, 6, 8, 10, 12, 15, 20];
const TARGET_TANKS_LEN: usize = TARGET_TANKS.len();

struct Result {
    dv: f64,
    twr: f64,
    battery_num: i32,
}

impl Result {
    fn to_string(self) -> String {
        format!(
            "{:.2} m/s ΔV\n{:.3} TWR\n{} batteries",
            self.dv, self.twr, self.battery_num
        )
    }
}

type Results = [[Option<Result>; TARGET_DVS_LEN]; TARGET_TANKS_LEN];

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let subscriber = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with(tracing_tree::HierarchicalLayer::new(2));

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    std::fs::create_dir_all("results")?;

    for added_dead_weight in [0, 100, 200, 300, 400, 500] {
        let _span = tracing::trace_span!("Added dead weight: {}", added_dead_weight).entered();

        let added_dead_weight_float = f64::value_from(added_dead_weight)?;

        let results = generate_results(added_dead_weight_float);

        let filename = format!(
            "results/limits_of_ions_{}kg_dead_weight.txt",
            added_dead_weight
        );
        let mut file = std::fs::File::create(filename)?;

        write!(file, "{}", generate_table(results))?;
    }

    Ok(())
}

fn generate_results(added_dead_weight: f64) -> Results {
    let mut results: Results = Default::default();

    for (target_dv_index, target_dv) in TARGET_DVS.iter().enumerate() {
        let _span = tracing::trace_span!("Target ΔV: {}", target_dv).entered();

        for (tank_index, tank_num) in TARGET_TANKS.iter().enumerate() {
            let _span = tracing::trace_span!("Number of tanks: {}", tank_num).entered();

            let max_xenon = (*tank_num as f64) * MIN_XENON_TANK_CAPACITY;
            trace!(max_xenon);

            let mut current_batteries = 0;
            let mut required_batteries = None;

            const ITERATION_LIMIT: usize = 100;
            for _ in 0..ITERATION_LIMIT {
                let new_batteries = get_number_of_batteries(
                    *tank_num,
                    current_batteries,
                    *target_dv,
                    max_xenon,
                    added_dead_weight,
                );

                let new_batteries = match new_batteries {
                    Some(new_batteries) => new_batteries,
                    None => break,
                };

                if current_batteries >= new_batteries {
                    required_batteries = Some(new_batteries);
                } else {
                    current_batteries = new_batteries
                }
            }

            results[tank_index][target_dv_index] = required_batteries.map(|battery_num| {
                let (dry_mass, wet_mass) =
                    dry_and_wet_mass(*tank_num, battery_num, added_dead_weight);
                Result {
                    dv: delta_v(dry_mass, wet_mass),
                    twr: twr(wet_mass),
                    battery_num,
                }
            });
        }
    }

    results
}

/// returns (dry_mass, wet_mass)
fn dry_and_wet_mass(tank_num: i32, battery_num: i32, added_dead_weight: f64) -> (f64, f64) {
    let tank_num: f64 = tank_num
        .value_into()
        .expect("Tank number should be representable as f64");
    let battery_num: f64 = battery_num
        .value_into()
        .expect("Battery number should be representable as f64");

    let dry_mass = DAWN_MASS
        + MIN_XENON_TANK_DRY * tank_num
        + MIN_BATTERY_MASS * battery_num
        + added_dead_weight;

    let wet_mass = dry_mass + MIN_XENON_TANK_CAPACITY * tank_num * XENON_DENSITY;

    (dry_mass, wet_mass)
}

fn delta_v(dry_mass: f64, wet_mass: f64) -> f64 {
    DAWN_ISP * G_0 * f64::ln(wet_mass / dry_mass)
}

fn twr(wet_mass: f64) -> f64 {
    DAWN_THRUST / wet_mass / G_KERBIN
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

fn required_batteries_for_xenon(xenon: f64) -> i32 {
    let required_energy = xenon / DAWN_XENON_DRAIN * DAWN_EC_DRAIN;
    let required_batteries = <i32 as ApproxFrom<f64, RoundToPosInf>>::approx_from(
        required_energy / MIN_BATTERY_CAPACITY,
    )
    .expect("Required batteries should be representable in usize");

    required_batteries
}

fn get_number_of_batteries(
    tank_num: i32,
    battery_num: i32,
    target_dv: f64,
    max_xenon: f64,
    added_dead_weight: f64,
) -> Option<i32> {
    let _span = tracing::debug_span!("Number of batteries: {}", battery_num).entered();

    let (_dry_mass, wet_mass) = dry_and_wet_mass(tank_num, battery_num, added_dead_weight);
    trace!(wet_mass);

    let required_xenon = xenon_required_for_dv_at_initial_burn(target_dv, wet_mass, max_xenon)?;
    trace!(required_xenon);

    let required_batteries = required_batteries_for_xenon(required_xenon);
    trace!(required_batteries);

    Some(required_batteries)
}

fn generate_table(results: Results) -> comfy_table::Table {
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

    for (tank_index, results) in results.into_iter().enumerate() {
        let r: [String; TARGET_DVS_LEN + 1] = std::array::from_fn(|i| {
            if i == 0 {
                format!("{}", TARGET_TANKS[tank_index])
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

    table
}
