use crate::actor::abstract_actor::{Actor, HandleError};
use crate::actor::message::Message;
use crate::system::System;
use std::future::Future;
use std::sync::Arc;

pub type ContextResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type ContextInitializationError<T> = Result<T, String>;

pub trait ActorContext: Sized + Unpin + 'static {
    #[inline]
    fn on_start(&mut self) -> ContextResult<()> {
        Ok(())
    }
    #[inline]
    fn on_die(&mut self, _actor_name: Arc<str>) -> ContextResult<()> {
        Ok(())
    }
    #[inline]
    fn after_iteration(&mut self) -> () {}
    #[inline]
    fn on_error(&mut self, _error: &HandleError) -> () {}
    fn kill(&mut self);
    fn get_name(&self) -> &str;
    #[allow(clippy::wrong_self_convention)]
    fn is_alive(&self) -> bool {
        true
    }
    fn create<A: Actor>(
        system: &mut System,
        name: Arc<str>,
    ) -> impl Future<Output = ContextInitializationError<Self>> + Send;
}

pub struct ActorDied(pub Arc<str>);

impl Message for ActorDied {
    fn static_name() -> &'static str {
        "ActorDied"
    }
}

#[derive(derive_more::Constructor)]
pub struct Context {
    alive: bool,
    name: Arc<str>,
}

impl ActorContext for Context {
    fn kill(&mut self) {
        self.alive = false;
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_alive(&self) -> bool {
        self.alive
    }

    async fn create<A: Actor>(_: &mut System, name: Arc<str>) -> ContextInitializationError<Self> {
        Ok(Context { alive: true, name })
    }
}

pub mod extensions {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::fmt;
    use std::hash::{BuildHasherDefault, Hasher};
    use std::ops::{Deref, DerefMut};
    use std::sync::Arc;

    type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>;

    // With TypeIds as keys, there's no need to hash them. They are already hashes
    // themselves, coming from the compiler. The IdHasher just holds the u64 of
    // the TypeId, and then returns it, instead of doing any bit fiddling.
    #[derive(Default)]
    pub struct IdHasher(u64);

    impl Hasher for IdHasher {
        fn write(&mut self, _: &[u8]) {
            unreachable!("TypeId calls write_u64");
        }

        #[inline]
        fn write_u64(&mut self, id: u64) {
            self.0 = id;
        }

        #[inline]
        fn finish(&self) -> u64 {
            self.0
        }
    }

    #[derive(Default)]
    pub struct Extensions {
        // If extensions are never used, no need to carry around an empty HashMap.
        // That's 3 words. Instead, this is only 1 word.
        map: Option<Box<AnyMap>>,
    }

    impl Extensions {
        /// Create an empty `Extensions`.
        #[inline]
        pub fn new() -> Extensions {
            Extensions { map: None }
        }

        /// Insert a type into this `Extensions`.
        ///
        /// If a extension of this type already existed, it will
        /// be returned.
        ///
        /// # Example
        ///
        /// ```
        /// # use uactor::actor::context::extensions::Extensions;
        /// let mut ext = Extensions::new();
        /// assert!(ext.insert(5i32).is_none());
        /// assert!(ext.insert(4u8).is_none());
        /// assert_eq!(ext.insert(9i32), Some(5i32));
        /// ```
        // pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
            self.map
                .get_or_insert_with(Box::<HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>>::default
                )
                .insert(TypeId::of::<T>(), Box::new(val))
                .and_then(|boxed| {
                    (boxed as Box<dyn Any + 'static>)
                        .downcast()
                        .ok()
                        .map(|boxed| *boxed)
                })
        }

        /// Get a reference to a type previously inserted on this `Extensions`.
        ///
        /// # Example
        ///
        /// ```
        /// # use uactor::actor::context::extensions::Extensions;
        /// let mut ext = Extensions::new();
        /// assert!(ext.get::<i32>().is_none());
        /// ext.insert(5i32);
        ///
        /// assert_eq!(ext.get::<i32>(), Some(&5i32));
        /// ```
        pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
            self.map
                .as_ref()
                .and_then(|map| map.get(&TypeId::of::<T>()))
                .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
        }

        /// Get a mutable reference to a type previously inserted on this `Extensions`.
        ///
        /// # Example
        ///
        /// ```
        /// # use uactor::actor::context::extensions::Extensions;
        /// let mut ext = Extensions::new();
        /// ext.insert(String::from("Hello"));
        /// ext.get_mut::<String>().unwrap().push_str(" World");
        ///
        /// assert_eq!(ext.get::<String>().unwrap(), "Hello World");
        /// ```
        pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
            self.map
                .as_mut()
                .and_then(|map| map.get_mut(&TypeId::of::<T>()))
                .and_then(|boxed| (&mut **boxed as &mut (dyn Any + 'static)).downcast_mut())
        }

        /// Remove a type from this `Extensions`.
        ///
        /// If a extension of this type existed, it will be returned.
        ///
        /// # Example
        ///
        /// ```
        /// # use uactor::actor::context::extensions::Extensions;
        /// let mut ext = Extensions::new();
        /// ext.insert(5i32);
        /// assert_eq!(ext.remove::<i32>(), Some(5i32));
        /// assert!(ext.get::<i32>().is_none());
        /// ```
        pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
            self.map
                .as_mut()
                .and_then(|map| map.remove(&TypeId::of::<T>()))
                .and_then(|boxed| {
                    (boxed as Box<dyn Any + 'static>)
                        .downcast()
                        .ok()
                        .map(|boxed| *boxed)
                })
        }

        /// Clear the `Extensions` of all inserted extensions.
        ///
        /// # Example
        ///
        /// ```
        /// # use uactor::actor::context::extensions::Extensions;
        /// let mut ext = Extensions::new();
        /// ext.insert(5i32);
        /// ext.clear();
        ///
        /// assert!(ext.get::<i32>().is_none());
        /// ```
        #[inline]
        pub fn clear(&mut self) {
            if let Some(ref mut map) = self.map {
                map.clear();
            }
        }

        /// Check whether the extension set is empty or not.
        ///
        /// # Example
        ///
        /// ```
        /// # use uactor::actor::context::extensions::Extensions;
        /// let mut ext = Extensions::new();
        /// assert!(ext.is_empty());
        /// ext.insert(5i32);
        /// assert!(!ext.is_empty());
        /// ```
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.map.as_ref().map_or(true, |map| map.is_empty())
        }

        /// Get the number of extensions available.
        ///
        /// # Example
        ///
        /// ```
        /// # use uactor::actor::context::extensions::Extensions;
        /// let mut ext = Extensions::new();
        /// assert_eq!(ext.len(), 0);
        /// ext.insert(5i32);
        /// assert_eq!(ext.len(), 1);
        /// ```
        #[inline]
        pub fn len(&self) -> usize {
            self.map.as_ref().map_or(0, |map| map.len())
        }

        /// Extends `self` with another `Extensions`.
        ///
        /// If an instance of a specific type exists in both, the one in `self` is overwritten with the
        /// one from `other`.
        ///
        /// # Example
        ///
        /// ```
        /// # use uactor::actor::context::extensions::Extensions;
        /// let mut ext_a = Extensions::new();
        /// ext_a.insert(8u8);
        /// ext_a.insert(16u16);
        ///
        /// let mut ext_b = Extensions::new();
        /// ext_b.insert(4u8);
        /// ext_b.insert("hello");
        ///
        /// ext_a.extend(ext_b);
        /// assert_eq!(ext_a.len(), 3);
        /// assert_eq!(ext_a.get::<u8>(), Some(&4u8));
        /// assert_eq!(ext_a.get::<u16>(), Some(&16u16));
        /// assert_eq!(ext_a.get::<&'static str>().copied(), Some("hello"));
        /// ```
        pub fn extend(&mut self, other: Self) {
            if let Some(other) = other.map {
                if let Some(map) = &mut self.map {
                    map.extend(*other);
                } else {
                    self.map = Some(other);
                }
            }
        }
    }

    impl fmt::Debug for Extensions {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Extensions").finish()
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    #[must_use]
    pub struct Service<T>(pub T);

    impl<T> DerefMut for Service<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<T> Deref for Service<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, Clone, Copy)]
    #[must_use]
    pub enum Actor {
        NamedActor { name: &'static str },
        All,
        First,
    }

    #[derive(thiserror::Error, Debug)]
    pub enum ExtensionErrors {
        #[error("Type {kind:?} is not registered within system context {system_name:?}")]
        NotRegisteredType { kind: String, system_name: Arc<str> },
    }
}

pub mod actor_registry {
    use crate::actor::abstract_actor::Actor;
    use crate::actor::context::extensions::IdHasher;
    use crate::actor::message::Message;
    use crate::data::data_publisher::{DataPublisher, TryClone, TryCloneError};
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::fmt;
    use std::hash::BuildHasherDefault;
    use std::ops::{Deref, DerefMut};
    use std::sync::Arc;

    type AnyBoxed = Box<dyn Any + Send + Sync>;

    #[derive(Default)]
    pub struct ActorRegistry {
        inner: HashMap<TypeId, HashMap<Arc<str>, AnyBoxed>, BuildHasherDefault<IdHasher>>,
    }

    impl ActorRegistry {
        /// Create an empty `ActorRegistry`.
        #[inline]
        pub fn new() -> Self {
            Self::default()
        }

        pub fn register_ref<A, M, D>(
            &mut self,
            actor_name: Arc<str>,
            channel: D,
            state: A::State,
        ) -> Option<D>
        where
            A: Actor,
            M: Message,
            D: DataPublisher<Item = M> + Send + Sync + 'static,
        {
            let entry = self.inner.entry(TypeId::of::<A>()).or_default();
            entry
                .insert(actor_name, Box::new((channel, state)))
                .and_then(|boxed| {
                    (boxed as Box<dyn Any + 'static>)
                        .downcast()
                        .ok()
                        .map(|boxed| *boxed)
                })
        }
        // TODO: docs
        pub fn get_all<A, M, D>(&self) -> Option<Vec<&(D, A::State)>>
        where
            A: Actor,
            M: Message,
            D: DataPublisher<Item = M> + Send + Sync + 'static,
        {
            self.inner
                .get(&TypeId::of::<A>())?
                .values()
                .map(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
                .collect::<Option<Vec<&(D, A::State)>>>()
        }

        // TODO: docs
        pub fn get_actor_ref<T, R>(&self, actor_name: Arc<str>) -> Option<&R>
        where
            T: Send + Sync + 'static,
            R: 'static,
        {
            let group_by_type = self.inner.get(&TypeId::of::<T>())?;
            let boxed_actor_ref = group_by_type.get(&actor_name)?;
            (&**boxed_actor_ref as &(dyn Any + 'static)).downcast_ref()
        }

        // TODO: docs
        pub fn remove<T: Send + Sync + 'static>(&mut self, actor_name: Arc<str>) -> Option<()> {
            self.inner
                .get_mut(&TypeId::of::<T>())?
                .remove(&actor_name)?;
            Some(())
        }
    }

    impl fmt::Debug for ActorRegistry {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("ActorRegistry").finish()
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    #[must_use]
    pub struct ActorRef<T>(pub T)
    where
        T: TryClone;

    impl<T: TryClone> TryClone for ActorRef<T> {
        fn try_clone(&self) -> Result<Self, TryCloneError> {
            Ok(ActorRef(self.0.try_clone()?))
        }
    }

    impl<T: TryClone> DerefMut for ActorRef<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<T: TryClone> Deref for ActorRef<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(thiserror::Error, Debug)]
    pub enum ActorRegistryErrors {
        #[error("Actor {kind:?} with name: {actor_name:?} is not registered within system context {system_name:?}")]
        NotRegisteredActor {
            system_name: Arc<str>,
            kind: String,
            actor_name: Arc<str>,
        },

        #[error("Actor {kind} is not registered within system context {system_name:?}")]
        NotRegisteredActorKind { system_name: Arc<str>, kind: String },

        #[error("Can't downcast registered actor into: {kind:?}, system: {system_name:?}")]
        CantDowncast { system_name: Arc<str>, kind: String },

        #[error(transparent)]
        TryCloneError(#[from] TryCloneError),
    }
}
