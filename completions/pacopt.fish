# -*- mode: fish; -*-

set --local progname pacopt
set --local listinstalled "(pacman -Q | string replace ' ' \t)"

complete --command $progname --short-option i --no-files --long-option installed --description "Show installed packages"
complete --command $progname --short-option u --no-files --long-option uninstalled --description "Show uninstalled packages"
complete --command $progname --short-option n --no-files --long-option name-only --description "Show package name only"
complete --command $progname --short-option x --no-files --long-option xargs --description "Create argument list"
complete --command $progname --long-option json --no-files --description "Output to JSON format without filter"
complete --command $progname --short-option h --no-files --long-option help --description "Print help"
complete --command $progname --short-option V --no-files --long-option version --description "Print version"

complete --command $progname --no-files --exclusive --arguments "$listinstalled" --description 'Installed package'
