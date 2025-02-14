// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use async_recursion::async_recursion;
use futures::future::AbortHandle;
use tokio::sync::RwLock;
use tracing::trace;
use uuid::Uuid;

use super::{
    actor::{
        addr::{Addr, OptionalAddr},
        sender::IsClosed,
        Actor,
    },
    shutdown::ShutdownHandle,
};

/// An alias type indicating that this is a scope id
pub(crate) type ScopeId = Uuid;

/// The root scope id, which is always a zeroed uuid.
pub(crate) const ROOT_SCOPE: Uuid = Uuid::nil();

/// A scope, which marks data as usable for a given task.
#[derive(Clone)]
pub(crate) struct Scope {
    pub(crate) inner: Arc<ScopeInner>,
    valid: Arc<AtomicBool>,
}

/// Shared scope information.
#[derive(Debug)]
pub(crate) struct ScopeInner {
    pub(crate) id: ScopeId,
    address_registry: RwLock<AddressRegistry>,
    shutdown_handle: RwLock<Option<ShutdownHandle>>,
    abort_handle: AbortHandle,
    parent: Option<Scope>,
    children: RwLock<HashMap<ScopeId, Scope>>,
}

impl Scope {
    pub(crate) fn root(abort_handle: AbortHandle) -> Scope {
        Scope {
            inner: Arc::new(ScopeInner {
                id: ROOT_SCOPE,
                address_registry: Default::default(),
                shutdown_handle: Default::default(),
                abort_handle,
                parent: None,
                children: Default::default(),
            }),
            valid: Arc::new(AtomicBool::new(true)),
        }
    }

    pub(crate) async fn child(&self, abort_handle: AbortHandle) -> Self {
        trace!("Adding child to {:x}", self.id.as_fields().0);
        let id = Uuid::new_v4();
        let parent = self.clone();
        let child = Scope {
            inner: Arc::new(ScopeInner {
                id,
                address_registry: Default::default(),
                shutdown_handle: Default::default(),
                abort_handle,
                parent: Some(parent),
                children: Default::default(),
            }),
            valid: Arc::new(AtomicBool::new(true)),
        };
        self.children.write().await.insert(id, child.clone());
        trace!("Added child to {:x}", self.id.as_fields().0);
        child
    }

    pub(crate) async fn set_shutdown_handle(&self, handle: ShutdownHandle) {
        self.inner.shutdown_handle.write().await.replace(handle);
    }

    /// Finds a scope by id.
    pub(crate) fn find(&self, id: ScopeId) -> Option<&Scope> {
        if id == self.id {
            Some(self)
        } else {
            self.parent.as_ref().and_then(|p| p.find(id))
        }
    }

    pub(crate) fn parent(&self) -> Option<&Scope> {
        self.parent.as_ref()
    }

    pub(crate) async fn children(&self) -> Vec<Scope> {
        self.children.read().await.values().cloned().collect()
    }

    pub(crate) async fn insert_addr<A: 'static + Actor>(&self, addr: Addr<A>) {
        self.address_registry.write().await.insert(addr);
    }

    pub(crate) async fn get_addr<A: 'static + Actor>(&self) -> OptionalAddr<A> {
        let mut curr_scope = Some(self);
        while let Some(scope) = curr_scope {
            let opt_addr = scope.address_registry.read().await.get();
            if opt_addr.is_none() {
                curr_scope = scope.parent.as_ref();
            } else {
                return opt_addr;
            }
        }
        None.into()
    }

    pub(crate) async fn drop(&self) {
        trace!("Dropping scope {:x}", self.id.as_fields().0);
        if let Some(parent) = self.parent.as_ref() {
            parent.children.write().await.remove(&self.id);
        }
        trace!("Dropped scope {:x}", self.id.as_fields().0);
    }

    pub(crate) async fn shutdown(&self) {
        trace!("Shutting down scope {:x}", self.id.as_fields().0);
        self.valid.store(false, Ordering::Release);
        if let Some(handle) = self.shutdown_handle.read().await.as_ref() {
            handle.shutdown();
        } else {
            self.abort_handle.abort();
        }
        trace!("Shut down scope {:x}", self.id.as_fields().0);
    }

    /// Aborts the tasks in this scope.
    #[async_recursion]
    pub(crate) async fn abort(&self) {
        trace!("Aborting scope {:x}", self.id.as_fields().0);
        let children = self.children().await;
        for child_scope in children {
            child_scope.abort().await;
        }
        self.shutdown().await;
        trace!("Aborted scope {:x}", self.id.as_fields().0);
    }
}

impl Deref for Scope {
    type Target = ScopeInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("inner", &self.inner)
            .field("valid", &self.valid)
            .finish()
    }
}

#[derive(Debug, Default)]
pub(crate) struct AddressRegistry {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl AddressRegistry {
    pub(crate) fn insert<A>(&mut self, addr: Addr<A>)
    where
        A: 'static + Actor,
    {
        self.map.insert(TypeId::of::<A>(), Box::new(addr));
    }

    pub(crate) fn get<A>(&self) -> OptionalAddr<A>
    where
        A: 'static + Actor,
    {
        self.map
            .get(&TypeId::of::<A>())
            .and_then(|addr| addr.downcast_ref())
            .and_then(|addr: &Addr<A>| (!addr.is_closed()).then(|| addr.clone()))
            .into()
    }
}
