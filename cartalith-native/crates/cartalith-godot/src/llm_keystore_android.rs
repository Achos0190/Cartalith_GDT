//! The Android half of the LLM assistant's key store (`LLM_ASSISTANT_SCOPE.md`
//! LLM-10, Ruling DD Q5): the API key is encrypted with an AES-256-GCM key that
//! lives in the **Android Keystore** and never leaves it, and only the
//! ciphertext is written to app-private storage.
//!
//! WHY JNI FROM RUST, NOT A `.aar` PLUGIN: Ruling AV's precedent for Android
//! APIs is a small Godot Android plugin, but a v2 plugin needs the Gradle build
//! (`gradle_build/use_gradle_build`, off in `export_presets.cfg`) and this
//! project has no Gradle tree. The Keystore is reachable through a dozen framework
//! calls whose classes the system class loader finds from any attached thread,
//! so the store is written here against the raw JNI table (`jni-sys`, already in
//! `Cargo.lock`), keeping the plaintext key inside Rust and Java and out of
//! GDScript altogether. Labelled judgement; the owner may overrule.
//!
//! HOW THE JVM IS FOUND: Godot loads this library with `dlopen`, so no
//! `JNI_OnLoad` runs here and Godot exports no handle to its `JavaVM`.
//! `JNI_GetCreatedJavaVMs` is looked up at run time in `libnativehelper.so`,
//! which the NDK lists as a public library from API 31 (Android 12). On an
//! older device the symbol is not reachable from the app's linker namespace,
//! [`available`] is `false`, and the caller refuses with a reason (Ruling DD
//! Q5: no store and no environment variable means refuse, never a file).
//!
//! WHAT IS ON DISK: `<user data dir>/llm_keystore/<account>.bin`, the bytes
//! [`blob::encode`] writes: a 4-byte magic, the GCM nonce the Keystore chose,
//! and the ciphertext with its 16-byte tag. The account id is the GCM
//! additional data, so a file renamed to another account fails to open. The
//! wrapping key's alias is per account ([`alias_for`]); forgetting a key
//! deletes the file **and** the alias, so a stray copy of the file can never be
//! opened again. The Keystore key is created without user authentication (no
//! biometric prompt per call) and is non-exportable by construction.
//!
//! MUST NEVER: write the plaintext anywhere (not a file, not the log, not an
//! exception message, not a Godot object); fall back to storing the key in the
//! clear when the Keystore refuses; call `ExceptionDescribe` (it prints to
//! logcat); leave a Java exception pending on return to Godot (the next JNI
//! call from Godot would abort the process).

// The pure helpers below are used by the Android half only; on every other
// target they exist for their unit tests, which run on the desktop.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

/// The on-disk format, pure and testable on every platform.
pub mod blob {
    /// First four bytes of every saved file: "Cartalith LLM Key, format 1".
    pub const MAGIC: [u8; 4] = *b"CLK1";
    /// The GCM authentication tag length in bytes (`GCMParameterSpec(128, ..)`).
    pub const TAG_BYTES: usize = 16;
    /// The longest nonce accepted on read. The Keystore's AES-GCM nonce is 12
    /// bytes; anything over this bound is a damaged file, not a key.
    pub const MAX_IV_BYTES: usize = 32;

    /// `MAGIC | iv_len (1 byte) | iv | ciphertext-with-tag`.
    pub fn encode(iv: &[u8], ct: &[u8]) -> Option<Vec<u8>> {
        if iv.is_empty() || iv.len() > MAX_IV_BYTES || ct.len() <= TAG_BYTES {
            return None;
        }
        let mut out = Vec::with_capacity(MAGIC.len() + 1 + iv.len() + ct.len());
        out.extend_from_slice(&MAGIC);
        out.push(iv.len() as u8);
        out.extend_from_slice(iv);
        out.extend_from_slice(ct);
        Some(out)
    }

    /// The `(iv, ciphertext)` of a saved file, or `None` for anything that is
    /// not exactly the format `encode` writes (wrong magic, an empty or
    /// over-long nonce, a ciphertext no longer than its tag).
    pub fn decode(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
        let rest = bytes.strip_prefix(&MAGIC[..])?;
        let (&n, rest) = rest.split_first()?;
        let n = n as usize;
        if n == 0 || n > MAX_IV_BYTES || rest.len() <= n + TAG_BYTES {
            return None;
        }
        Some(rest.split_at(n))
    }
}

/// The Keystore alias that wraps `account`'s key. Callers have already
/// checked the account id with `account_ok` (`[a-z0-9_-]`, at most 64).
pub fn alias_for(account: &str) -> String {
    format!("cartalith_llm_{account}")
}

/// The saved file's name inside the store directory.
pub fn file_name_for(account: &str) -> String {
    format!("{account}.bin")
}

/// The directory under Godot's user data dir that holds the ciphertexts.
pub const DIR_NAME: &str = "llm_keystore";

#[cfg(target_os = "android")]
pub use jvm_side::{available, delete, read, write};

#[cfg(target_os = "android")]
mod jvm_side {
    use super::{DIR_NAME, alias_for, blob, file_name_for};
    use cartalith_llm::keystore::StoreError;
    use core::ffi::{c_char, c_int, c_void};
    use jni_sys::{
        JNI_EDETACHED, JNI_OK, JNI_VERSION_1_6, JNIEnv, JavaVM, jbyteArray, jclass, jint, jmethodID, jobject, jsize,
        jvalue,
    };
    use std::ffi::CString;
    use std::sync::OnceLock;

    unsafe extern "C" {
        fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }
    /// bionic's `RTLD_NOW` (`<dlfcn.h>`: `RTLD_NOW 2`).
    const RTLD_NOW: c_int = 2;

    /// `KeyProperties.PURPOSE_ENCRYPT | PURPOSE_DECRYPT` (1 | 2, Android SDK).
    const PURPOSES: jint = 3;
    /// `Cipher.ENCRYPT_MODE` / `Cipher.DECRYPT_MODE` (1 / 2, Java SE).
    const ENCRYPT_MODE: jint = 1;
    const DECRYPT_MODE: jint = 2;
    /// AES key size, bits.
    const KEY_BITS: jint = 256;
    /// GCM tag, bits; must agree with `blob::TAG_BYTES`.
    const TAG_BITS: jint = (blob::TAG_BYTES * 8) as jint;

    type GetVms = unsafe extern "system" fn(*mut *mut JavaVM, jsize, *mut jsize) -> jint;

    /// The process's JVM, looked up once. `None` when `libnativehelper.so`
    /// or its symbol is not reachable (before API 31) or no VM exists.
    fn vm() -> Option<*mut JavaVM> {
        static VM: OnceLock<usize> = OnceLock::new();
        let p = *VM.get_or_init(|| {
            // SAFETY: plain C calls on NUL-terminated literals; a null result
            // is checked before use. The handle is never closed, by design: the
            // library stays mapped for the life of the process anyway.
            unsafe {
                let h = dlopen(c"libnativehelper.so".as_ptr(), RTLD_NOW);
                if h.is_null() {
                    return 0;
                }
                let f = dlsym(h, c"JNI_GetCreatedJavaVMs".as_ptr());
                if f.is_null() {
                    return 0;
                }
                let get: GetVms = std::mem::transmute(f);
                let mut vm: *mut JavaVM = std::ptr::null_mut();
                let mut n: jsize = 0;
                if get(&mut vm, 1, &mut n) != JNI_OK || n < 1 || vm.is_null() {
                    return 0;
                }
                vm as usize
            }
        });
        (p != 0).then_some(p as *mut JavaVM)
    }

    /// Whether the store can be reached at all. Does not touch the Keystore.
    pub fn available() -> bool {
        vm().is_some()
    }

    /// A JNI env for the calling thread, detaching on drop if this guard
    /// attached it (Godot's main thread is already attached; a worker is not).
    struct Env {
        env: *mut JNIEnv,
        vm: *mut JavaVM,
        attached_here: bool,
    }

    impl Drop for Env {
        fn drop(&mut self) {
            if self.attached_here {
                // SAFETY: this thread was attached by `Env::get` on `self.vm`.
                unsafe { ((**self.vm).v1_4.DetachCurrentThread)(self.vm) };
            }
        }
    }

    const NO_JVM: StoreError = StoreError::Failed("the Android Keystore could not be reached");
    const JAVA_FAILED: StoreError = StoreError::Failed("the Android Keystore refused the operation");

    impl Env {
        fn get() -> Result<Env, StoreError> {
            let vm = vm().ok_or(NO_JVM)?;
            let mut env: *mut c_void = std::ptr::null_mut();
            // SAFETY: `vm` is the live process VM; `env` is a valid out-pointer.
            let rc = unsafe { ((**vm).v1_4.GetEnv)(vm, &mut env, JNI_VERSION_1_6) };
            if rc == JNI_OK {
                return Ok(Env { env: env as *mut JNIEnv, vm, attached_here: false });
            }
            if rc != JNI_EDETACHED {
                return Err(NO_JVM);
            }
            // SAFETY: as above; null args attach with default name and group.
            let rc = unsafe { ((**vm).v1_4.AttachCurrentThread)(vm, &mut env, std::ptr::null_mut()) };
            if rc != JNI_OK || env.is_null() {
                return Err(NO_JVM);
            }
            Ok(Env { env: env as *mut JNIEnv, vm, attached_here: true })
        }

        /// Clears any pending exception; `true` if there was one. Never
        /// describes it (that prints to logcat).
        fn failed(&self) -> bool {
            // SAFETY: `self.env` is this thread's env.
            unsafe {
                let e = self.env;
                if ((**e).v1_6.ExceptionCheck)(e) {
                    ((**e).v1_6.ExceptionClear)(e);
                    true
                } else {
                    false
                }
            }
        }

        /// Turn a JNI result into `Err` when it is null or an exception is pending.
        fn ok<T: IsNull>(&self, v: T) -> Result<T, StoreError> {
            if self.failed() || v.is_null_ref() { Err(JAVA_FAILED) } else { Ok(v) }
        }

        fn class(&self, name: &std::ffi::CStr) -> Result<jclass, StoreError> {
            // SAFETY: NUL-terminated class name; result checked by `ok`.
            let c = unsafe { ((**self.env).v1_6.FindClass)(self.env, name.as_ptr()) };
            self.ok(c)
        }

        fn method(&self, c: jclass, name: &std::ffi::CStr, sig: &std::ffi::CStr) -> Result<jmethodID, StoreError> {
            // SAFETY: `c` is a live class ref; strings NUL-terminated.
            let m = unsafe { ((**self.env).v1_6.GetMethodID)(self.env, c, name.as_ptr(), sig.as_ptr()) };
            self.ok(m)
        }

        fn static_method(
            &self,
            c: jclass,
            name: &std::ffi::CStr,
            sig: &std::ffi::CStr,
        ) -> Result<jmethodID, StoreError> {
            // SAFETY: as `method`.
            let m = unsafe { ((**self.env).v1_6.GetStaticMethodID)(self.env, c, name.as_ptr(), sig.as_ptr()) };
            self.ok(m)
        }

        fn string(&self, s: &str) -> Result<jobject, StoreError> {
            let c = CString::new(s).map_err(|_| JAVA_FAILED)?;
            // SAFETY: NUL-terminated modified-UTF-8 (the inputs here are ASCII).
            let j = unsafe { ((**self.env).v1_6.NewStringUTF)(self.env, c.as_ptr()) };
            self.ok(j)
        }

        fn bytes(&self, b: &[u8]) -> Result<jbyteArray, StoreError> {
            let n = jsize::try_from(b.len()).map_err(|_| JAVA_FAILED)?;
            // SAFETY: a fresh array of `n` bytes, then a copy of exactly `n`.
            unsafe {
                let a = self.ok(((**self.env).v1_6.NewByteArray)(self.env, n))?;
                ((**self.env).v1_6.SetByteArrayRegion)(self.env, a, 0, n, b.as_ptr() as *const i8);
                self.ok(a)
            }
        }

        fn read_bytes(&self, a: jbyteArray) -> Result<Vec<u8>, StoreError> {
            // SAFETY: `a` is a live byte[]; the copy is bounded by its length.
            unsafe {
                let n = ((**self.env).v1_6.GetArrayLength)(self.env, a);
                let mut out = vec![0u8; n.max(0) as usize];
                ((**self.env).v1_6.GetByteArrayRegion)(self.env, a, 0, n, out.as_mut_ptr() as *mut i8);
                if self.failed() { Err(JAVA_FAILED) } else { Ok(out) }
            }
        }

        /// Overwrite a Java byte[] with zeros (best effort; the GC may have
        /// copied it, which is why the plaintext array lives as briefly as it can).
        fn wipe(&self, a: jbyteArray, n: usize) {
            let zeros = vec![0u8; n];
            // SAFETY: `a` has exactly `n` elements (created by `bytes`).
            unsafe {
                ((**self.env).v1_6.SetByteArrayRegion)(self.env, a, 0, n as jsize, zeros.as_ptr() as *const i8)
            };
            let _ = self.failed();
        }

        fn new_string_array(&self, items: &[&str]) -> Result<jobject, StoreError> {
            let sc = self.class(c"java/lang/String")?;
            // SAFETY: a fresh String[] of `items.len()` nulls, then filled.
            unsafe {
                let arr = self.ok(((**self.env).v1_6.NewObjectArray)(
                    self.env,
                    items.len() as jsize,
                    sc,
                    std::ptr::null_mut(),
                ))?;
                for (i, s) in items.iter().enumerate() {
                    let js = self.string(s)?;
                    ((**self.env).v1_6.SetObjectArrayElement)(self.env, arr, i as jsize, js);
                    if self.failed() {
                        return Err(JAVA_FAILED);
                    }
                }
                Ok(arr)
            }
        }

        fn call_obj(&self, o: jobject, m: jmethodID, args: &[jvalue]) -> Result<jobject, StoreError> {
            // SAFETY: `m` belongs to `o`'s class and `args` match its signature
            // (each call site pairs them by the literal signature beside it).
            let r = unsafe { ((**self.env).v1_6.CallObjectMethodA)(self.env, o, m, args.as_ptr()) };
            self.ok(r)
        }

        /// Like `call_obj`, but a null result is a normal answer (`getKey` of
        /// a missing alias), not a failure.
        fn call_obj_nullable(&self, o: jobject, m: jmethodID, args: &[jvalue]) -> Result<jobject, StoreError> {
            // SAFETY: as `call_obj`.
            let r = unsafe { ((**self.env).v1_6.CallObjectMethodA)(self.env, o, m, args.as_ptr()) };
            if self.failed() { Err(JAVA_FAILED) } else { Ok(r) }
        }

        fn call_void(&self, o: jobject, m: jmethodID, args: &[jvalue]) -> Result<(), StoreError> {
            // SAFETY: as `call_obj`.
            unsafe { ((**self.env).v1_6.CallVoidMethodA)(self.env, o, m, args.as_ptr()) };
            if self.failed() { Err(JAVA_FAILED) } else { Ok(()) }
        }

        fn call_static_obj(&self, c: jclass, m: jmethodID, args: &[jvalue]) -> Result<jobject, StoreError> {
            // SAFETY: as `call_obj`, for a static method of `c`.
            let r = unsafe { ((**self.env).v1_6.CallStaticObjectMethodA)(self.env, c, m, args.as_ptr()) };
            self.ok(r)
        }

        fn new_obj(&self, c: jclass, ctor: jmethodID, args: &[jvalue]) -> Result<jobject, StoreError> {
            // SAFETY: `ctor` is a `<init>` of `c` and `args` match it.
            let r = unsafe { ((**self.env).v1_6.NewObjectA)(self.env, c, ctor, args.as_ptr()) };
            self.ok(r)
        }

        /// Run `f` inside a JNI local frame so every local ref it makes is
        /// released on return, success or failure.
        fn frame<T>(&self, f: impl FnOnce(&Env) -> Result<T, StoreError>) -> Result<T, StoreError> {
            // SAFETY: balanced Push/Pop on this thread's env.
            unsafe {
                if ((**self.env).v1_6.PushLocalFrame)(self.env, 64) != JNI_OK {
                    let _ = self.failed();
                    return Err(JAVA_FAILED);
                }
                let r = f(self);
                let _ = self.failed();
                ((**self.env).v1_6.PopLocalFrame)(self.env, std::ptr::null_mut());
                r
            }
        }
    }

    trait IsNull {
        fn is_null_ref(&self) -> bool;
    }
    impl IsNull for jobject {
        fn is_null_ref(&self) -> bool {
            self.is_null()
        }
    }
    impl IsNull for jmethodID {
        fn is_null_ref(&self) -> bool {
            self.is_null()
        }
    }

    fn l(o: jobject) -> jvalue {
        jvalue { l: o }
    }
    fn i(v: jint) -> jvalue {
        jvalue { i: v }
    }

    /// The loaded `AndroidKeyStore` `KeyStore`.
    fn keystore(e: &Env) -> Result<jobject, StoreError> {
        let ksc = e.class(c"java/security/KeyStore")?;
        let get = e.static_method(ksc, c"getInstance", c"(Ljava/lang/String;)Ljava/security/KeyStore;")?;
        let ks = e.call_static_obj(ksc, get, &[l(e.string("AndroidKeyStore")?)])?;
        let load = e.method(ksc, c"load", c"(Ljava/security/KeyStore$LoadStoreParameter;)V")?;
        e.call_void(ks, load, &[l(std::ptr::null_mut())])?;
        Ok(ks)
    }

    /// The wrapping key for `alias`, or `None` when it does not exist.
    fn existing_key(e: &Env, ks: jobject, alias: &str) -> Result<Option<jobject>, StoreError> {
        let ksc = e.class(c"java/security/KeyStore")?;
        let get = e.method(ksc, c"getKey", c"(Ljava/lang/String;[C)Ljava/security/Key;")?;
        let k = e.call_obj_nullable(ks, get, &[l(e.string(alias)?), l(std::ptr::null_mut())])?;
        Ok((!k.is_null()).then_some(k))
    }

    /// Create the AES-256-GCM wrapping key for `alias` inside the Keystore.
    fn generate_key(e: &Env, alias: &str) -> Result<jobject, StoreError> {
        let bc = e.class(c"android/security/keystore/KeyGenParameterSpec$Builder")?;
        let ctor = e.method(bc, c"<init>", c"(Ljava/lang/String;I)V")?;
        let b = e.new_obj(bc, ctor, &[l(e.string(alias)?), i(PURPOSES)])?;
        let ret = c"([Ljava/lang/String;)Landroid/security/keystore/KeyGenParameterSpec$Builder;";
        let modes = e.method(bc, c"setBlockModes", ret)?;
        e.call_obj(b, modes, &[l(e.new_string_array(&["GCM"])?)])?;
        let pads = e.method(bc, c"setEncryptionPaddings", ret)?;
        e.call_obj(b, pads, &[l(e.new_string_array(&["NoPadding"])?)])?;
        let size = e.method(bc, c"setKeySize", c"(I)Landroid/security/keystore/KeyGenParameterSpec$Builder;")?;
        e.call_obj(b, size, &[i(KEY_BITS)])?;
        let build = e.method(bc, c"build", c"()Landroid/security/keystore/KeyGenParameterSpec;")?;
        let spec = e.call_obj(b, build, &[])?;
        let kgc = e.class(c"javax/crypto/KeyGenerator")?;
        let get = e.static_method(
            kgc,
            c"getInstance",
            c"(Ljava/lang/String;Ljava/lang/String;)Ljavax/crypto/KeyGenerator;",
        )?;
        let kg = e.call_static_obj(kgc, get, &[l(e.string("AES")?), l(e.string("AndroidKeyStore")?)])?;
        let init = e.method(kgc, c"init", c"(Ljava/security/spec/AlgorithmParameterSpec;)V")?;
        e.call_void(kg, init, &[l(spec)])?;
        let gen_key = e.method(kgc, c"generateKey", c"()Ljavax/crypto/SecretKey;")?;
        e.call_obj(kg, gen_key, &[])
    }

    /// A fresh `AES/GCM/NoPadding` cipher.
    fn cipher(e: &Env) -> Result<(jclass, jobject), StoreError> {
        let cc = e.class(c"javax/crypto/Cipher")?;
        let get = e.static_method(cc, c"getInstance", c"(Ljava/lang/String;)Ljavax/crypto/Cipher;")?;
        let c = e.call_static_obj(cc, get, &[l(e.string("AES/GCM/NoPadding")?)])?;
        Ok((cc, c))
    }

    fn store_dir() -> Option<std::path::PathBuf> {
        use godot::obj::Singleton as _;
        let base = godot::classes::Os::singleton().get_user_data_dir().to_string();
        (!base.is_empty()).then(|| std::path::Path::new(&base).join(DIR_NAME))
    }

    fn file_for(account: &str) -> Result<std::path::PathBuf, StoreError> {
        store_dir()
            .map(|d| d.join(file_name_for(account)))
            .ok_or(StoreError::Failed("the app storage folder is not known"))
    }

    /// Encrypt and save. The plaintext crosses into Java as one byte[] that
    /// is overwritten with zeros straight after `doFinal`.
    pub fn write(account: &str, secret: &str) -> Result<(), StoreError> {
        let path = file_for(account)?;
        let env = Env::get()?;
        let encoded = env.frame(|e| {
            let ks = keystore(e)?;
            let alias = alias_for(account);
            let key = match existing_key(e, ks, &alias)? {
                Some(k) => k,
                None => generate_key(e, &alias)?,
            };
            let (cc, c) = cipher(e)?;
            let init = e.method(cc, c"init", c"(ILjava/security/Key;)V")?;
            e.call_void(c, init, &[i(ENCRYPT_MODE), l(key)])?;
            let aad = e.method(cc, c"updateAAD", c"([B)V")?;
            e.call_void(c, aad, &[l(e.bytes(account.as_bytes())?)])?;
            let fin = e.method(cc, c"doFinal", c"([B)[B")?;
            let plain = e.bytes(secret.as_bytes())?;
            let ct = e.call_obj(c, fin, &[l(plain)]);
            e.wipe(plain, secret.len());
            let ct = e.read_bytes(ct?)?;
            let get_iv = e.method(cc, c"getIV", c"()[B")?;
            let iv = e.read_bytes(e.call_obj(c, get_iv, &[])?)?;
            blob::encode(&iv, &ct).ok_or(JAVA_FAILED)
        })?;
        drop(env);
        let dir = path.parent().ok_or(StoreError::Failed("the app storage folder is not known"))?;
        std::fs::create_dir_all(dir).map_err(|_| StoreError::Failed("could not create the key folder"))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &encoded).map_err(|_| StoreError::Failed("could not write the encrypted key"))?;
        std::fs::rename(&tmp, &path).map_err(|_| {
            let _ = std::fs::remove_file(&tmp);
            StoreError::Failed("could not write the encrypted key")
        })
    }

    /// Read and decrypt; `Ok(None)` when nothing is saved for `account`.
    pub fn read(account: &str) -> Result<Option<String>, StoreError> {
        let path = file_for(account)?;
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(StoreError::Failed("could not read the encrypted key")),
        };
        let (iv, ct) =
            blob::decode(&bytes).ok_or(StoreError::Failed("the saved key file is damaged; save the key again"))?;
        let env = Env::get()?;
        let plain = env.frame(|e| {
            let ks = keystore(e)?;
            let key = existing_key(e, ks, &alias_for(account))?
                .ok_or(StoreError::Failed("the saved key can no longer be unlocked; save the key again"))?;
            let sc = e.class(c"javax/crypto/spec/GCMParameterSpec")?;
            let ctor = e.method(sc, c"<init>", c"(I[B)V")?;
            let spec = e.new_obj(sc, ctor, &[i(TAG_BITS), l(e.bytes(iv)?)])?;
            let (cc, c) = cipher(e)?;
            let init = e.method(cc, c"init", c"(ILjava/security/Key;Ljava/security/spec/AlgorithmParameterSpec;)V")?;
            e.call_void(c, init, &[i(DECRYPT_MODE), l(key), l(spec)])?;
            let aad = e.method(cc, c"updateAAD", c"([B)V")?;
            e.call_void(c, aad, &[l(e.bytes(account.as_bytes())?)])?;
            let fin = e.method(cc, c"doFinal", c"([B)[B")?;
            // A failed tag check (a file from another account, a tampered
            // file) throws AEADBadTagException: a refusal, never a guess.
            let out = e
                .call_obj(c, fin, &[l(e.bytes(ct)?)])
                .map_err(|_| StoreError::Failed("the saved key can no longer be unlocked; save the key again"))?;
            let mut v = e.read_bytes(out)?;
            let n = v.len();
            e.wipe(out, n);
            String::from_utf8(std::mem::take(&mut v)).map_err(|_| StoreError::Failed("the saved value is not text"))
        })?;
        Ok(Some(plain))
    }

    /// Remove the file and the wrapping key. Removing nothing is success.
    pub fn delete(account: &str) -> Result<(), StoreError> {
        let path = file_for(account)?;
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(StoreError::Failed("could not remove the encrypted key")),
        }
        let env = Env::get()?;
        env.frame(|e| {
            let ks = keystore(e)?;
            let ksc = e.class(c"java/security/KeyStore")?;
            let del = e.method(ksc, c"deleteEntry", c"(Ljava/lang/String;)V")?;
            e.call_void(ks, del, &[l(e.string(&alias_for(account))?)])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the saved-file format round-trips, byte for byte, with the
    /// layout written out as a literal. MUTATION: change `MAGIC`, drop the
    /// length byte, or swap iv and ciphertext, and the literal goes red.
    #[test]
    fn blob_round_trips_with_a_literal_layout() {
        let iv = [7u8; 12];
        let ct = [9u8; 20];
        let enc = blob::encode(&iv, &ct).unwrap();
        let mut want = b"CLK1".to_vec();
        want.push(12);
        want.extend_from_slice(&[7u8; 12]);
        want.extend_from_slice(&[9u8; 20]);
        assert_eq!(enc, want);
        let (i2, c2) = blob::decode(&enc).unwrap();
        assert_eq!((i2, c2), (&iv[..], &ct[..]));
    }

    /// Protects: a damaged or foreign file is refused, never decoded into a
    /// plausible nonce and ciphertext. MUTATION: drop the magic check or the
    /// tag-length bound and a row goes red.
    #[test]
    fn blob_refuses_anything_else() {
        let good = blob::encode(&[1u8; 12], &[2u8; 17]).unwrap();
        assert!(blob::decode(&good).is_some());
        let mut bad_magic = good.clone();
        bad_magic[0] = b'X';
        assert!(blob::decode(&bad_magic).is_none(), "wrong magic");
        assert!(blob::decode(b"").is_none());
        assert!(blob::decode(b"CLK1").is_none(), "no length byte");
        let mut zero_iv = b"CLK1".to_vec();
        zero_iv.push(0);
        zero_iv.extend_from_slice(&[0u8; 40]);
        assert!(blob::decode(&zero_iv).is_none(), "empty nonce");
        let mut long_iv = b"CLK1".to_vec();
        long_iv.push(33);
        long_iv.extend_from_slice(&[0u8; 80]);
        assert!(blob::decode(&long_iv).is_none(), "nonce over 32 bytes");
        // 12-byte nonce and exactly 16 bytes after it: a tag with no
        // ciphertext, which no non-empty key can produce.
        let mut tag_only = b"CLK1".to_vec();
        tag_only.push(12);
        tag_only.extend_from_slice(&[0u8; 28]);
        assert!(blob::decode(&tag_only).is_none(), "tag only");
        assert!(blob::encode(&[], &[0u8; 20]).is_none());
        assert!(blob::encode(&[0u8; 33], &[0u8; 20]).is_none());
        assert!(blob::encode(&[0u8; 12], &[0u8; 16]).is_none());
        assert_eq!(blob::TAG_BYTES, 16, "GCMParameterSpec(128) is a 16-byte tag");
    }

    /// Protects: the alias and file name are the documented literals, so an
    /// installed key is found again after an update.
    #[test]
    fn names_are_stable() {
        assert_eq!(alias_for("openai"), "cartalith_llm_openai");
        assert_eq!(file_name_for("lmstudio"), "lmstudio.bin");
        assert_eq!(DIR_NAME, "llm_keystore");
    }
}
