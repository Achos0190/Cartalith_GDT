//! The OS credential store behind the LLM assistant's API key
//! (`LLM_ASSISTANT_SCOPE.md` LLM-2, Ruling DD item 4).
//!
//! WHY HERE: `cartalith-llm` is held to a single dependency so it can never
//! open a socket, and the Windows Credential Manager needs `windows-sys`. The
//! pure rules (resolution order, what "unavailable" means) live there behind
//! the [`cartalith_llm::KeyStore`] trait; this file is the one platform
//! implementation of it.
//!
//! WINDOWS: a generic credential (`CRED_TYPE_GENERIC`) named
//! `Cartalith/LLM/<account>`, persisted for this user on this machine
//! (`CRED_PERSIST_LOCAL_MACHINE`: it does not roam). The secret is stored as
//! its UTF-8 bytes in the credential blob, which Windows protects with the
//! user's logon credentials (DPAPI). Nothing is ever written to a file.
//!
//! EVERYWHERE ELSE: [`OsKeyStore::available`] is `false` and every call says
//! so. There is no fallback to a file or to anything else; the caller refuses
//! (scope s.1 rule 5). Android's Keystore is LLM-10.
//!
//! MUST NEVER: log, print or return the secret except through
//! [`cartalith_llm::KeyStore::read`]'s one caller (which wraps it in an
//! `ApiKey` at once); store anything but a validated key; accept an account
//! id that is not `[a-z0-9_-]`, so a stray character cannot build an odd
//! credential target name.

use cartalith_llm::keystore::{KeyStore, StoreError, account_ok};

/// The credential-store prefix; the full target is `Cartalith/LLM/<account>`.
pub const TARGET_PREFIX: &str = "Cartalith/LLM/";

/// The longest secret stored, in bytes. `CRED_MAX_CREDENTIAL_BLOB_SIZE` is
/// 2560 in the Windows SDK; real API keys are far shorter (a few hundred).
pub const MAX_SECRET_BYTES: usize = 2560;

/// The platform's credential store.
pub struct OsKeyStore;

/// The full credential target for an account, or `None` for an illegal id.
fn target(account: &str) -> Option<String> {
    account_ok(account).then(|| format!("{TARGET_PREFIX}{account}"))
}

#[cfg(windows)]
mod win {
    use super::*;
    use windows_sys::Win32::Foundation::{ERROR_NOT_FOUND, GetLastError};
    use windows_sys::Win32::Security::Credentials::{
        CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree, CredReadW, CredWriteW,
    };

    /// A NUL-terminated UTF-16 copy of `s`.
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn read(account: &str) -> Result<Option<String>, StoreError> {
        let name = wide(&target(account).ok_or(StoreError::Failed("the account id is not valid"))?);
        let mut out: *mut CREDENTIALW = std::ptr::null_mut();
        // SAFETY: `name` is NUL-terminated and outlives the call; `out` is a
        // live out-pointer. On success Windows allocates `*out`, which is
        // freed with `CredFree` below on every path that sees it non-null.
        let ok = unsafe { CredReadW(name.as_ptr(), CRED_TYPE_GENERIC, 0, &mut out) };
        if ok == 0 {
            // SAFETY: `GetLastError` reads thread-local state and has no
            // preconditions; called immediately after the failing call.
            let err = unsafe { GetLastError() };
            return if err == ERROR_NOT_FOUND { Ok(None) } else { Err(StoreError::Failed("the credential store refused the read")) };
        }
        // SAFETY: `ok != 0` means `out` points at a CREDENTIALW that Windows
        // allocated, valid until `CredFree`. The blob is `CredentialBlobSize`
        // bytes at `CredentialBlob` (null only when the size is 0).
        let bytes: Vec<u8> = unsafe {
            let c = &*out;
            if c.CredentialBlob.is_null() || c.CredentialBlobSize == 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize).to_vec()
            }
        };
        // SAFETY: `out` came from a successful `CredReadW` and is freed once.
        unsafe { CredFree(out as *const _) };
        if bytes.is_empty() {
            // An empty blob is "nothing saved", never an empty key.
            return Ok(None);
        }
        String::from_utf8(bytes).map(Some).map_err(|_| StoreError::Failed("the saved value is not text"))
    }

    pub fn write(account: &str, secret: &str) -> Result<(), StoreError> {
        if secret.is_empty() || secret.len() > MAX_SECRET_BYTES {
            return Err(StoreError::Failed("the secret is empty or too long"));
        }
        let name = wide(&target(account).ok_or(StoreError::Failed("the account id is not valid"))?);
        let mut blob = secret.as_bytes().to_vec();
        let mut cred: CREDENTIALW = unsafe { std::mem::zeroed() };
        cred.Type = CRED_TYPE_GENERIC;
        cred.TargetName = name.as_ptr() as *mut u16;
        cred.CredentialBlobSize = blob.len() as u32;
        cred.CredentialBlob = blob.as_mut_ptr();
        cred.Persist = CRED_PERSIST_LOCAL_MACHINE;
        // SAFETY: `cred` is fully initialised (zeroed, then the fields above);
        // `name` and `blob` outlive the call; Windows copies the data and does
        // not keep the pointers.
        let ok = unsafe { CredWriteW(&cred, 0) };
        // Overwrite our copy of the secret before it is freed.
        blob.iter_mut().for_each(|b| *b = 0);
        if ok == 0 { Err(StoreError::Failed("the credential store refused the write")) } else { Ok(()) }
    }

    pub fn delete(account: &str) -> Result<(), StoreError> {
        let name = wide(&target(account).ok_or(StoreError::Failed("the account id is not valid"))?);
        // SAFETY: `name` is NUL-terminated and outlives the call.
        let ok = unsafe { CredDeleteW(name.as_ptr(), CRED_TYPE_GENERIC, 0) };
        if ok != 0 {
            return Ok(());
        }
        // SAFETY: as in `read`.
        let err = unsafe { GetLastError() };
        if err == ERROR_NOT_FOUND { Ok(()) } else { Err(StoreError::Failed("the credential store refused the delete")) }
    }
}

impl KeyStore for OsKeyStore {
    /// Windows: always. Android: when the process JVM can be reached
    /// (`libnativehelper.so`, API 31+); see `llm_keystore_android`.
    /// Elsewhere: never.
    fn available(&self) -> bool {
        #[cfg(target_os = "android")]
        {
            crate::llm_keystore_android::available()
        }
        #[cfg(not(target_os = "android"))]
        {
            cfg!(windows)
        }
    }

    fn read(&self, account: &str) -> Result<Option<String>, StoreError> {
        target(account).ok_or(StoreError::Failed("the account id is not valid"))?;
        #[cfg(windows)]
        return win::read(account);
        #[cfg(target_os = "android")]
        return if self.available() { crate::llm_keystore_android::read(account) } else { Err(StoreError::Unavailable) };
        #[cfg(not(any(windows, target_os = "android")))]
        Err(StoreError::Unavailable)
    }

    fn write(&self, account: &str, secret: &str) -> Result<(), StoreError> {
        target(account).ok_or(StoreError::Failed("the account id is not valid"))?;
        if secret.is_empty() || secret.len() > MAX_SECRET_BYTES {
            return Err(StoreError::Failed("the secret is empty or too long"));
        }
        #[cfg(windows)]
        return win::write(account, secret);
        #[cfg(target_os = "android")]
        return if self.available() {
            crate::llm_keystore_android::write(account, secret)
        } else {
            Err(StoreError::Unavailable)
        };
        #[cfg(not(any(windows, target_os = "android")))]
        Err(StoreError::Unavailable)
    }

    fn delete(&self, account: &str) -> Result<(), StoreError> {
        target(account).ok_or(StoreError::Failed("the account id is not valid"))?;
        #[cfg(windows)]
        return win::delete(account);
        #[cfg(target_os = "android")]
        return if self.available() { crate::llm_keystore_android::delete(account) } else { Err(StoreError::Unavailable) };
        #[cfg(not(any(windows, target_os = "android")))]
        Err(StoreError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the account-to-target rule, on every platform.
    #[test]
    fn targets_are_prefixed_and_illegal_ids_have_none() {
        assert_eq!(target("openai").as_deref(), Some("Cartalith/LLM/openai"));
        assert_eq!(target("custom_openai").as_deref(), Some("Cartalith/LLM/custom_openai"));
        for bad in ["", "A", "a b", "a/b", "a\\b", "..", &"x".repeat(65)] {
            assert!(target(bad).is_none(), "{bad:?}");
        }
    }

    /// Protects the LLM-2 bar "Windows: key round-trips through Credential
    /// Manager": write, read the same bytes back, overwrite, delete, read
    /// `None`, delete again without error. Uses a private account id so a
    /// user's real saved key is never touched, and deletes it on every path.
    /// On a platform with no store it asserts the refusal instead of passing
    /// vacuously.
    #[test]
    fn credential_manager_round_trip() {
        let s = OsKeyStore;
        let acct = format!("test-roundtrip-{}", std::process::id());
        if !s.available() {
            assert_eq!(s.write(&acct, "sk-x"), Err(StoreError::Unavailable));
            assert_eq!(s.read(&acct), Err(StoreError::Unavailable));
            return;
        }
        let _ = s.delete(&acct);
        assert_eq!(s.read(&acct), Ok(None), "nothing saved yet");
        let secret = "sk-test-\u{7e}\u{21}abcdef0123456789";
        let result = std::panic::catch_unwind(|| {
            let s = OsKeyStore;
            s.write(&acct, secret).unwrap();
            assert_eq!(s.read(&acct).unwrap().as_deref(), Some(secret));
            s.write(&acct, "sk-second-value-0000").unwrap();
            assert_eq!(s.read(&acct).unwrap().as_deref(), Some("sk-second-value-0000"));
            s.delete(&acct).unwrap();
            assert_eq!(s.read(&acct), Ok(None));
            assert_eq!(s.delete(&acct), Ok(()), "deleting nothing is not an error");
            assert!(s.write(&acct, "").is_err(), "an empty secret is refused, not stored");
            assert!(s.write(&acct, &"k".repeat(MAX_SECRET_BYTES + 1)).is_err());
            assert!(s.write("Bad Account", "sk-x").is_err());
        });
        let _ = s.delete(&acct);
        if let Err(p) = result {
            std::panic::resume_unwind(p);
        }
    }
}
