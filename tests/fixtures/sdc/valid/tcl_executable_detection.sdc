set current_exe $::TimingAnalyzerInfo(nameofexecutable)
if { [string equal $current_exe "quartus_fit"] } {
    set_multicycle_path -setup -from src_reg* -to dst_reg* 2
} else {
    set_multicycle_path -setup -from src_reg* -to dst_reg* 3
}

