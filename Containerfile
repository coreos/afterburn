ARG BASE
ARG LABEL
ARG TOOLBOX
FROM $TOOLBOX as build

RUN dnf install -y rust cargo openssl-devel make

COPY . /afterburn/

# Use a cache mount for the cargo registry
RUN  make -C afterburn

FROM $BASE
COPY --from=build /afterburn/target/release/afterburn /usr/bin/afterburn

RUN set -xeuo pipefail && \
    KERNEL_VERSION="$(basename $(ls -d /lib/modules/*))" && \
    raw_args="$(lsinitrd /lib/modules/${KERNEL_VERSION}/initramfs.img | grep '^Arguments: ' | sed 's/^Arguments: //')" && \
    stock_arguments=$(echo "$raw_args" | sed "s/'//g") && \
    echo "Using kernel: $KERNEL_VERSION" && \
    echo "Dracut arguments: $stock_arguments" && \
    mkdir -p /tmp/dracut /var/roothome && \
    dracut $stock_arguments && \
    mv -v /boot/initramfs*.img "/lib/modules/${KERNEL_VERSION}/initramfs.img" && \
    ostree container commit

LABEL com.coreos.osname=$LABEL
