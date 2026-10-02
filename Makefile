.PHONY: run-app

run-app:
	@echo "Running iOS..."
	cargo build -p tetrad-lb --lib --release
	swift run --package-path src/ios \
		-Xlinker -L -Xlinker "$(CURDIR)/target/release" \
		-Xlinker -rpath -Xlinker "$(CURDIR)/target/release" \
		TetradUI
