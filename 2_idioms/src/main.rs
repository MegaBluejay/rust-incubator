use std::{borrow::Borrow, collections::HashMap, hash::Hash, num::NonZeroU32};

use enum_iterator::{reverse_all, Sequence};
use enum_map::{Enum, EnumMap};
use thiserror::Error;

#[derive(Debug)]
pub struct BaseVendingMachine {
    items: HashMap<Box<str>, Item>,
    max_products: usize,
    max_item_count: u32,
    coins: EnumMap<Coin, u32>,
}

#[derive(Debug)]
pub struct VendingMachine<State: VendingState> {
    base: BaseVendingMachine,
    state: State::State,
}

pub type Cost = NonZeroU32;

#[derive(Debug)]
pub struct Item {
    price: Cost,
    count: u32,
}

#[derive(Debug, Error)]
pub enum VendingError {
    #[error("no such product")]
    NoSuchProduct,
    #[error("product not available")]
    ProductNotAvailable,
    #[error("no change")]
    NoChange,
}

#[derive(Debug, Sequence, Clone, Copy, PartialEq, Enum)]
#[repr(u32)]
pub enum Coin {
    One = 1,
    Two = 2,
    Five = 5,
    Ten = 10,
    Twenty = 20,
    Fifty = 50,
}

#[derive(Debug, PartialEq)]
pub struct Product {
    name: Box<str>,
    price: Cost,
}

#[derive(Debug)]
pub struct TempState {
    product: Product,
    accepted: u32,
}

pub trait VendingState {
    type State;
}

#[derive(Debug)]
pub struct Ready;

#[derive(Debug)]
pub struct Accepting;

#[derive(Debug)]
pub struct Dispensing;

impl VendingState for Ready {
    type State = Ready;
}

impl VendingState for Accepting {
    type State = TempState;
}

impl VendingState for Dispensing {
    type State = TempState;
}

#[derive(Debug, PartialEq, Error)]
pub enum FillError {
    #[error("product already exists")]
    ProductExists,
    #[error("too many products")]
    TooManyProducts,
    #[error("no such product")]
    NoSuchProduct,
    #[error("too many items")]
    TooManyItems,
}

impl BaseVendingMachine {
    pub fn new(max_products: usize, max_item_count: u32) -> Self {
        Self {
            items: Default::default(),
            max_products,
            max_item_count,
            coins: Default::default(),
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

    pub fn fill_coins(&mut self, coins: impl IntoIterator<Item = Coin>) {
        for coin in coins {
            self.coins[coin] += 1;
        }
    }
}

impl<State: VendingState> VendingMachine<State> {
    pub fn products(&self) -> &HashMap<Box<str>, Item> {
        &self.base.items
    }

    fn as_coins(&mut self, mut change: u32) -> Result<Vec<Coin>, VendingError> {
        let mut res = vec![];
        for coin in reverse_all::<Coin>() {
            while (change >= coin as u32) && self.base.coins[coin] != 0 {
                res.push(coin);
                change -= coin as u32;
                self.base.coins[coin] -= 1;
            }
            if change == 0 {
                break;
            }
        }
        if change == 0 {
            Ok(res)
        } else {
            for coin in res {
                self.base.coins[coin] += 1;
            }
            Err(VendingError::NoChange)
        }
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
                let product = Product {
                    name: key.clone(),
                    price: item.price,
                };
                Ok(VendingMachine {
                    base: self.base,
                    state: TempState {
                        product,
                        accepted: 0,
                    },
                })
            }
        } else {
            Err((VendingError::NoSuchProduct, self))
        }
    }

    pub fn into_base(self) -> BaseVendingMachine {
        self.base
    }
}

impl VendingMachine<Accepting> {
    pub fn product(&self) -> &Product {
        &self.state.product
    }

    pub fn accepted(&self) -> u32 {
        self.state.accepted
    }

    fn enough(self) -> Result<VendingMachine<Dispensing>, Self> {
        if self
            .state
            .accepted
            .checked_sub(self.state.product.price.get())
            .is_some()
        {
            Ok(VendingMachine {
                base: self.base,
                state: self.state,
            })
        } else {
            Err(self)
        }
    }

    pub fn accept(mut self, coin: Coin) -> Result<VendingMachine<Dispensing>, Self> {
        self.state.accepted += coin as u32;
        self.base.coins[coin] += 1;
        self.enough()
    }

    pub fn accept_many(
        mut self,
        coins: impl IntoIterator<Item = Coin>,
    ) -> Result<VendingMachine<Dispensing>, Self> {
        for coin in coins {
            self.state.accepted += coin as u32;
            self.base.coins[coin] += 1;
        }
        self.enough()
    }

    pub fn cancel(mut self) -> (Vec<Coin>, VendingMachine<Ready>) {
        (
            self.as_coins(self.state.accepted).unwrap(),
            self.base.into(),
        )
    }
}

impl VendingMachine<Dispensing> {
    pub fn dispense(
        mut self,
    ) -> (
        Vec<Coin>,
        Result<Product, VendingError>,
        VendingMachine<Ready>,
    ) {
        let change = self.state.accepted - self.state.product.price.get();
        match self.as_coins(change) {
            Ok(coins) => {
                self.base
                    .items
                    .get_mut(&self.state.product.name)
                    .unwrap()
                    .count -= 1;
                (coins, Ok(self.state.product), self.base.into())
            }
            Err(err) => (
                self.as_coins(self.state.accepted).unwrap(),
                Err(err),
                self.base.into(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use nonzero_ext::nonzero;

    use super::*;

    #[test]
    fn max_products() -> Result<()> {
        let mut base = BaseVendingMachine::new(2, 0);

        base.add_product(Product {
            name: "p1".into(),
            price: nonzero!(1u32),
        })?
        .add_product(Product {
            name: "p2".into(),
            price: nonzero!(1u32),
        })?;

        assert_eq!(
            base.add_product(Product {
                name: "p3".into(),
                price: nonzero!(1u32)
            })
            .unwrap_err(),
            FillError::TooManyProducts
        );
        Ok(())
    }

    #[test]
    fn max_items() -> Result<()> {
        let mut base = BaseVendingMachine::new(1, 2);

        base.add_product(Product {
            name: "p1".into(),
            price: nonzero!(1u32),
        })?
        .fill_product("p1", 2)?;

        assert_eq!(
            base.fill_product("p1", 1).unwrap_err(),
            FillError::TooManyItems
        );
        Ok(())
    }

    #[test]
    fn purchase() -> Result<()> {
        let mut base = BaseVendingMachine::new(1, 1);

        base.add_product(Product {
            name: "p1".into(),
            price: nonzero!(3u32),
        })?
        .fill_product("p1", 1)?
        .fill_coins(vec![Coin::One]);

        let machine: VendingMachine<Ready> = base.into();
        let machine = machine.select("p1").map_err(|(err, _)| err)?;
        let machine = machine
            .accept_many(vec![Coin::Two, Coin::Two])
            .expect("should be enough");
        let (change, result, _machine) = machine.dispense();

        assert_eq!(change, vec![Coin::One]);
        assert_eq!(
            result?,
            Product {
                name: "p1".into(),
                price: nonzero!(3u32),
            }
        );

        Ok(())
    }
}

fn main() {}
