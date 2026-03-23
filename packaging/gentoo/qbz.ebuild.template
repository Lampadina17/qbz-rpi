# Copyright 2024-2026 Gentoo Authors
# Distributed under the terms of the MIT License

EAPI=8

inherit cargo xdg

DESCRIPTION="Native hi-fi Qobuz desktop player for Linux"
HOMEPAGE="https://qbz.lol https://github.com/vicrodh/qbz"

MY_PV="${PV}"
SRC_URI="
	https://github.com/vicrodh/qbz/archive/refs/tags/v${MY_PV}.tar.gz -> ${P}.tar.gz
	${CARGO_CRATE_URIS}
"

LICENSE="MIT"
SLOT="0"
KEYWORDS="~amd64 ~arm64"
IUSE="pipewire pulseaudio"

RDEPEND="
	net-libs/webkit-gtk:4.1
	x11-libs/gtk+:3
	media-libs/alsa-lib
	dev-libs/openssl
	dev-libs/libappindicator:3
	x11-libs/libxkbcommon
	gnome-base/librsvg:2
	pipewire? (
		media-video/pipewire[alsa-plugin]
	)
	pulseaudio? (
		media-libs/libpulse
	)
"

BDEPEND="
	virtual/rust
	net-libs/nodejs[npm]
	sys-devel/clang
	virtual/pkgconfig
"

# alsa-utils improves device detection for bit-perfect playback
PDEPEND="
	media-sound/alsa-utils
"

S="${WORKDIR}/qbz-${MY_PV}"

src_prepare() {
	default

	# Generate cargo config for offline build if needed
	cargo_src_prepare
}

src_configure() {
	cargo_src_configure
}

src_compile() {
	# Build the SvelteKit frontend
	npm ci --prefer-offline || die "npm ci failed"
	npm run build || die "frontend build failed"

	# Build the Tauri/Rust backend (no bundler — we install manually)
	cd src-tauri || die
	cargo_src_compile --bin qbz
}

src_install() {
	# Install the binary
	dobin "src-tauri/target/release/qbz"

	# Desktop file
	insinto /usr/share/applications
	newins packaging/arch/qbz.desktop qbz.desktop

	# Icons
	local size
	for size in 32 48 64 128 256; do
		local src_icon="src-tauri/icons/${size}x${size}.png"
		if [[ -f "${src_icon}" ]]; then
			insinto "/usr/share/icons/hicolor/${size}x${size}/apps"
			newins "${src_icon}" qbz.png
		fi
	done

	# Use the high-res icon as a fallback for missing sizes
	local fallback="src-tauri/icons/128x128.png"
	for size in 48 64; do
		if [[ ! -f "src-tauri/icons/${size}x${size}.png" ]]; then
			insinto "/usr/share/icons/hicolor/${size}x${size}/apps"
			newins "${fallback}" qbz.png
		fi
	done
}

pkg_postinst() {
	xdg_icon_cache_update
	xdg_desktop_database_update

	elog "For bit-perfect audio playback, ensure your user is in the 'audio' group:"
	elog "  usermod -aG audio \${USER}"
	elog ""
	elog "Optional: install media-sound/alsa-utils for better ALSA device detection."
}

pkg_postrm() {
	xdg_icon_cache_update
	xdg_desktop_database_update
}
