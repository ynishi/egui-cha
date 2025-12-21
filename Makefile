.PHONY: help publish clean

# Default target
help:
	@echo "Available targets:"
	@echo "  publish           - Publish all crates to crates.io"
	@echo "  clean             - Clean all build artifacts"

# Crates in dependency order (least dependent first)
CRATES := \
	crates/egui-cha-macros \
	crates/egui-cha \
	crates/egui-cha-ds \
	crates/egui-cha-analyzer

# Actual publish (requires crates.io authentication)
publish:
	@echo "⚠️  This will publish all crates to crates.io!"
	@echo "⚠️  Press Ctrl+C to cancel, or Enter to continue..."
	@read dummy
	@echo "🚀 Publishing to crates.io..."
	@for crate in $(CRATES); do \
		echo "📤 Publishing $$crate..."; \
		(cd $$crate && cargo publish); \
		echo "⏳ Waiting 30s for crates.io to index..."; \
		sleep 30; \
	done
	@echo "✅ All crates published successfully!"

# Clean build artifacts
clean:
	cargo clean
