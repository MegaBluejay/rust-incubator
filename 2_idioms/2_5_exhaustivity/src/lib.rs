pub trait EventSourced<Ev: ?Sized> {
    fn apply(&mut self, event: &Ev);
}

pub mod user {
    use std::time::SystemTime;

    use super::{event, EventSourced};

    #[derive(Debug)]
    #[non_exhaustive]
    pub struct User {
        pub id: Id,
        pub name: Option<Name>,
        pub online_since: Option<SystemTime>,
        pub created_at: CreationDateTime,
        pub last_activity_at: LastActivityDateTime,
        pub deleted_at: Option<DeletionDateTime>,
    }

    impl EventSourced<event::UserCreated> for User {
        fn apply(&mut self, ev: &event::UserCreated) {
            let Self {
                ref mut id,
                name: _,
                online_since: _,
                ref mut created_at,
                ref mut last_activity_at,
                deleted_at: _,
            } = self;

            let event::UserCreated { user_id, at } = ev;

            *id = *user_id;
            *created_at = *at;
            *last_activity_at = (*at).into();
        }
    }

    impl EventSourced<event::UserNameUpdated> for User {
        fn apply(&mut self, ev: &event::UserNameUpdated) {
            let Self {
                id: _,
                ref mut name,
                online_since: _,
                created_at: _,
                ref mut last_activity_at,
                deleted_at: _,
            } = self;

            let event::UserNameUpdated {
                user_id: _,
                name: new_name,
                at,
            } = ev;

            *name = new_name.clone();
            *last_activity_at = (*at).into();
        }
    }

    impl EventSourced<event::UserBecameOnline> for User {
        fn apply(&mut self, ev: &event::UserBecameOnline) {
            let Self {
                id: _,
                name: _,
                ref mut online_since,
                created_at: _,
                ref mut last_activity_at,
                deleted_at: _,
            } = self;

            let event::UserBecameOnline { user_id: _, at } = ev;

            *online_since = Some(*at);
            *last_activity_at = (*at).into();
        }
    }

    impl EventSourced<event::UserBecameOffline> for User {
        fn apply(&mut self, ev: &event::UserBecameOffline) {
            let Self {
                id: _,
                name: _,
                ref mut online_since,
                created_at: _,
                ref mut last_activity_at,
                deleted_at: _,
            } = self;

            let event::UserBecameOffline { user_id: _, at } = ev;

            *online_since = None;
            *last_activity_at = (*at).into();
        }
    }

    impl EventSourced<event::UserDeleted> for User {
        fn apply(&mut self, ev: &event::UserDeleted) {
            let Self {
                id: _,
                name: _,
                online_since: _,
                created_at: _,
                ref mut last_activity_at,
                ref mut deleted_at,
            } = self;

            let event::UserDeleted { user_id: _, at } = ev;

            *deleted_at = Some(*at);
            *last_activity_at = (*at).into();
        }
    }

    #[derive(Debug)]
    #[non_exhaustive]
    pub enum Event {
        Created(event::UserCreated),
        NameUpdated(event::UserNameUpdated),
        Online(event::UserBecameOnline),
        Offline(event::UserBecameOffline),
        Deleted(event::UserDeleted),
    }

    impl EventSourced<Event> for User {
        fn apply(&mut self, ev: &Event) {
            match ev {
                Event::Created(inner_ev) => self.apply(inner_ev),
                Event::NameUpdated(inner_ev) => self.apply(inner_ev),
                Event::Online(inner_ev) => self.apply(inner_ev),
                Event::Offline(inner_ev) => self.apply(inner_ev),
                Event::Deleted(inner_ev) => self.apply(inner_ev),
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Id(pub u64);

    #[derive(Clone, Debug)]
    pub struct Name(pub Box<str>);

    #[derive(Clone, Copy, Debug)]
    pub struct CreationDateTime(pub SystemTime);

    impl From<SystemTime> for CreationDateTime {
        fn from(value: SystemTime) -> Self {
            Self(value)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct LastActivityDateTime(pub SystemTime);

    impl From<SystemTime> for LastActivityDateTime {
        fn from(value: SystemTime) -> Self {
            Self(value)
        }
    }

    impl From<CreationDateTime> for LastActivityDateTime {
        fn from(value: CreationDateTime) -> Self {
            Self(value.0)
        }
    }

    impl From<DeletionDateTime> for LastActivityDateTime {
        fn from(value: DeletionDateTime) -> Self {
            Self(value.0)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct DeletionDateTime(pub SystemTime);

    impl From<SystemTime> for DeletionDateTime {
        fn from(value: SystemTime) -> Self {
            Self(value)
        }
    }
}

pub mod event {
    use std::time::SystemTime;

    use super::user;

    #[derive(Debug)]
    pub struct UserCreated {
        pub user_id: user::Id,
        pub at: user::CreationDateTime,
    }

    #[derive(Debug)]
    pub struct UserNameUpdated {
        pub user_id: user::Id,
        pub name: Option<user::Name>,
        pub at: SystemTime,
    }

    #[derive(Debug)]
    pub struct UserBecameOnline {
        pub user_id: user::Id,
        pub at: SystemTime,
    }

    #[derive(Debug)]
    pub struct UserBecameOffline {
        pub user_id: user::Id,
        pub at: SystemTime,
    }

    #[derive(Debug)]
    pub struct UserDeleted {
        pub user_id: user::Id,
        pub at: user::DeletionDateTime,
    }
}
