.PHONY: help publish clean release-patch release-minor release-major storybook

# Default target
help:
	@echo "Available targets:"
	@echo "  publish           - Publish all crates to crates.io"
	@echo "  release-patch     - Patch version bump (0.1.0 -> 0.1.1) with git operations [dry-run by default]"
	@echo "  release-minor     - Minor version bump (0.1.0 -> 0.2.0) with git operations [dry-run by default]"
	@echo "  release-major     - Major version bump (0.1.0 -> 1.0.0) with git operations [dry-run by default]"
	@echo "  clean             - Clean all build artifacts"
	@echo "  storybook         - Run storybook (egui-cha-ds component showcase)"
	@echo ""
	@echo "Release notes:"
	@echo "  - Run with EXECUTE=yes to actually perform the release (e.g., make release-patch EXECUTE=yes)"
	@echo "  - By default, releases run in dry-run mode for safety"
	@echo "  - Releases include git commit, tag creation, push, and crates.io publish"

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

# Run storybook (egui-cha-ds component showcase)
storybook:
	cargo run -p egui-cha-ds --example storybook --all-features

# Version bumps with git operations (dry-run by default)
# Usage: make release-patch EXECUTE=yes (to actually execute)

# Patch version bump (0.1.0 -> 0.1.1)
release-patch:
ifdef EXECUTE
	@echo "🚀 Executing patch release..."
	cargo release patch --workspace --execute
else
	@echo "🔍 Running in dry-run mode..."
	@echo "💡 To execute for real, run: make release-patch EXECUTE=yes"
	@echo ""
	cargo release patch --workspace
endif

# Minor version bump (0.1.0 -> 0.2.0)
release-minor:
ifdef EXECUTE
	@echo "🚀 Executing minor release..."
	cargo release minor --workspace --execute
else
	@echo "🔍 Running in dry-run mode..."
	@echo "💡 To execute for real, run: make release-minor EXECUTE=yes"
	@echo ""
	cargo release minor --workspace
endif

# Major version bump (0.1.0 -> 1.0.0)
release-major:
ifdef EXECUTE
	@echo "🚀 Executing major release..."
	cargo release major --workspace --execute
else
	@echo "🔍 Running in dry-run mode..."
	@echo "💡 To execute for real, run: make release-major EXECUTE=yes"
	@echo ""
	cargo release major --workspace
endif
