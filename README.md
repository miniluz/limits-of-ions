## The limits of ions

A simple program made to test the limits of what the
[DAWN](https://wiki.kerbalspaceprogram.com/wiki/IX-6315_%22Dawn%22_Electric_Propulsion_System)
ion engine can do in [Kerbal Space Program](https://en.wikipedia.org/wiki/Kerbal_Space_Program).

It's based on my attempt to make a little ion lander,
where I noticed that adding batteries to allow the burning of a certain
[delta-v](https://en.wikipedia.org/wiki/Delta-v)
in one manouver even at nighttime
drastically reduced the delta-v. I wanted to test the engine's limits.

## Project

It's split in two files: [`main.rs`](src/main.rs) contains the business logic
and [`consts.rs`](src/const.rs) contains the constants required.

The program calculates for a single dawn engine a [few tables](results) for some amounts of dead weight.
Each table has on the X axis
a target delta-v that has to be executable in a single burn on battery power
and on the Y axis
an amount of fuel tanks.
The amount of batteries to execute that burn in a single manouver is calculated,
and displayed in the corresponding cell
along with the total delta-v
and [thrust to weight ratio](https://en.wikipedia.org/wiki/Thrust-to-weight_ratio).

The most interesting table is perhaps the one with [0 kg of dead weight added](results/limits_of_ions_0kg_dead_weight.txt)
as it represents the true limit of ion engines,
as if the rocket were to keep adding more engines
with the same amount of fuel tanks and batteries per engine
the results would approach that table
as the effects of the dead weight lessen.

## Running and contributing

To generate the results yourself, simply run `cargo run`.

With [Nix](https://nixos.wiki/wiki/Nix_package_manager)
and [direnv](https://direnv.net/) installed,
you can automatically install the Rust toolchain used isolated from the rest of your system.
Otherwise, you must have [Rust](https://www.rust-lang.org/tools/install) installed to run it.

I don't expect the project will need to be updated or fixed but if you notice something let me know.