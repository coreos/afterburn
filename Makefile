DESTDIR ?=
PREFIX ?= /usr

DEFAULT_INSTANCE ?= core

units = $(addprefix systemd/, \
	afterburn-checkin.service \
	afterburn-firstboot-checkin.service \
	afterburn.service \
	afterburn-sshkeys@.service)

.PHONY: all
all: $(units)
	cargo build --release

%.service: %.service.in
	sed -e 's,@DEFAULT_INSTANCE@,'$(DEFAULT_INSTANCE)',' < $< > $@.tmp && mv $@.tmp $@

.PHONY: install-units
install-units: $(units)
	for unit in $(units); do install -D --target-directory=$(DESTDIR)$(PREFIX)/lib/systemd/system/ $$unit; done

.PHONY: install
install: install-units
	install -D -m 0444 -t ${DESTDIR}$(PREFIX)/lib/dracut/modules.d/30afterburn dracut/30afterburn/*
	install -D -t ${DESTDIR}$(PREFIX)/bin target/debug/afterburn
