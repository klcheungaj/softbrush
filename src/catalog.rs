//! Command catalogs used for completion, hover, and dialect-aware linting.

/// A Tcl-based language variant selected from a document path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dialect {
    /// Standard Tcl source.
    Tcl,
    /// Synopsys Design Constraints source.
    Sdc,
    /// AMD Vivado Design Constraints source.
    Xdc,
}

impl Dialect {
    /// Infers the dialect from a URI path extension.
    ///
    /// Unknown and missing extensions intentionally fall back to Tcl.
    #[must_use]
    pub fn from_uri_path(path: &str) -> Self {
        match path
            .rsplit('.')
            .next()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("sdc") => Self::Sdc,
            Some("xdc") => Self::Xdc,
            _ => Self::Tcl,
        }
    }
}

/// Tcl commands that receive keyword semantic highlighting.
pub const TCL_KEYWORDS: &[&str] = &[
    "break",
    "catch",
    "continue",
    "error",
    "eval",
    "expr",
    "for",
    "foreach",
    "if",
    "namespace",
    "proc",
    "return",
    "switch",
    "tailcall",
    "throw",
    "try",
    "uplevel",
    "while",
];

/// Common Tcl 8.6 commands offered by completion and command linting.
pub const TCL_COMMANDS: &[&str] = &[
    "after",
    "append",
    "apply",
    "array",
    "binary",
    "break",
    "catch",
    "cd",
    "chan",
    "clock",
    "close",
    "concat",
    "continue",
    "coroutine",
    "dict",
    "encoding",
    "eof",
    "error",
    "eval",
    "exec",
    "exit",
    "expr",
    "fblocked",
    "fconfigure",
    "fcopy",
    "file",
    "fileevent",
    "flush",
    "for",
    "foreach",
    "format",
    "gets",
    "glob",
    "global",
    "history",
    "if",
    "incr",
    "info",
    "interp",
    "join",
    "lappend",
    "lassign",
    "lindex",
    "linsert",
    "list",
    "llength",
    "lmap",
    "load",
    "lrange",
    "lrepeat",
    "lreplace",
    "lreverse",
    "lsearch",
    "lset",
    "lsort",
    "namespace",
    "open",
    "package",
    "pid",
    "proc",
    "puts",
    "pwd",
    "read",
    "regexp",
    "regsub",
    "rename",
    "return",
    "scan",
    "seek",
    "set",
    "socket",
    "source",
    "split",
    "string",
    "subst",
    "switch",
    "tailcall",
    "tell",
    "time",
    "trace",
    "try",
    "unknown",
    "unload",
    "unset",
    "update",
    "uplevel",
    "upvar",
    "variable",
    "vwait",
    "while",
    "yield",
    "yieldto",
    "zlib",
];

/// Common SDC commands; this intentionally remains a tolerant subset.
pub const SDC_COMMANDS: &[&str] = &[
    "all_clocks",
    "all_inputs",
    "all_outputs",
    "all_registers",
    "create_clock",
    "create_generated_clock",
    "current_design",
    "current_instance",
    "derive_clock_uncertainty",
    "derive_clocks",
    "get_cells",
    "get_clocks",
    "get_keepers",
    "get_lib_cells",
    "get_lib_pins",
    "get_libs",
    "get_nets",
    "get_pins",
    "get_ports",
    "group_path",
    "set_bus_skew",
    "set_case_analysis",
    "set_clock_groups",
    "set_clock_latency",
    "set_clock_sense",
    "set_clock_transition",
    "set_clock_uncertainty",
    "set_data_check",
    "set_disable_timing",
    "set_false_path",
    "set_hierarchy_separator",
    "set_input_delay",
    "set_input_transition",
    "set_load",
    "set_logic_dc",
    "set_logic_one",
    "set_logic_zero",
    "set_max_capacitance",
    "set_max_delay",
    "set_max_fanout",
    "set_max_skew",
    "set_max_time_borrow",
    "set_max_transition",
    "set_min_capacitance",
    "set_min_delay",
    "set_multicycle_path",
    "set_net_delay",
    "set_operating_conditions",
    "set_output_delay",
    "set_propagated_clock",
    "set_units",
];

/// Common XDC commands used by Vivado design constraints.
pub const XDC_COMMANDS: &[&str] = &[
    "add_cells_to_pblock",
    "add_to_power_rail",
    "all_clocks",
    "all_cpus",
    "all_dsps",
    "all_fanin",
    "all_fanout",
    "all_ffs",
    "all_hsios",
    "all_inputs",
    "all_latches",
    "all_outputs",
    "all_rams",
    "all_registers",
    "connect_debug_cores",
    "connect_debug_port",
    "create_clock",
    "create_debug_core",
    "create_debug_port",
    "create_generated_clock",
    "create_macro",
    "create_noc_connection",
    "create_noc_interface",
    "create_pblock",
    "create_power_rail",
    "create_property",
    "create_waiver",
    "current_design",
    "current_instance",
    "delete_macros",
    "delete_noc_connection",
    "delete_noc_interface",
    "delete_pblock",
    "delete_pblocks",
    "delete_power_rails",
    "endgroup",
    "filter",
    "get_bel_pins",
    "get_bels",
    "get_cells",
    "get_clocks",
    "get_debug_cores",
    "get_debug_ports",
    "get_generated_clocks",
    "get_hierarchy_separator",
    "get_iobanks",
    "get_lib_cells",
    "get_lib_pins",
    "get_libs",
    "get_macros",
    "get_nets",
    "get_noc_connections",
    "get_noc_interfaces",
    "get_nodes",
    "get_package_pins",
    "get_path_groups",
    "get_pblocks",
    "get_pins",
    "get_pips",
    "get_pkgpin_bytegroups",
    "get_pkgpin_nibbles",
    "get_ports",
    "get_power_rails",
    "get_property",
    "get_site_pins",
    "get_site_pips",
    "get_sites",
    "get_slrs",
    "get_speed_models",
    "get_tiles",
    "get_timing_arcs",
    "get_wires",
    "group_path",
    "make_diff_pair_ports",
    "move_pblock",
    "remove_cells_from_pblock",
    "remove_from_power_rail",
    "reset_operating_conditions",
    "reset_switching_activity",
    "resize_pblock",
    "set_bus_skew",
    "set_case_analysis",
    "set_clock_groups",
    "set_clock_latency",
    "set_clock_sense",
    "set_clock_uncertainty",
    "set_data_check",
    "set_disable_timing",
    "set_external_delay",
    "set_false_path",
    "set_hierarchy_separator",
    "set_input_delay",
    "set_input_jitter",
    "set_load",
    "set_logic_dc",
    "set_logic_one",
    "set_logic_unconnected",
    "set_logic_zero",
    "set_max_delay",
    "set_max_time_borrow",
    "set_min_delay",
    "set_multicycle_path",
    "set_operating_conditions",
    "set_output_delay",
    "set_package_pin_val",
    "set_power_opt",
    "set_propagated_clock",
    "set_property",
    "set_switching_activity",
    "set_system_jitter",
    "set_units",
    "startgroup",
    "update_macro",
];

/// Reports whether an option accepts a clock name in common SDC/XDC dialects.
///
/// Endpoint options are included only on commands where the reference manuals
/// permit clocks. A matched name is still treated as a reference only when a
/// preceding clock declaration resolves it.
pub(crate) fn option_accepts_clock_name(command: &str, option: &str) -> bool {
    if matches!(
        option,
        "-clock"
            | "-clocks"
            | "-master_clock"
            | "-from_clock"
            | "-rise_from_clock"
            | "-fall_from_clock"
            | "-to_clock"
            | "-rise_to_clock"
            | "-fall_to_clock"
            | "-rise_clock"
            | "-fall_clock"
    ) {
        return true;
    }

    if option == "-group" {
        return command == "set_clock_groups";
    }

    matches!(
        command,
        "group_path"
            | "set_bus_skew"
            | "set_clock_uncertainty"
            | "set_false_path"
            | "set_max_delay"
            | "set_min_delay"
            | "set_multicycle_path"
    ) && matches!(
        option,
        "-from"
            | "-rise_from"
            | "-fall_from"
            | "-to"
            | "-rise_to"
            | "-fall_to"
            | "-through"
            | "-rise_through"
            | "-fall_through"
    )
}

/// Reports whether a command's positional arguments may name clocks.
pub(crate) fn positional_arguments_accept_clock_names(command: &str) -> bool {
    matches!(
        command,
        "get_clocks"
            | "set_clock_latency"
            | "set_clock_uncertainty"
            | "set_input_jitter"
            | "set_propagated_clock"
    )
}

/// Iterates over Tcl commands and commands specific to `dialect`.
pub fn commands(dialect: Dialect) -> impl Iterator<Item = &'static str> {
    let dialect_commands = match dialect {
        Dialect::Tcl => &[][..],
        Dialect::Sdc => SDC_COMMANDS,
        Dialect::Xdc => XDC_COMMANDS,
    };
    TCL_COMMANDS.iter().chain(dialect_commands).copied()
}

/// Reports whether `name` belongs to the known tolerant command catalog.
#[must_use]
pub fn is_known_command(dialect: Dialect, name: &str) -> bool {
    commands(dialect).any(|command| command == name)
}

/// Returns concise hover documentation for selected common commands.
#[must_use]
pub fn command_summary(name: &str) -> Option<&'static str> {
    Some(match name {
        "proc" => "Define a Tcl procedure: `proc name arguments body`.",
        "set" => "Read or assign a Tcl variable: `set varName ?newValue?`.",
        "create_clock" => "Create a primary or virtual timing clock.",
        "create_generated_clock" => "Create a clock derived from another clock source.",
        "set_input_delay" => "Specify an input path delay relative to a clock.",
        "set_output_delay" => "Specify an output path delay relative to a clock.",
        "set_false_path" => "Declare selected timing paths as false paths.",
        "set_multicycle_path" => "Change the setup or hold cycle count for selected paths.",
        "set_property" => "Set an XDC property on one or more design objects.",
        "get_cells" => "Return a collection of cells matching the query.",
        "get_pins" => "Return a collection of pins matching the query.",
        "get_ports" => "Return a collection of top-level ports matching the query.",
        "create_pblock" => "Create an XDC placement block.",
        _ => return None,
    })
}
