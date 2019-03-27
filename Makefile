DESTDIR ?=
PREFIX ?= /usr

DEFAULT_INSTANCE ?= core

units = $(addprefix systemd/,afterburn.service afterburn-sshkeys@.service)

%.service: %.service.in
	sed -e 's,@DEFAULT_INSTANCE@,'$(DEFAULT_INSTANCE)',' < $< > $@.tmp && mv $@.tmp $@

all:
	cargo build --release

install-units: $(units)
	for unit in $(units); do install -D --target-directory=$(DESTDIR)$(PREFIX)/lib/systemd/system/ $$unit; done
