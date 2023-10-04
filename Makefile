TARGET=ff_daemon

all: $(TARGET)

PREFIX ?= $(DESTDIR)/usr/local
BINDIR ?= $(PREFIX)/bin

BUILD ?= release

BUILD_FLAGS=

ifeq ($(BUILD),release)
	BUILD_FLAGS+=--release
endif

DEPS = $(wildcard src/*.rs) Cargo.toml

CARGO=$(HOME)/.cargo/bin/cargo
ifeq (,$(wildcard $(CARGO)))
	CARGO=cargo
endif

target/$(BUILD)/$(TARGET): $(DEPS)
	$(CARGO) build $(BUILD_FLAGS)

$(TARGET): target/$(BUILD)/$(TARGET)
	cp -a $< $@

install: target/$(BUILD)/$(TARGET)
	install -m0755 $< $(BINDIR)/$(TARGET)

uninstall:
	$(RM) $(addprefix $(BINDIR)/,$(TARGET))

test:
	$(CARGO) test $(BUILD_FLAGS) -- --test-threads=1 --nocapture

clean:
	rm -rf target $(TARGET)

.PHONY: all clean install test uninstall