#!/bin/bash
#
# http://wiki.openssl.org/index.php/Android
#
set -e

output_dir="cross/"

mkdir -p $output_dir

# Clean openssl:
cd openssl
git clean -dfx && git checkout -f
cd -

archs=(armeabi-v7a arm64-v8a mips mips64 x86 x86_64)

openssl_config_options=$(cat openssl-build-scripts/openssl-config-params.txt)

for arch in ${archs[@]}; do
    xLIB="/lib"
	unset _ANDROID_INCLUDE_TRIPLE
    case ${arch} in
        "armeabi-v7a")
            _ANDROID_API="android-19"
            _ANDROID_TARGET_SELECT=arch-arm
            _ANDROID_ARCH=arch-arm
			_ANDROID_TRIPLE=armv7-linux-androideabi
			_ANDROID_INCLUDE_TRIPLE=arm-linux-androideabi
            _ANDROID_EABI=arm-linux-androideabi-4.9
            configure_platform="android-armeabi" ;;
        "arm64-v8a")
            _ANDROID_API="android-21"
            _ANDROID_TARGET_SELECT=arch-arm64-v8a
            _ANDROID_ARCH=arch-arm64
            _ANDROID_TRIPLE=aarch64-linux-android
            _ANDROID_EABI=$_ANDROID_TRIPLE-4.9
            #no xLIB="/lib64"
            configure_platform="android64-aarch64" ;;
        "mips")
            _ANDROID_API="android-19"
            _ANDROID_TARGET_SELECT=arch-mips
            _ANDROID_ARCH=arch-mips
            _ANDROID_TRIPLE=mipsel-linux-android
            _ANDROID_EABI=$_ANDROID_TRIPLE-4.9
            configure_platform="android -DB_ENDIAN" ;;
        "mips64")
            _ANDROID_API="android-21"
            _ANDROID_TARGET_SELECT=arch-mips64
            _ANDROID_ARCH=arch-mips64
            _ANDROID_TRIPLE=mips64el-linux-android
            _ANDROID_EABI=$_ANDROID_TRIPLE-4.9
            xLIB="/lib64"
            configure_platform="linux-generic64 -DB_ENDIAN" ;;
        "x86")
            _ANDROID_API="android-19"
            _ANDROID_TARGET_SELECT=arch-x86
            _ANDROID_ARCH=arch-x86
            _ANDROID_TRIPLE=i686-linux-android
            _ANDROID_EABI=x86-4.9
            configure_platform="android-x86" ;;
        "x86_64")
            _ANDROID_API="android-21"
            _ANDROID_TARGET_SELECT=arch-x86_64
            _ANDROID_ARCH=arch-x86_64
            _ANDROID_TRIPLE=x86_64-linux-android
            _ANDROID_EABI=x86_64-4.9
            xLIB="/lib64"
            configure_platform="linux-generic64" ;;
        *)
            configure_platform="linux-elf" ;;
    esac

    mkdir -p "$output_dir/${_ANDROID_TRIPLE}/lib"
    mkdir -p "$output_dir/${_ANDROID_TRIPLE}/include"

    . ./openssl-build-scripts/build-android-setenv.sh

    echo "CROSS COMPILE ENV : $CROSS_COMPILE"
    cd openssl

	_ANDROID_INCLUDE_TRIPLE=${_ANDROID_INCLUDE_TRIPLE:-$_ANDROID_TRIPLE}

    xCFLAGS="-fPIC -I$ANDROID_DEV/include -B$ANDROID_DEV/$xLIB -I$ANDROID_NDK_ROOT/sysroot/usr/include -I$ANDROID_NDK_ROOT/sysroot/usr/include/$_ANDROID_INCLUDE_TRIPLE -DOPENSSL_NO_SSL3 -DOPENSSL_NO_SSL2 -DOPENSSL_NO_SSL3_METHOD"


	./Configure dist
	./Configure $openssl_config_options --openssldir=/tmp/openssl_android/ $configure_platform $xCFLAGS

    #perl -pi -e 's/install: all install_docs install_sw/install: install_docs install_sw/g' Makefile.org

    
	make clean
    make -j depend
	#make -j build_crypto
	#make -j build_ssl
	#make -j all
	make -j build_generated
	make libcrypto.a libssl.a

    file libcrypto.a
    cp libcrypto.a ../$output_dir/${_ANDROID_TRIPLE}/lib/libcrypto.a

    file libssl.a
    cp libssl.a ../$output_dir/${_ANDROID_TRIPLE}/lib/libssl.a

	cp -r include/* ../$output_dir/${_ANDROID_TRIPLE}/include

    # Cleanup:
    git clean -dfx && git checkout -f

    cd -
done
exit 0
