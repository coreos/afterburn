#!/bin/bash
# -*- mode: shell-script; indent-tabs-mode: nil; sh-basic-offset: 4; -*-
# ex: ts=8 sw=4 sts=4 et filetype=sh

check() {
    # Return 255 so this module is only included if some other module depends on it
    # See: https://github.com/coreos/fedora-coreos-tracker/issues/1832
    #
    # This module requires integration with the rest of the initramfs, so don't include it by default.
    return 255
}

depends() {
    echo systemd network
}

install() {
    inst_multiple afterburn

    inst_simple "$moddir/afterburn-hostname.service" \
        "$systemdutildir/system/afterburn-hostname.service"

    inst_simple "$moddir/afterburn-network-kargs.service" \
        "$systemdutildir/system/afterburn-network-kargs.service"

    # These services are only run once on first-boot, so they piggyback
    # on Ignition completion target.
    mkdir -p "$initdir/$systemdsystemunitdir/ignition-complete.target.requires"
    ln -s "../afterburn-hostname.service" "$initdir/$systemdsystemunitdir/ignition-complete.target.requires/afterburn-hostname.service"
    ln -s "../afterburn-network-kargs.service" "$initdir/$systemdsystemunitdir/ignition-complete.target.requires/afterburn-network-kargs.service"
}
