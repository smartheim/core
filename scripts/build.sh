#!/bin/bash

DEST=$(realpath "target/libdbus")

download() {
    local url="$1"
    local file="$(basename $url)"
    local strip="$3"
    : "${strip:=1}"
    trap "rm $file" EXIT
    ensure wget --no-check-certificate -q --show-progress "$url" -O "$file"
    mkdir -p "$2"
    if [ "$4" = "zip" ]; then
        ensure unzip -q "$file" -d "$2"
    else
        ensure tar xaf "$file" --strip-components=$strip -C "$2"
    fi
    ensure rm "$file"
    trap "" EXIT
}

prerequirements() {
    need_cmd wget
    need_cmd tput
    need_cmd tar

    mkdir -p $DEST

    if [ ! -d $DEST/x86_64 ]; then
        say "Download x86_64 musl compiler toolchain"
        download https://musl.cc/x86_64-linux-musl-native.tgz $DEST/x86_64 2
    fi

    if [ ! -d $DEST/armv7l ]; then
        say "Download armv7l musl cross compiler toolchain"
        download https://musl.cc/armv7l-linux-musleabihf-cross.tgz $DEST/armv7l 2
    fi

    if [ ! -d $DEST/aarch64 ]; then
        say "Download aarch64 musl cross compiler toolchain"
        download https://musl.cc/aarch64-linux-musl-cross.tgz $DEST/aarch64 2
    fi
}

compile_crate() {
    local ARCH="$1"
    local TARGET="$2"
    local DEST_ARCH="$DEST/$ARCH"
    export PATH=$PATH:$DEST_ARCH/bin
    export PKG_CONFIG_PATH="$DEST_ARCH"
    export PKG_CONFIG_LIBDIR="$DEST_ARCH/lib"
    say "Build crate for $ARCH"
    ensure cargo build --release --target $TARGET
    local METADATA=$(cargo metadata --format-version 1 | jq -r '.workspace_members[]' | tail -n1)
    CRATE_NAME=$(echo $METADATA | cut -d' ' -f1)
    CRATE_VERSION=$(echo $METADATA | cut -d' ' -f2)
    local BINFILE="target/$TARGET/release/$CRATE_NAME"
    say "Before stripping $CRATE_NAME ($CRATE_VERSION): $(wc -c $BINFILE | cut -d' ' -f1) Bytes"
    if [ "$ARCH" = "x86_64" ]; then
        $DEST_ARCH/bin/strip $BINFILE
    else
        local compiler_variant=$(ls $DEST_ARCH/lib/gcc)
        $DEST_ARCH/bin/${compiler_variant}-strip $BINFILE
    fi
    say "After stripping: $(wc -c $BINFILE | cut -d' ' -f1) Bytes"
    mkdir -p $DEST/docker_root
    touch $DEST/docker_root/.empty
    local BINFILE_REL=$(realpath --relative-to="$DEST" "$BINFILE")
printf "
FROM scratch
COPY $BINFILE_REL /bin
ENTRYPOINT [\"/bin\"]
" > $DEST/../Dockerfile_$ARCH

}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "need '$1' (command not found) $2"
    fi
}

ensure() {
    "$@"
    if [ $? != 0 ]; then
        err "ERROR: command failed: $*";
    fi
}

say() {
	local color=$( tput setaf 2 )
	local normal=$( tput sgr0 )
	echo "${color}$1${normal}"
}

err() {
	local color=$( tput setaf 1 )
	local normal=$( tput sgr0 )
	echo "${color}$1${normal}" >&2
	exit 1
}

prerequirements
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_ALL_STATIC=1

export CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc
export CC_armv7_unknown_linux_musleabihf=armv7l-linux-musleabihf-gcc
export LDFLAGS_aarch64_unknown_linux_musl="-lgcc"
export CFLAGS_aarch64_unknown_linux_musl="-lgcc"
export CFLAGS_armv7_unknown_linux_musleabihf="-mfpu=vfpv3-d16"

rustup target add armv7-unknown-linux-musleabihf
rustup target add aarch64-unknown-linux-musl

compile_crate "x86_64" "x86_64-unknown-linux-musl"
compile_crate "aarch64" "aarch64-unknown-linux-musl"
compile_crate "armv7l" "armv7-unknown-linux-musleabihf"

exit 0