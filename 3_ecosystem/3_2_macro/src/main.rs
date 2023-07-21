macro_rules! btreemap {
    ($($key:expr => $val:expr),*) => {
        {
            #[allow(unused_mut)]
            let mut __map = ::std::collections::BTreeMap::new();
            $(
                __map.insert($key, $val);
            )*
            __map
        }
    };
}

fn main() {
    let m1 = btreemap!["a".to_owned() => 1, "b".to_owned() => 2];
    let m2 = step_3_2_proc::btreemap!["a" => 3, "b" => 2 + 2];
    println!("{:?}, {:?}", m1, m2);
}
