UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
LINUX_PAIR=linux-$(shell uname -m)

cross/$(LINUX_PAIR)/lib/libsodium.a:
	cd libsodium && ./configure --prefix=${PWD}/cross/$(LINUX_PAIR) && make clean && make && make install

cross/$(LINUX_PAIR)/lib/libssl.a:
	cd openssl && ./config -fPIC --prefix=${PWD}/cross/$(LINUX_PAIR) && make clean && make --quiet -j depend && make --quiet -j build_generated && make --quiet libcrypto.a libssl.a && make --quiet install

OPENSSL_DEP=cross/$(LINUX_PAIR)/lib/libssl.a
OPENSSL_FLAGS=OPENSSL_DIR=${PWD}/cross/$(LINUX_PAIR) OPENSSL_STATIC=1

SODIUM_DEP=cross/$(LINUX_PAIR)/lib/libsodium.a
SODIUM_FLAGS=SODIUM_STATIC=1 SODIUM_LIB_DIR=${PWD}/cross/$(LINUX_PAIR)/lib
endif

ifeq ($(UNAME_S),Darwin)
SODIUM_FLAGS=SODIUM_STATIC=1 SODIUM_LIB_DIR="`PKG_CONFIG_ALLOW_SYSTEM_LIBS=1 pkg-config --static libsodium --libs-only-L | tail -c +3`" 
endif

libsigchain-with-dashboard: $(SODIUM_DEP) $(OPENSSL_DEP)
	if [ "`cargo web --version`" != "cargo-web 0.6.10" ]; then echo "Please install cargo-web 0.6.10 with \`cargo install -f --version 0.6.10 cargo-web\`"; exit 1; fi
	# use rsync so that file modifed times are only updated when the contents change
	cd dashboard_yew && cargo web deploy --release --target=wasm32-unknown-emscripten && rsync --checksum --delete -r ../target/deploy/* ../target/deploy-final
	cd libsigchain && $(SODIUM_FLAGS) $(OPENSSL_FLAGS) cargo build ${CARGO_RELEASE}

check-libsigchain-with-dashboard: libsigchain-with-dashboard
	DATABASE_URL=test.db $(SODIUM_FLAGS) $(OPENSSL_FLAGS) cargo test ${CARGO_RELEASE} -- --test-threads=1

AARCH64_LINUX_ANDROID_PREFIX=${ANDROID_NDK}/arm64/bin/aarch64-linux-android-
ANDROID_AARCH64_ENV=LDFLAGS="-ldl ${LDFLAGS}" CXX=${AARCH64_LINUX_ANDROID_PREFIX}g++ CC=${AARCH64_LINUX_ANDROID_PREFIX}gcc AR=${AARCH64_LINUX_ANDROID_PREFIX}ar STRIP=${AARCH64_LINUX_ANDROID_PREFIX}strip NM=${AARCH64_LINUX_ANDROID_PREFIX}nm RANLIB=${AARCH64_LINUX_ANDROID_PREFIX}ranlib CCLD=${AARCH64_LINUX_ANDROID_PREFIX}gcc _ANDROID_EABI=aarch64-linux-android-4.9 _ANDROID_ARCH=aarch64 _ANDROID_API=26 INCLUDE_PATH="" CPP_INCLUDE_PATH="" 
ANDROID_AARCH64_CARGO_FLAGS=CC_aarch64_linux_android=${AARCH64_LINUX_ANDROID_PREFIX}gcc AR_aarch64_linux_android=${AARCH64_LINUX_ANDROID_PREFIX}ar

cross/aarch64-linux-android/lib/libssl.a:
	./openssl-build-scripts/build-android.sh

cross/aarch64-linux-android/lib/libsodium.so: 
	cd libsodium && (make clean || true) && USE_DEV_URANDOM=1 $(ANDROID_AARCH64_ENV) ./configure --prefix=${PWD}/cross/aarch64-linux-android --host=aarch64-linux-android
	cd libsodium && $(ANDROID_AARCH64_ENV) make -j && $(ANDROID_AARCH64_ENV) make install

aarch64-linux-android: | cross/aarch64-linux-android/lib/libsodium.so cross/aarch64-linux-android/lib/libssl.a android-smoke
	cd sigchain_client && $(ANDROID_AARCH64_CARGO_FLAGS) TARGET=aarch64-linux-android OPENSSL_DIR=${PWD}/cross/aarch64-linux-android OPENSSL_STATIC=1 SODIUM_LIB_DIR=cross/aarch64-linux-android/lib PKG_CONFIG_ALLOW_CROSS=1 cargo build ${CARGO_RELEASE} --target=aarch64-linux-android --no-default-features --features="android_client" --verbose

ARMV7_LINUX_ANDROIDEABI_PREFIX=${ANDROID_NDK}/arm/bin/arm-linux-androideabi-
ARMV7_LINUX_ANDROIDEABI_ENV=LDFLAGS="-ldl ${LDFLAGS}" CXX=${ARMV7_LINUX_ANDROIDEABI_PREFIX}g++ CC=${ARMV7_LINUX_ANDROIDEABI_PREFIX}gcc AR=${ARMV7_LINUX_ANDROIDEABI_PREFIX}ar STRIP=${ARMV7_LINUX_ANDROIDEABI_PREFIX}strip NM=${ARMV7_LINUX_ANDROIDEABI_PREFIX}nm RANLIB=${ARMV7_LINUX_ANDROIDEABI_PREFIX}ranlib CCLD=${ARMV7_LINUX_ANDROIDEABI_PREFIX}gcc _ANDROID_EABI=aarch64-linux-android-4.9 INCLUDE_PATH="" CPP_INCLUDE_PATH=""
ARMV7_LINUX_ANDROIDEABI_CARGO_FLAGS=CC_armv7_linux_androideabi=${ARMV7_LINUX_ANDROIDEABI_PREFIX}gcc AR_armv7_linux_androideabi=${ARMV7_LINUX_ANDROIDEABI_PREFIX}ar


cross/armv7-linux-androideabi/lib/libssl.a:
	./openssl-build-scripts/build-android.sh

cross/armv7-linux-androideabi/lib/libsodium.so: 
	cd libsodium && (make clean || true) && USE_DEV_URANDOM=1 ${ARMV7_LINUX_ANDROIDEABI_ENV} ./configure --prefix=${PWD}/cross/armv7-linux-androideabi --host=armv7-linux-androideabi
	cd libsodium && ${ARMV7_LINUX_ANDROIDEABI_ENV} make -j && ${ARMV7_LINUX_ANDROIDEABI_ENV} make install

armv7-linux-androideabi: | cross/armv7-linux-androideabi/lib/libsodium.so cross/armv7-linux-androideabi/lib/libssl.a android-smoke
	cd sigchain_client && $(ARMV7_LINUX_ANDROIDEABI_CARGO_FLAGS) TARGET=armv7-linux-androideabi OPENSSL_DIR=${PWD}/cross/armv7-linux-androideabi OPENSSL_STATIC=1 SODIUM_LIB_DIR=cross/armv7-linux-androideabi/lib PKG_CONFIG_ALLOW_CROSS=1 ${ARMV7_LINUX_ANDROIDEABI_ENV} cargo build ${CARGO_RELEASE} --target=armv7-linux-androideabi --no-default-features --features="android_client" --verbose

android-smoke:
ifndef ANDROID_NDK
	$(error ANDROID_NDK is undefined)
endif

.PHONY:
