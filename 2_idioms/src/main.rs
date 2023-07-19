use std::{borrow::Borrow, collections::HashMap, hash::Hash, num::NonZeroU32};

use enum_iterator::{reverse_all, Sequence};

#[derive(Debug)]
struct BaseVendingMachine {
    items: HashMap<Box<str>, Item>,
    max_products: usize,
    max_item_count: u32,
}

pub struct VendingMachine<State> {
    base: BaseVendingMachine,
    state: State,
}

pub type Cost = NonZeroU32;

#[derive(Debug)]
pub struct Item {
    price: Cost,
    count: u32,
}

pub enum VendingError {
    NoSuchProduct,
    ProductNotAvailable,
}

#[derive(Debug, Sequence, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum Coin {
    One = 1,
    Two = 2,
    Five = 5,
    Ten = 10,
    Twenty = 20,
    Fifty = 50,
}

pub struct Product {
    name: Box<str>,
    price: Cost,
}

pub struct Accepting {
    selected: Product,
    accepted: u32,
}

pub struct Ready;

#[derive(Debug, PartialEq)]
pub enum FillError {
    ProductExists,
    TooManyProducts,
    NoSuchProduct,
    TooManyItems,
}

impl BaseVendingMachine {
    pub fn new(max_products: usize, max_item_count: u32) -> Self {
        Self {
            items: HashMap::new(),
            max_products,
            max_item_count,
        }
    }

    pub fn add_product(&mut self, product: Product) -> Result<&mut Self, FillError> {
        if self.items.contains_key(&product.name) {
            Err(FillError::ProductExists)
        } else if self.items.len() == self.max_products {
            Err(FillError::TooManyProducts)
        } else {
            self.items.insert(
                product.name,
                Item {
                    price: product.price,
                    count: 0,
                },
            );
            Ok(self)
        }
    }

    pub fn fill_product<Q>(&mut self, product_name: &Q, count: u32) -> Result<&mut Self, FillError>
    where
        Box<str>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(item) = self.items.get_mut(product_name) {
            let new_count = item.count + count;
            if new_count > self.max_item_count {
                Err(FillError::TooManyItems)
            } else {
                item.count = new_count;
                Ok(self)
            }
        } else {
            Err(FillError::NoSuchProduct)
        }
    }
}

impl<State> VendingMachine<State> {
    pub fn products(&self) -> &HashMap<Box<str>, Item> {
        &self.base.items
    }
}

impl From<BaseVendingMachine> for VendingMachine<Ready> {
    fn from(value: BaseVendingMachine) -> Self {
        VendingMachine {
            base: value,
            state: Ready,
        }
    }
}

impl VendingMachine<Ready> {
    pub fn select<Q>(
        self,
        product_name: &Q,
    ) -> Result<VendingMachine<Accepting>, (VendingError, Self)>
    where
        Box<str>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some((key, item)) = self.base.items.get_key_value(product_name) {
            if item.count == 0 {
                Err((VendingError::ProductNotAvailable, self))
            } else {
                let selected = Product {
                    name: key.clone(),
                    price: item.price,
                };
                Ok(VendingMachine {
                    base: self.base,
                    state: Accepting {
                        selected,
                        accepted: 0,
                    },
                })
            }
        } else {
            Err((VendingError::NoSuchProduct, self))
        }
    }
}

impl VendingMachine<Accepting> {
    pub fn selected(&self) -> &Product {
        &self.state.selected
    }

    pub fn accepted(&self) -> u32 {
        self.state.accepted
    }

    pub fn accept(
        mut self,
        coin: Coin,
    ) -> Result<(Product, Vec<Coin>, VendingMachine<Ready>), Self> {
        self.state.accepted += coin as u32;
        if let Some(change) = self
            .state
            .accepted
            .checked_sub(self.state.selected.price.get())
        {
            self.base
                .items
                .get_mut(&self.state.selected.name)
                .unwrap()
                .count -= 1;
            Ok((
                self.state.selected,
                Self::as_coins(change),
                self.base.into(),
            ))
        } else {
            Err(self)
        }
    }

    pub fn cancel(self) -> (Vec<Coin>, VendingMachine<Ready>) {
        (Self::as_coins(self.state.accepted), self.base.into())
    }

    fn as_coins(mut change: u32) -> Vec<Coin> {
        let mut res = vec![];
        for coin in reverse_all::<Coin>() {
            while change >= coin as u32 {
                res.push(coin);
                change -= coin as u32;
            }
            if change == 0 {
                break;
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use nonzero_ext::nonzero;

    use super::*;

    #[test]
    fn max_products() {
        let mut base = BaseVendingMachine::new(2, 0);

        base.add_product(Product {
            name: "p1".into(),
            price: nonzero!(1u32),
        })
        .unwrap()
        .add_product(Product {
            name: "p2".into(),
            price: nonzero!(1u32),
        })
        .unwrap();

        assert_eq!(
            base.add_product(Product {
                name: "p3".into(),
                price: nonzero!(1u32)
            })
            .unwrap_err(),
            FillError::TooManyProducts
        );
    }

    #[test]
    fn max_items() {
        let mut base = BaseVendingMachine::new(1, 2);

        base.add_product(Product {
            name: "p1".into(),
            price: nonzero!(1u32),
        })
        .unwrap()
        .fill_product("p1", 2)
        .unwrap();

        assert_eq!(
            base.fill_product("p1", 1).unwrap_err(),
            FillError::TooManyItems
        );
    }

    #[test]
    fn as_coins() {
        assert_eq!(
            VendingMachine::<Accepting>::as_coins(13),
            vec![Coin::Ten, Coin::Two, Coin::One]
        );
    }
}

fn main() {}
