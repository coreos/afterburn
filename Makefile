DESTDIR ?=
PREFIX ?= /usr
RELEASE ?= 1
DEFAULT_INSTANCE ?= core

ifeq ($(RELEASE),1)
        PROFILE ?= release
        CARGO_ARGS = --release
else
        PROFILE ?= debug
        CARGO_ARGS =
endif

units = $(addprefix systemd/, \
	afterburn-checkin.service \
	afterburn-firstboot-checkin.service \
	afterburn.service \
	afterburn-sshkeys@.service)

.PHONY: all
all: $(units)
	cargo build ${CARGO_ARGS}

%.service: %.service.in
	sed -e 's,@DEFAULT_INSTANCE@,'$(DEFAULT_INSTANCE)',' < $< > $@.tmp && mv $@.tmp $@

.PHONY: install-units
install-units: $(units)
	for unit in $(units); do install -D -m 644 --target-directory=$(DESTDIR)$(PREFIX)/lib/systemd/system/ $$unit; done

.PHONY: install
install: install-units
	install -D -m 644 -t ${DESTDIR}$(PREFIX)/lib/dracut/modules.d/30afterburn dracut/30afterburn/*
	install -D -t ${DESTDIR}$(PREFIX)/bin target/${PROFILE}/afterburn
