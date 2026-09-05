//! Advisory option names from the SDC 1.9 and Vivado command references.
//! Sources and dialect boundaries are documented in `docs/command_options.md`.

use super::Dialect;

/// Returns known options for a command, without rejecting vendor extensions.
pub(crate) fn command_options(dialect: Dialect, command: &str) -> Vec<&'static str> {
    if dialect == Dialect::Tcl {
        return Vec::new();
    }
    let mut options = common_options(command).to_vec();
    if dialect == Dialect::Xdc && command == "set_clock_uncertainty" {
        options.retain(|option| !matches!(*option, "-rise" | "-fall"));
    }
    if matches!(
        command,
        "set_false_path"
            | "set_multicycle_path"
            | "set_max_delay"
            | "set_min_delay"
            | "group_path"
            | "set_bus_skew"
    ) {
        options.extend([
            "-from",
            "-rise_from",
            "-fall_from",
            "-to",
            "-rise_to",
            "-fall_to",
            "-through",
            "-rise_through",
            "-fall_through",
        ]);
    }
    if dialect == Dialect::Xdc && command == "group_path" {
        options.retain(|option| !option.starts_with("-rise_") && !option.starts_with("-fall_"));
    }
    options.extend_from_slice(dialect_options(dialect, command));
    // Vivado's quiet/verbose switches apply to the commands described here.
    if dialect == Dialect::Xdc && (!options.is_empty() || command == "create_pblock") {
        options.extend(["-quiet", "-verbose"]);
    }
    options.sort_unstable();
    options.dedup();
    options
}

fn common_options(command: &str) -> &'static [&'static str] {
    match command {
        "create_clock" => &["-name", "-period", "-waveform", "-add"],
        "create_generated_clock" => &[
            "-name",
            "-source",
            "-edges",
            "-divide_by",
            "-multiply_by",
            "-duty_cycle",
            "-edge_shift",
            "-add",
            "-master_clock",
            "-combinational",
            "-invert",
        ],
        "set_input_delay" | "set_output_delay" => &[
            "-clock",
            "-clock_fall",
            "-rise",
            "-fall",
            "-max",
            "-min",
            "-add_delay",
            "-network_latency_included",
            "-source_latency_included",
        ],
        "set_clock_groups" => &[
            "-name",
            "-logically_exclusive",
            "-physically_exclusive",
            "-asynchronous",
            "-group",
        ],
        "set_clock_latency" => &[
            "-rise", "-fall", "-min", "-max", "-source", "-early", "-late", "-clock",
        ],
        "set_clock_uncertainty" => &[
            "-from",
            "-rise_from",
            "-fall_from",
            "-to",
            "-rise_to",
            "-fall_to",
            "-rise",
            "-fall",
            "-setup",
            "-hold",
        ],
        "set_false_path" => &["-setup", "-hold", "-rise", "-fall"],
        "set_multicycle_path" => &["-setup", "-hold", "-rise", "-fall", "-start", "-end"],
        "set_max_delay" | "set_min_delay" => &["-rise", "-fall"],
        "group_path" => &["-name", "-weight"],
        "set_disable_timing" => &["-from", "-to"],
        "set_data_check" => &[
            "-from",
            "-to",
            "-rise_from",
            "-fall_from",
            "-rise_to",
            "-fall_to",
            "-setup",
            "-hold",
            "-clock",
        ],
        "get_ports" | "get_pins" | "get_cells" | "get_nets" | "get_clocks" => {
            &["-regexp", "-nocase"]
        }
        _ => &[],
    }
}

fn dialect_options(dialect: Dialect, command: &str) -> &'static [&'static str] {
    match (dialect, command) {
        (Dialect::Sdc, "set_input_delay" | "set_output_delay") => &["-level_sensitive"],
        (Dialect::Sdc, "set_clock_groups") => &["-allow_paths"],
        (Dialect::Sdc, "group_path") => &["-default"],
        (Dialect::Sdc, "get_cells" | "get_nets" | "get_pins") => {
            &["-hierarchical", "-hsc", "-of_objects"]
        }
        (Dialect::Sdc, "set_input_transition" | "set_clock_transition") => {
            &["-rise", "-fall", "-min", "-max"]
        }
        (Dialect::Sdc, "set_max_transition") => &["-clock_path", "-data_path", "-rise", "-fall"],
        (Dialect::Xdc, "set_input_delay" | "set_output_delay") => &["-reference_pin"],
        (Dialect::Xdc, "set_max_delay") => &["-datapath_only", "-reset_path"],
        (Dialect::Xdc, "set_min_delay" | "set_false_path" | "set_multicycle_path") => {
            &["-reset_path"]
        }
        (Dialect::Xdc, "get_ports") => &[
            "-filter",
            "-of_objects",
            "-match_style",
            "-scoped_to_current_instance",
            "-prop_thru_buffers",
        ],
        (Dialect::Xdc, "get_cells" | "get_pins") => &[
            "-hierarchical",
            "-hsc",
            "-filter",
            "-of_objects",
            "-match_style",
            "-include_replicated_objects",
        ],
        (Dialect::Xdc, "get_nets") => &[
            "-hierarchical",
            "-hsc",
            "-filter",
            "-of_objects",
            "-match_style",
            "-segments",
            "-top_net_of_hierarchical_group",
        ],
        (Dialect::Xdc, "get_clocks") => &["-filter", "-of_objects", "-include_generated_clocks"],
        (Dialect::Xdc, "set_property") => &["-dict"],
        (Dialect::Xdc, "resize_pblock") => {
            &["-add", "-remove", "-replace", "-locs", "-from", "-to"]
        }
        (Dialect::Xdc, "add_cells_to_pblock") => &["-top", "-clear_locs", "-add_primitives"],
        _ => &[],
    }
}
