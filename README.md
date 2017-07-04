This is intended to replace the unreliable [AkaPi](https://github.com/Dhertz/AkaPi) emailing/texting functionality.

To compile this for pi, you will need OPENSSL libs compiled for arm and a arm gcc compiler ([look in the pi-toolkit](https://github.com/raspberrypi/tools))

Add to your cargo config:

```
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

then run to compile:

`OPENSSL_LIB_DIR=$PIOPENSSL/openssl-1.0.1t/ OPENSSL_INCLUDE_DIR=$PIOPENSSL/openssl-1.0.1t/include/ cargo build --target=armv7-unknown-linux-gnueabihf --release`



Secrets in lib/secrets.rs needed to compile:

```rust
 pub const MY_EMAIL:&str 
 pub const PURPLE_EMAIL:&str 
 pub const TW_ACC_ID:&str 
 pub const TW_NUMBER:&str 
 pub const TW_UID:&str 
 pub const TW_SID:&str 
 pub const TW_KEY:&str 
 pub const SMTP_HOST:&str 
```
