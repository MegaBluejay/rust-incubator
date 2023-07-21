use std::collections::BTreeMap;

macro_rules! btreemap {
    ($($key:expr => $val:expr),*) => {
        {
            #[allow(unused_mut)]
            let mut m = BTreeMap::new();
            $(
                m.insert($key, $val);
            )*
            m
        }
    };
}

fn main() {
    let m = btreemap!["a".to_owned() => 2, "b".to_owned() => 4];
    println!("{:?}", m);
}
