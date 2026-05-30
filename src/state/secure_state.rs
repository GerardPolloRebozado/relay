use keyring_core::set_default_store;

#[cfg(target_os = "windows")]
pub fn init_secure_storage() {
    use windows_native_keyring_store::Store;
    let store = Store::new();

    if store.is_err() {
        panic!("Can't open keychain")
    }
    let store = store.unwrap();
    set_default_store(store);
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn init_secure_storage() {
    use apple_native_keyring_store::keychain::Store;
    let store = Store::new();

    if store.is_err() {
        panic!("Can't open keychain")
    }
    let store = store.unwrap();
    set_default_store(store);
}

#[cfg(target_os = "linux")]
pub fn init_secure_storage() {
    use linux_keyutils_keyring_store::Store;
    let store = Store::new();

    if store.is_err() {
        panic!("Can't open keychain")
    }
    let store = store.unwrap();
    set_default_store(store);
}

#[cfg(target_os = "android")]
pub fn init_secure_storage() {
    use android_native_keyring_store::Store;
    let store = Store::new();

    if store.is_err() {
        panic!("Can't open keychain")
    }
    let store = store.unwrap();
    set_default_store(store);
}
