use post::Post;

mod post {
    use std::{marker::PhantomData, mem};

    use super::user;

    #[derive(Clone, Debug, PartialEq)]
    pub struct Id(pub u64);

    #[derive(Clone, Debug, PartialEq)]
    pub struct Title(pub String);

    #[derive(Clone, Debug, PartialEq)]
    pub struct Body(pub String);

    #[derive(Clone, Debug)]
    pub struct SomePost {
        pub id: Id,
        pub user_id: user::Id,
        pub title: Title,
        pub body: Body,
    }

    #[repr(transparent)]
    #[derive(Debug)]
    pub struct Post<S> {
        pub post: SomePost,
        state: PhantomData<S>,
    }

    impl<S> Clone for Post<S> {
        fn clone(&self) -> Self {
            Self {
                post: self.post.clone(),
                state: PhantomData,
            }
        }
    }

    #[derive(Debug)]
    pub struct New;

    #[derive(Debug)]
    pub struct Unmoderated;

    #[derive(Debug)]
    pub struct Published;

    #[derive(Debug)]
    pub struct Deleted;

    impl<T> Post<T> {
        fn change_state<U>(self) -> Post<U> {
            Post {
                post: self.post,
                state: PhantomData,
            }
        }
    }

    impl Post<New> {
        pub fn new(id: Id, user_id: user::Id, title: Title, body: Body) -> Self {
            Self {
                post: SomePost {
                    id,
                    user_id,
                    title,
                    body,
                },
                state: PhantomData,
            }
        }

        pub fn publish(self) -> Post<Unmoderated> {
            self.change_state()
        }
    }

    impl Post<Unmoderated> {
        pub fn allow(self) -> Post<Published> {
            self.change_state()
        }

        pub fn deny(self) -> Post<Deleted> {
            unsafe { mem::transmute(self) }
        }
    }

    impl Post<Published> {
        pub fn delete(self) -> Post<Deleted> {
            unsafe { mem::transmute(self) }
        }
    }
}

mod user {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Id(pub u64);
}

fn main() {
    let post1 = Post::new(
        post::Id(1),
        user::Id(1),
        post::Title("title".into()),
        post::Body("body".into()),
    );
    let post2 = post1.clone();

    let deled = [post1.publish().deny(), post2.publish().allow().delete()];
    println!("{:?}", deled);
}
