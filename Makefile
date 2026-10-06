.PHONY: run-app

run-app:
	@echo "Running iOS..."
	cargo build -p tetrad-lib --lib --release
	swift run --package-path src/client/ios \
		-Xlinker -L -Xlinker "$(CURDIR)/target/release" \
		-Xlinker -rpath -Xlinker "$(CURDIR)/target/release" \
		TetradUI
