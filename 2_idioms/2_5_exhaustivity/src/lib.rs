use std::time::SystemTime;

pub trait EventSourced<Ev: ?Sized> {
    fn apply(&mut self, event: &Ev);
}

pub trait Activity<T: ?Sized> {
    fn at(&self) -> &SystemTime;

    fn apply_rest(&self, sourced: &mut T);
}

pub mod user {
    use std::time::SystemTime;

    use super::{event, Activity, EventSourced};

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

    impl<Ev: ?Sized + Activity<User>> EventSourced<Ev> for User {
        fn apply(&mut self, event: &Ev) {
            self.last_activity_at = (*event.at()).into();
            event.apply_rest(self);
        }
    }

    impl Activity<User> for event::UserCreated {
        fn at(&self) -> &SystemTime {
            &self.at.0
        }

        fn apply_rest(&self, sourced: &mut User) {
            let Self { user_id, at } = self;
            let User {
                ref mut id,
                name: _,
                online_since: _,
                ref mut created_at,
                last_activity_at: _,
                deleted_at: _,
            } = sourced;

            *id = *user_id;
            *created_at = *at;
        }
    }

    impl Activity<User> for event::UserNameUpdated {
        fn at(&self) -> &SystemTime {
            &self.at
        }

        fn apply_rest(&self, sourced: &mut User) {
            let Self {
                user_id: _,
                name: new_name,
                at: _,
            } = self;
            let User {
                id: _,
                ref mut name,
                online_since: _,
                created_at: _,
                last_activity_at: _,
                deleted_at: _,
            } = sourced;

            *name = new_name.clone();
        }
    }

    impl Activity<User> for event::UserBecameOnline {
        fn at(&self) -> &SystemTime {
            &self.at
        }

        fn apply_rest(&self, sourced: &mut User) {
            let Self { user_id: _, at } = self;
            let User {
                id: _,
                name: _,
                ref mut online_since,
                created_at: _,
                last_activity_at: _,
                deleted_at: _,
            } = sourced;

            *online_since = Some(*at);
        }
    }

    impl Activity<User> for event::UserBecameOffline {
        fn at(&self) -> &SystemTime {
            &self.at
        }

        fn apply_rest(&self, sourced: &mut User) {
            let Self { user_id: _, at: _ } = self;
            let User {
                id: _,
                name: _,
                ref mut online_since,
                created_at: _,
                last_activity_at: _,
                deleted_at: _,
            } = sourced;

            *online_since = None;
        }
    }

    impl Activity<User> for event::UserDeleted {
        fn at(&self) -> &SystemTime {
            &self.at.0
        }

        fn apply_rest(&self, sourced: &mut User) {
            let Self { user_id: _, at } = self;
            let User {
                id: _,
                name: _,
                online_since: _,
                created_at: _,
                last_activity_at: _,
                ref mut deleted_at,
            } = sourced;

            *deleted_at = Some(*at);
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
