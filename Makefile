all: copy
VERSION := 0.1.0

copy:
	cargo build

	copy .\target\debug\tmpl.exe .\

build:
	cargo build --release

	copy .\target\release\tmpl.exe .\release\installer\tmpl.exe

msi: build
	wix build release\installer\product.wxs -o release\tmpl-cli-$(VERSION).msi

	del /F /Q release\tmpl-cli-$(VERSION).wixpdb