use std::{collections::BTreeMap, sync::Arc};

use js_sys::{Array, Map, Promise, Set};
use ruma::{DeviceKeyAlgorithm, UInt};
use wasm_bindgen::prelude::*;

use crate::js::{future::future_to_promise, identifiers, sync_events};

#[wasm_bindgen]
#[derive(Debug)]
pub struct OlmMachine {
    inner: Arc<crate::OlmMachine>,
}

#[cfg_attr(feature = "js", wasm_bindgen)]
impl OlmMachine {
    #[wasm_bindgen(constructor)]
    pub fn new(user_id: &identifiers::UserId, device_id: &identifiers::DeviceId) -> Promise {
        let user_id = user_id.inner.clone();
        let device_id = device_id.inner.clone();

        future_to_promise(async move {
            Ok(OlmMachine {
                inner: Arc::new(crate::OlmMachine::new(user_id.as_ref(), device_id.as_ref()).await),
            })
        })
    }

    /// The unique user ID that owns this `OlmMachine` instance.
    pub fn user_id(&self) -> identifiers::UserId {
        identifiers::UserId { inner: self.inner.user_id().to_owned() }
    }

    /// The unique device ID that identifies this `OlmMachine`.
    pub fn device_id(&self) -> identifiers::DeviceId {
        identifiers::DeviceId { inner: self.inner.device_id().to_owned() }
    }

    ///// Get the public parts of our Olm identity keys.
    pub fn identity_keys(&self) -> IdentityKeys {
        self.inner.identity_keys().into()
    }

    /// Get the display name of our own device.
    pub fn display_name(&self) -> Promise {
        let me = self.inner.clone();

        future_to_promise(async move { Ok(me.display_name().await?) })
    }

    pub fn receive_sync_changes(
        &self,
        to_device_events: &str,
        changed_devices: &sync_events::DeviceLists,
        one_time_key_counts: &Map,
        unused_fallback_keys: &Set,
    ) -> Result<Promise, JsError> {
        let to_device_events = serde_json::from_str(to_device_events)?;
        let changed_devices = changed_devices.inner.clone();
        let one_time_key_counts: BTreeMap<DeviceKeyAlgorithm, UInt> = one_time_key_counts
            .entries()
            .into_iter()
            .filter_map(|js_value| {
                let pair = Array::from(&js_value.ok()?);
                let (key, value) = (
                    DeviceKeyAlgorithm::from(pair.at(0).as_string()?),
                    UInt::new(pair.at(1).as_f64()? as u64)?,
                );

                Some((key, value))
            })
            .collect();
        let unused_fallback_keys: Option<Vec<DeviceKeyAlgorithm>> = Some(
            unused_fallback_keys
                .values()
                .into_iter()
                .filter_map(|js_value| Some(DeviceKeyAlgorithm::from(js_value.ok()?.as_string()?)))
                .collect(),
        );

        let me = self.inner.clone();

        Ok(future_to_promise(async move {
            Ok(serde_json::to_string(
                &me.receive_sync_changes(
                    to_device_events,
                    &changed_devices,
                    &one_time_key_counts,
                    unused_fallback_keys.as_deref(),
                )
                .await?,
            )?)
        }))
    }

    pub fn outgoing_requests(&self) -> Promise {
        let me = self.inner.clone();

        future_to_promise(async move {
            Ok(me
                .outgoing_requests()
                .await?
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<JsValue>, _>>()?
                .into_iter()
                .collect::<Array>())
        })
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct Ed25519PublicKey {
    inner: vodozemac::Ed25519PublicKey,
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct Curve25519PublicKey {
    inner: vodozemac::Curve25519PublicKey,
}

#[derive(Debug)]
#[wasm_bindgen(getter_with_clone)]
pub struct IdentityKeys {
    pub ed25519: Ed25519PublicKey,
    pub curve25519: Curve25519PublicKey,
}

impl From<crate::olm::IdentityKeys> for IdentityKeys {
    fn from(value: crate::olm::IdentityKeys) -> Self {
        Self {
            ed25519: Ed25519PublicKey { inner: value.ed25519 },
            curve25519: Curve25519PublicKey { inner: value.curve25519 },
        }
    }
}
