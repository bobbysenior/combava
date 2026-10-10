# Installation de combava pour l'utilisateur courant, sans droits root.
#
#   make install     binaire dans ~/.local/bin ; configuration globale et
#                    template par défaut dans ~/.config/combava
#   make uninstall   retire le binaire, conserve la configuration
#   make build       compile le binaire (target/release/combava)
#   make test        lance tous les tests
#
# Une configuration ou un template déjà installé n'est jamais écrasé : il peut
# avoir été modifié (réglages, vrais logos). `make install FORCE=1` le remplace.
#
# Le template installé remplace celui embarqué dans le binaire (spécification,
# section 7) : c'est là qu'on remplace les logos ou qu'on retouche le style.

PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin

# Dossier de configuration globale lu par combava (crate `directories`).
ifeq ($(shell uname -s),Darwin)
CONFIG_DIR ?= $(HOME)/Library/Application Support/combava
else
XDG_CONFIG_HOME ?= $(HOME)/.config
CONFIG_DIR ?= $(XDG_CONFIG_HOME)/combava
endif

CARGO ?= cargo
FORCE ?=

BIN := target/release/combava
CONFIG := $(CONFIG_DIR)/config.toml
TEMPLATE := $(CONFIG_DIR)/templates/default

.PHONY: all build test install uninstall

all: build

build:
	$(CARGO) build --release --locked -p combava-cli

test:
	$(CARGO) test --workspace

install: build
	@mkdir -p '$(BINDIR)'
	@install -m 755 '$(BIN)' '$(BINDIR)/combava'
	@echo "installé : $(BINDIR)/combava"
	@if [ -e '$(CONFIG)' ] && [ -z '$(FORCE)' ]; then \
		echo "conservé : $(CONFIG) (make install FORCE=1 pour le remplacer)"; \
	else \
		mkdir -p '$(CONFIG_DIR)' && \
		install -m 644 config/config.toml '$(CONFIG)' && \
		echo "installé : $(CONFIG)"; \
	fi
	@if [ -e '$(TEMPLATE)' ] && [ -z '$(FORCE)' ]; then \
		echo "conservé : $(TEMPLATE)/ (make install FORCE=1 pour le remplacer)"; \
	else \
		rm -rf '$(TEMPLATE)' && \
		mkdir -p '$(CONFIG_DIR)/templates' && \
		tar -C templates --exclude='*.pdf' -cf - default | tar -C '$(CONFIG_DIR)/templates' -xf - && \
		echo "installé : $(TEMPLATE)/"; \
	fi
	@case ":$$PATH:" in \
		*':$(BINDIR):'*) ;; \
		*) echo "attention : $(BINDIR) n'est pas dans le PATH" ;; \
	esac

uninstall:
	rm -f '$(BINDIR)/combava'
	@echo "conservé : $(CONFIG_DIR)/ (à supprimer à la main si besoin)"
