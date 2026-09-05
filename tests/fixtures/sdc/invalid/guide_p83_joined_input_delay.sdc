# Apparent source defect from p. 83: two commands are joined by continuation.
set_input_delay -clock {clockone_ext} -max 4 [get_ports {data1}]\
    set_input_delay -clock {clockone_ext} -min -1 [get_ports {data1}]

